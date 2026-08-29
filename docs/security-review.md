# 安全审查记录

审查日期：2026-07-15。本文仅记录代码和自动化测试已验证的结论；不把未执行的人工或发布环境检查视为通过。

## 已验证边界

| 边界 | 已验证措施 | 证据 |
| --- | --- | --- |
| URL 导入 | 仅允许 HTTP/HTTPS；拒绝 URL 凭据；每次解析和重定向均拒绝本机、私网、链路本地、保留与混合 DNS 地址；限制超时、跳转和响应大小。 | `crates/prompt-import/tests/url_policy.rs`、`url_import.rs` |
| 不可信内容 | 文件、URL、AI 与 MCP 内容进入收件箱；前端以 React 文本节点和 `pre` 展示，未使用 `dangerouslySetInnerHTML`。 | `crates/prompt-domain/tests/domain_rules.rs`、`crates/prompt-mcp/tests/tools.rs`、源码检索 |
| 密钥和日志 | AI 凭据通过系统凭据适配器保存；脱敏器去除 URL 用户名、密码、查询值与完整请求头，正文的 `Display`/`Debug` 均为脱敏值。 | `crates/prompt-security/tests/security_contract.rs` |
| MCP 写入 | STDIO MCP 只有四个版本化工具；写工具只能创建收件箱草稿，不能发布、覆盖或永久清除。 | `crates/prompt-mcp/tests/schema_contract.rs`、`stdio.rs`、`tools.rs` |
| 破坏性操作 | 恢复和永久清除前创建完整性校验备份；恢复预览先校验数据库；软删除才能进入永久清除确认。 | `crates/prompt-store/tests/backup.rs`、桌面命令测试 |
| 并发基础 | SQLite 启用外键、WAL 与 5 秒 busy timeout。 | `crates/prompt-store/src/migration.rs` |
| 迁移血缘 | 拒绝未来架构版本、空或缺项的已提交迁移账本、未知迁移与校验和冲突；旧 0.1.2 prompt-usage 血缘可升级并再次打开。 | `crates/prompt-store/tests/migrations.rs` |
| 架构完整性 | 迁移提交前验证版本 7 的全部应用表和关键字段；账本完整但实际结构缺损时 fail closed。 | `crates/prompt-store/tests/migrations.rs` |
| 迁移备份 | 迁移前使用 SQLite 在线备份 API 创建一致快照，包含尚在 WAL 中的已提交记录。 | `pre_migration_backup_includes_committed_rows_still_in_the_wal` |
| 启动数据路径 | 系统应用数据目录无法解析或为空时进入恢复状态；准备服务与重试不会打开相对路径数据库。 | `apps/desktop/src-tauri/tests/bootstrap.rs` |

## 已知限制与发布阻断项

- 首发未提供数据库加密或应用锁；已在 [privacy.md](privacy.md) 说明本机磁盘访问风险。
- 尚未完成并记录桌面与 MCP 同时读写、备份、索引重建的压力试验。
- 尚未在干净 Windows 用户配置文件上完成签名安装包、卸载后数据保留和 MCP 发现的发布验收。
- 未提供代码签名证书或更新服务配置；因此不得宣称已签名公开发布或已启用自动更新。

## 2026-08-29 Stage A 复查

Stage A 已补充数据库 fail-closed、迁移账本完整性、最终架构形状、WAL 一致备份和应用数据目录不可用的自动化验证。详细命令与红/绿证据记录在 [stage-a.md](release-evidence/0.1.10/stage-a.md)。本次工作面向自用正式产品，不把代码签名、公开更新服务或公开 Release tag 作为本阶段完成条件。

## 复查命令

```powershell
cargo test -p prompt-security
cargo test -p prompt-import --test url_policy --test url_import
cargo test -p prompt-mcp --test schema_contract --test stdio --test tools
cargo test -p prompt-store --test backup
pnpm test
```
