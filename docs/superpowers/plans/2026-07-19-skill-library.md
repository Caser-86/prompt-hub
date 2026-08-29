# Prompt Hub Skill 库实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Skill 作为独立的本地资产，实现安全收集、审核、搜索、收藏、受控安装、备份恢复和已安装文件漂移检测。

**Architecture:** 新增 `prompt-skill` Rust crate，负责不执行代码的 Skill 目录检查、`SKILL.md` 元数据解析、风险分类、哈希和安装文件操作。SQLite 在现有 `prompt-store` 中增加独立 Skill 表；Tauri commands 只传输摘要、审核和安装操作结果。React 使用独立 `features/skills` 模块，复用应用外壳、通知和紧凑列表样式，不复用提示词实体或 MCP 写入口。

**Tech Stack:** Rust 2024、SQLite/rusqlite、Tauri 2、React 19、TypeScript、Vitest。

## Global Constraints

- 所有收集、预览、检索和安装都不得执行 Skill 内脚本。
- 本地扫描、收件箱、审核、安装与备份不能依赖模型凭据或网络。
- 数据库每次变化须有前向迁移和迁移测试。
- 安装冲突默认拒绝；替换必须显式确认、先建立备份、完成后校验哈希。
- 日志、诊断和错误信息不得泄露 `SKILL.md` 或其它文本正文。

---

### Task 1: Skill 领域模型与只读目录审计

**Files:**
- Create: `crates/prompt-skill/Cargo.toml`
- Create: `crates/prompt-skill/src/lib.rs`
- Create: `crates/prompt-skill/src/scan.rs`
- Create: `crates/prompt-skill/tests/scan.rs`
- Modify: `Cargo.toml`

**Interfaces:**
- Produces `SkillCandidate::scan(root: &Path) -> Result<SkillCandidate, SkillScanError>`。
- `SkillCandidate` 含 `name`, `description`, `skill_markdown`, `files`, `risk_flags`, `content_hash`, `total_bytes`。
- `SkillFile` 含相对路径、字节数、SHA-256、`SkillFileKind`；`SkillRisk` 至少覆盖脚本、二进制、隐藏文件和符号链接。

- [ ] **Step 1: 写失败测试**：为正常 `SKILL.md`、脚本风险、符号链接拒绝、目录穿越和超出文件/字节限制建立临时目录测试。
- [ ] **Step 2: 运行失败测试**：`cargo test -p prompt-skill --test scan`，预期因 crate 与接口不存在失败。
- [ ] **Step 3: 实现纯本地扫描器**：递归读取常规文件；拒绝根目录之外的路径和缺失 `SKILL.md`；用 SHA-256 形成稳定内容哈希；不得调用进程或解释器。
- [ ] **Step 4: 验证**：`cargo test -p prompt-skill --test scan` 通过。
- [ ] **Step 5: 提交**：`git commit -m "feat: add safe skill scanner"`。

### Task 2: Skill SQLite 模型、迁移和仓储

**Files:**
- Create: `crates/prompt-store/migrations/0005_skills.sql`
- Modify: `crates/prompt-store/src/migration.rs`
- Modify: `crates/prompt-store/src/lib.rs`
- Create: `crates/prompt-store/src/skills.rs`
- Create: `crates/prompt-store/tests/skills.rs`
- Modify: `crates/prompt-store/tests/migrations.rs`

**Interfaces:**
- Produces `SkillRepository::{save_candidate,list_skills,get_skill,set_review,set_favorite,record_installation}`。
- `StoredSkill` 记录 ID、名称、来源、内容哈希、文件清单摘要、风险、审核状态、收藏和安装状态；不将正文加入 FTS 或诊断。

- [ ] **Step 1: 写失败迁移与仓储测试**：验证 schema 版本升级、保存后可读取、收藏排序、审核状态持久化，以及正文不进入诊断字段。
- [ ] **Step 2: 运行**：`cargo test -p prompt-store --test migrations --test skills`，预期失败。
- [ ] **Step 3: 新增 0005 前向迁移**：创建 `skills`、`skill_files`、`skill_installations` 表和必要索引；将 `LATEST_SCHEMA_VERSION` 更新为 5。
- [ ] **Step 4: 实现仓储**：参数化 SQL、事务写入、只返回 UI 所需摘要；同一来源+内容哈希幂等去重。
- [ ] **Step 5: 验证并提交**：运行上述测试，通过后 `git commit -m "feat: persist skill assets"`。

### Task 3: Tauri 收集、审核和查询命令

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `packages/contracts/src/index.ts`
- Modify: `packages/contracts/src/index.test.ts`
- Modify: `apps/desktop/src/services/desktop.ts`
- Create: `apps/desktop/src-tauri/tests/skills.rs`

**Interfaces:**
- Produces `scan_skill_folder(path)`, `list_skills(filters)`, `get_skill(id)`, `review_skill(id, status, notes)`, `set_skill_favorite(id, favorite)`。
- 所有收集结果默认 `pending_review`，不能由命令直接成为 `approved` 或已安装。

- [ ] **Step 1: 写失败合约测试**：断言 Tauri 可调用命令、Wire 类型的 snake/camel 转换和“扫描不会执行脚本”。
- [ ] **Step 2: 运行**：`cargo test -p prompt-hub-desktop --test skills` 与 `pnpm --filter @prompt-hub/contracts test`，预期失败。
- [ ] **Step 3: 实现 commands**：连接 `PromptService` 与 `SkillRepository`；只接受绝对本地目录；按扫描器上限处理错误；返回脱敏摘要。
- [ ] **Step 4: 验证并提交**：运行 Rust 和 contracts 测试，通过后 `git commit -m "feat: expose skill collection commands"`。

### Task 4: Skill 收件箱、库和详情 UI

**Files:**
- Create: `apps/desktop/src/features/skills/SkillLibrary.tsx`
- Create: `apps/desktop/src/features/skills/SkillLibrary.test.tsx`
- Create: `apps/desktop/src/features/skills/SkillDetail.tsx`
- Create: `apps/desktop/src/features/skills/skill-library.css`
- Modify: `apps/desktop/src/app/navigation.ts`
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/components/CommandPalette.tsx`
- Modify: `apps/desktop/src/styles.css`

**Interfaces:**
- 消费 Task 3 contracts；导航新路由为 `skills`。
- 列表显示名称、来源、审核、安装、风险、收藏和更新时间；详情正文以 `<pre>` 文本显示。

- [ ] **Step 1: 写失败 UI 测试**：验证导航、空状态、本地目录收集、风险徽章、审核确认、收藏筛选和详情不渲染 HTML。
- [ ] **Step 2: 运行**：`pnpm --filter @prompt-hub/desktop test -- SkillLibrary`，预期失败。
- [ ] **Step 3: 实现最小 UI**：新增“Skill 库”导航，目录输入与扫描按钮，待审核/已审核/已安装/收藏筛选，详情侧栏。
- [ ] **Step 4: 验证并提交**：运行对应测试、`pnpm lint`、`pnpm typecheck`，通过后 `git commit -m "feat: add skill library workspace"`。

### Task 5: 受控安装、备份和漂移检测

**Files:**
- Modify: `crates/prompt-skill/src/lib.rs`
- Create: `crates/prompt-skill/src/install.rs`
- Create: `crates/prompt-skill/tests/install.rs`
- Modify: `crates/prompt-store/src/skills.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `packages/contracts/src/index.ts`
- Modify: `apps/desktop/src/features/skills/SkillDetail.tsx`
- Modify: `apps/desktop/src/features/skills/SkillLibrary.test.tsx`

**Interfaces:**
- Produces `install_skill(id, target_root, mode)`；`mode` 只允许 `new_name` 与 `replace_after_backup`。
- `detect_skill_drift(id)` 只返回差异摘要，绝不写入目标。

- [x] **Step 1: 写失败测试**：正常安装、未审核拒绝、已存在默认拒绝、替换前备份、复制后哈希不一致回滚、脚本未执行和漂移检测。
- [x] **Step 2: 运行**：`cargo test -p prompt-skill --test install`，预期失败。
- [x] **Step 3: 实现安装器**：只复制受审清单中的常规文件到临时目录；校验后原子移动；替换先将原目录移至应用备份目录；失败时恢复备份。
- [x] **Step 4: 接入 UI**：仅审核通过时显示安装；冲突和替换必须由明确对话框确认；显示目标、备份和结果。
- [x] **Step 5: 验证并提交**：运行 Rust/UI 测试，通过后 `git commit -m "feat: install reviewed skills safely"`。

### Task 6: Git 候选导入、文档和发布验证

**Files:**
- Modify: `crates/prompt-skill/src/scan.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src/features/skills/SkillLibrary.tsx`
- Modify: `docs/skill-library-design.md`
- Modify: `docs/user-guide.md`
- Modify: `README.md`
- Create: `tests/e2e/skill-library.spec.ts`

**Interfaces:**
- `collect_skill_git_candidate(url, revision)` 仅接受公开 HTTPS Git URL、固定提交 SHA，并在应用临时目录中检出后交给同一只读扫描器。
- 网络失败、无效 URL 或非固定 revision 必须返回可操作错误，不能创建半成品资产。

- [x] **Step 1: 写失败测试**：HTTPS 允许、非 HTTPS/含凭据/内网拒绝、未固定 revision 拒绝、网络错误不落库。
- [x] **Step 2: 运行**：对应 crate 与 Tauri 测试，预期失败。
- [x] **Step 3: 实现最小 Git 候选收集**：使用受限 HTTPS、固定提交、临时目录和大小限制；不递归 submodule，不运行 hooks，不执行 Skill 文件。
- [x] **Step 4: 补充使用文档与端到端测试**：覆盖本地收集、审核、安装冲突默认取消与漂移只读检测。
- [x] **Step 5: 完整验证与提交**：运行 AGENTS.md 所列 pnpm、cargo 和桌面构建命令；`git commit -m "feat: complete skill library workflow"`。
