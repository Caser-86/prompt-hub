# 数据库迁移血缘与发布完整性设计

## 目标

Prompt Hub 必须能够安全升级所有已发布的本地数据库，且不得因迁移历史分叉、错误安装包或启动期数据库错误而崩溃、丢失提示词或静默创建不兼容状态。

本设计同时解决三个问题：

1. 数据库迁移不能再依赖可被不同分支重复使用的整数版本号。
2. 数据库升级失败时，桌面程序必须显示可恢复的状态，而不是在 Tauri setup 阶段 panic。
3. 只有可追溯到正式集成分支和发布 tag 的提交可以生成正式候选安装包。

## 非目标

- 不实现自动在线更新、代码签名服务或云端数据库同步。
- 不修改既有提示词正文、版本历史、收藏、兼容性或使用时间。
- 不删除旧 `schema_migrations` 和 `PRAGMA user_version`；它们保留为历史诊断信息，不能再作为迁移真相来源。

## 全局约束

- 升级前必须创建并校验本地备份；任何失败均不得替换原数据库。
- 所有数据库写入继续离线执行，不记录提示词正文、密钥、授权头或未脱敏路径到诊断日志。
- 迁移只允许向前执行；不支持自动降级。
- 每项行为变更遵循 TDD：先观察失败测试，再写最小实现。
- 正式发布只从 `main` 的带注释 Git tag `v<package-version>` 构建；内部候选包必须标记为 `candidate`，不得作为正式版发布。

## 现状与事故原因

`feat/session-a-foundations` 的 0.1.2 将 `PRAGMA user_version = 5` 用于 `prompts.last_used_at`；随后 `feat/permanent-prompt-deletion` 也将版本 5 用于 `skills` 表。两个安装包共用 `app.prompthub.desktop` 的 AppData 数据库。

因此，后一安装包把旧数据库误判为已执行 Skill 迁移，跳过建表；随后启动期访问 `skills` 表失败。Tauri 的 `.run(...).expect(...)` 将 setup 错误转为进程终止，Windows 只显示原生崩溃码。

0.1.10 已对这一已知结构组合做了兼容修复。以下设计把该临时结构识别提升为可验证的长期机制。

## 架构

### 1. 迁移账本为唯一真相

新增 `migration_ledger` 表：

```sql
CREATE TABLE migration_ledger (
    migration_id TEXT PRIMARY KEY,
    checksum_sha256 TEXT NOT NULL,
    applied_at INTEGER NOT NULL,
    provenance TEXT NOT NULL CHECK(provenance IN ('canonical', 'legacy_recovery'))
) STRICT;
```

每条未来迁移由一个不可复用的定义描述：

```rust
struct MigrationDefinition {
    id: &'static str,
    checksum_sha256: &'static str,
    sql: &'static str,
}
```

`id` 使用全局唯一、按时间排序的命名，例如 `20260824_01_migration_ledger` 与 `20260824_02_skill_snapshot_retention`。一个已发布的 `id`、SQL 文件和 SHA-256 永远不可修改或复用。

启动时按以下顺序处理：

1. 创建并完整性校验升级前备份。
2. 如果没有 `migration_ledger`，仅通过表、列和索引的只读结构指纹识别已支持的历史数据库。
3. 在单一事务中写入对应历史迁移账本记录；旧 0.1.2 的 `last_used_at` 标记为 `legacy/0.1.2-prompt-usage`。
4. 对每个 canonical 迁移：已有账本记录时校验 SHA-256；没有记录时执行 SQL 后写入账本。
5. 账本 ID 存在但 SHA-256 不同、数据库结构不在受支持历史集合、或迁移失败时中止事务并返回结构化错误。

`schema_migrations` 和 `user_version` 只用于旧库识别与人工诊断；新迁移不得根据其整数值跳过 SQL。

### 2. 历史数据库兼容策略

受支持的历史指纹固定为：

| 历史来源 | 判定条件 | 迁移账本结果 |
| --- | --- | --- |
| 初始至 v4 | `prompts`、搜索、收藏、导入表齐全，且无 `last_used_at`、无 `skills` | 写入 core 迁移记录，再应用 Skill 和后续迁移 |
| 0.1.2 使用排序库 | `prompts.last_used_at` 存在，`skills` 不存在 | 写入 core 与 `legacy/0.1.2-prompt-usage`，随后应用 Skill 和后续迁移 |
| Skill 库 v6 | `skills` 及 `skills.snapshot_path` 存在 | 写入对应 core、Skill、快照记录 |

任何不符合上述指纹的数据库都不得猜测迁移路径。程序进入恢复状态，保留原数据库和自动备份，允许用户导出诊断摘要、选择备份、或在未来版本支持后重试。

### 3. 无崩溃启动与恢复界面

将 `AppShell` 的 setup 过程拆为 `bootstrap_application()`，返回：

```rust
enum BootstrapState {
    Ready(DesktopServices),
    Recovery(BootstrapFailure),
}

struct BootstrapFailure {
    code: &'static str,
    safe_message: String,
    backup_path: Option<PathBuf>,
}
```

Tauri setup 始终成功注册 `BootstrapState`、`get_bootstrap_status`、`retry_database_bootstrap` 和只读 `export_bootstrap_diagnostics`。只有 `Ready` 状态注册或允许提示词、Skill、搜索与设置命令；`Recovery` 状态下这些命令返回统一的 `application_recovery_required` 错误。

前端首先读取启动状态。`RecoveryScreen` 必须显示：安全错误说明、是否已创建备份、备份文件名（不显示提示词正文）、“重试”、"导出诊断摘要" 和“打开恢复说明”操作。任何未知迁移错误都不得调用 `.expect` 或使主进程退出。

### 4. 发布血缘与安装包证明

新增 `scripts/verify-release.mjs`，在本地发布和 CI 中执行。它必须：

1. 读取根 `package.json`、`apps/desktop/package.json`、`tauri.conf.json` 和 Cargo workspace 版本，要求全部一致。
2. 要求工作树干净，当前提交带精确 tag `v<version>`。
3. 要求 tag 提交是 `origin/main` 的祖先；候选构建只能显式指定 `--channel=candidate`，且不得生成公开发布步骤。
4. 计算迁移定义清单 SHA-256、当前 Git commit SHA 和前端资源 SHA-256。
5. 生成随安装包嵌入的 `build-info.json`：`version`、`channel`、`gitCommit`、`migrationManifestSha256` 与 `builtAt`。

GitHub Actions 的正式发布工作流只在 `v*` tag 上运行，先 fetch 完整 `main` 历史，再运行 `scripts/verify-release.mjs --channel=release`。未通过时不得上传或发布安装包。手工 workflow 只允许 `candidate` 通道，并在 artifact 名称中包含 commit SHA。

### 5. CI 与测试矩阵

新增合成数据库 fixture 和测试覆盖：

- 新建数据库：写入完整迁移账本，账本校验可重复执行。
- 每个已发布结构（v4、0.1.2 的 v5 使用排序库、Skill v6）：升级后保留原提示词、收藏、版本和 `last_used_at`，并补齐当前表。
- 已存在账本 ID 的错误 SHA-256：启动进入 `Recovery`，不执行 SQL、不修改数据库。
- 未知结构：启动进入 `Recovery`，保留备份且不产生部分表。
- 迁移执行中失败：整个事务回滚，恢复状态包含安全错误代码。
- 桌面端：`RecoveryScreen` 渲染、重试成功后进入库页面、业务命令在恢复状态被拒绝。
- 发布脚本：版本不一致、脏工作树、错误 tag、tag 不在 main、候选通道与 release 通道均有正反例。
- 发布候选：在新的 Windows 用户配置文件升级各 fixture，验证程序持续运行、提示词库页面完整渲染，并保存不含敏感内容的证据。

## 代码边界

| 范围 | 职责 |
| --- | --- |
| `crates/prompt-store/src/migration.rs` | 历史结构识别、账本校验、原子迁移与结构化迁移错误 |
| `crates/prompt-store/migrations/` | 不可变 SQL 迁移文件与 ledger 创建迁移 |
| `crates/prompt-store/tests/migrations.rs` | 合成旧库、校验和冲突、回滚和数据保留测试 |
| `apps/desktop/src-tauri/src/bootstrap.rs` | 无 panic 的应用启动状态与受限服务访问 |
| `apps/desktop/src-tauri/src/lib.rs` | 注册 bootstrap 命令，移除 setup 失败导致的主进程终止 |
| `apps/desktop/src/features/recovery/` | 恢复页和安全诊断交互 |
| `packages/contracts/src/index.ts` | Bootstrap 状态与受限命令错误的前端契约 |
| `scripts/verify-release.mjs` | 正式/候选发布来源、版本与迁移清单校验 |
| `.github/workflows/release.yml` | 强制 release gate、候选 artifact 命名与证据上传 |
| `docs/release-checklist.md` | 发布前数据库矩阵、构建血缘与人工验收清单 |

## 验收标准

1. 任何受支持历史数据库升级后可启动完整桌面应用，且用户现有提示词数据不丢失。
2. 不认识的数据库或迁移校验和冲突时，应用显示恢复页，不产生 Windows 原生崩溃。
3. 每个新迁移在 CI 中有唯一 ID、固定 SHA-256，并被历史升级矩阵覆盖。
4. GitHub 不能从 feature 分支或不在 `main` 的 tag 生成 release 通道安装包。
5. 已安装应用的“诊断”可显示版本、Git commit、发布通道与迁移清单摘要，不显示提示词正文或密钥。

## 风险与迁移顺序

先上线账本与历史识别，再上线恢复页，最后启用 CI 的正式 release 阻断。CI 不能早于账本和历史 fixture 启用，否则只会验证旧的整数版本机制。每个阶段均在独立提交中完成，可单独测试和回滚；任何阶段都不删除已有本地备份。
