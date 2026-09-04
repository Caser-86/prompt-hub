# Prompt Hub

本地优先、可离线使用的 Windows 桌面提示词资产库。它帮助你收集、审核、搜索、复用和维护提示词，并为每条提示词保留来源、时间、适用工具/模型和有效性记录。

## English overview

Prompt Hub is a local-first Windows desktop workspace for collecting, reviewing, searching, and reusing prompts and Codex Skills. Core workflows work offline, while provenance, compatibility, usage, and effectiveness metadata stay attached to each asset. Codex/MCP and AI integrations are bounded by inbox-only writes and explicit review steps.

> 当前版本：`0.1.10`<br>
> 数据默认只保存在本机；没有 AI 密钥时，创建、搜索、导入、备份等核心功能仍可使用。

> **Project status:** Private, self-use candidate package. The current build is intended for local use; code signing, public release automation, and a hosted update service are not included.

## 能做什么

- **提示词库**：创建、编辑、分类、标签、收藏、复制与导出 Markdown。
- **快速定位**：使用顶部搜索框或 `Ctrl + K`，按标题、标签和来源查找提示词；词库会优先展示收藏和常用项。
- **高级筛选**：按有效性、评分、生命周期、来源、分类、工具、模型、时间与收藏状态筛选。
- **收件箱工作流**：手动录入、文件、文件夹和网页导入的内容先进入收件箱，审核后再发布。
- **版本与生命周期**：发布后每次编辑保留版本历史；支持归档、软删除、恢复，以及带安全备份的永久清除。
- **本地备份**：可创建、校验、恢复和清理本地备份。
- **AI 草稿**：可接入 OpenAI 兼容接口生成或优化草稿；结果只会进入收件箱。
- **Codex MCP**：通过本地 STDIO MCP 搜索、读取和渲染已发布提示词；MCP 新建内容只能写入收件箱。
- **Skill 库**：作为独立资产收集 Codex `SKILL.md` 目录，展示文件清单、哈希和脚本/二进制/隐藏文件风险；支持审核、收藏与受控安装。

## 安装与启动

1. 从 [GitHub Releases](https://github.com/Caser-86/prompt-hub/releases) 下载 Windows 安装包；若尚未发布安装包，可从仓库 Actions 或本地构建产物取得。
2. 运行安装程序并启动 **Prompt Hub**。
3. 在“提示词库”中新建提示词，或在“收件箱”导入已有内容。

应用升级、恢复备份和永久清除前都会创建或要求确认本地安全备份。建议在“设置 → 备份与恢复”中定期创建独立备份。

如果本地数据库升级无法确认来源，应用会打开“需要恢复本地数据”页面，保留原数据库并提供重试和脱敏诊断导出；不要手动删除数据库或备份文件。

## 快速使用

1. **收集**：在“收件箱”粘贴网页 URL、导入文件/文件夹，或手动创建草稿。
2. **审核**：补齐分类或标签、来源、适用工具/模型及有效性，然后发布。
3. **检索**：用 `Ctrl + K` 快速打开提示词；需要多条件检索时，在“提示词库”使用“高级筛选”。
4. **复用**：打开详情，填写变量并复制渲染后的提示词或导出 Markdown。
5. **维护**：将常用内容收藏；定期复测有效性；通过版本历史恢复可靠版本。

### Skill 库

1. 在左侧打开“Skill 库”，扫描本地目录，或填入公开 GitHub HTTPS 仓库、固定 40 位提交 SHA 和可选子目录。
2. Git 导入会只读取固定提交对象并保存到应用的本地隔离快照；不会检出仓库或执行其中脚本。
3. 审核 `SKILL.md`、文件清单和风险标记后，明确点击“审核通过”。
4. 输入 Codex Skill 目标目录后安装。默认遇到同名目录会取消；只有勾选“同名时先备份再替换”并通过二次确认才会替换。

Skill 收集、预览和安装均不执行 Skill 文件。当前 Git 收集仅限公开 GitHub HTTPS 仓库，且必须固定到一个完整提交 SHA。

## 数据、隐私与安全

- 提示词数据库、导入记录和备份默认保存在本机，不进行隐式上传或遥测。
- API 密钥通过系统凭据存储保存，不写入提示词数据库或诊断日志。
- 网页导入只接受公开 HTTP/HTTPS 文本响应，并拦截本机、内网和带凭据地址。
- 外部导入、AI 生成与 MCP 写入均只能创建收件箱草稿，不能覆盖已发布提示词。
- Skill 安装只复制已审核且哈希复核通过的文件；源文件发生变化、符号链接或同名冲突都会阻止默认安装。安装登记失败会自动恢复旧备份。
- 固定 Git 导入禁用 hooks、submodule、本地/扩展协议和外部凭据助手，并在读取文件内容前执行数量、大小和层级限制。

## Codex MCP

在“设置 → Codex MCP”中复制生成的配置，并添加到 Codex。可用能力包括：

- `search_prompts(query, filters, limit)`：搜索已发布提示词。
- `get_prompt(id)`：读取提示词完整内容。
- `render_prompt(id, variables)`：填充变量后渲染提示词。
- `save_prompt_draft(payload)`：仅创建收件箱草稿。

MCP 使用本地 STDIO，不会开启网络服务端口。

## 本地开发

### 环境要求

- Node.js `>= 20.15`
- pnpm `11.7.0`
- Rust 工具链与 Windows Visual Studio Build Tools（用于 Tauri 桌面端）

### 常用命令

```powershell
pnpm install --frozen-lockfile
pnpm dev
pnpm lint
pnpm typecheck
pnpm test
pnpm --filter @prompt-hub/desktop build
node scripts/verify-release.mjs --channel=candidate
```

Rust 工作区验证：

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 文档

- [文档索引](docs/README.md)：按使用、产品设计、数据安全、发布证据和历史计划分类的完整入口。
- [用户指南](docs/user-guide.md)：完整使用说明，包括导入、搜索、备份、AI 草稿和 MCP。
- [产品规格](docs/prompt-hub-product-spec.md)：产品范围、数据模型、安全要求与验收标准。
- [模型分批执行计划](docs/superpowers/plans/2026-07-15-prompt-hub-model-batched-execution.md)：项目分期计划。
- [Skill 库设计](docs/skill-library-design.md)：Skill 收集、审核与受控安装的安全边界与后续扩展方案。
- [变更日志](CHANGELOG.md)：版本变更记录。
- [提示词导入种子](docs/import-seeds)：可复用的提示词收集样本。

历史计划和设计规格会保留在 `docs/superpowers/` 中，用于追踪决策；新需求以产品规格为准。

## 贡献与反馈

欢迎提交 Issue 或 Pull Request。涉及数据模型、导入逻辑、MCP 行为或安全策略的改动，请先阅读产品规格并补充相应测试。

## 许可证

当前仓库尚未声明许可证；在引入第三方代码或对外分发前，请先补充许可证文件。
