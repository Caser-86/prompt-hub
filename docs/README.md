# Prompt Hub 文档索引

这里按使用目的整理项目文档。产品行为以 [正式产品规格](prompt-hub-product-spec.md) 为准；本索引只负责导航，不替代规格、恢复手册或发布清单。

## 开始使用

- [用户指南](user-guide.md)：创建、导入、搜索、复用、备份、MCP 和 Skill 库的操作说明。
- [导入格式](import-formats.md)：本地文件、文件夹和公开网页导入的支持范围与限制。
- [Codex MCP 设置](mcp-setup.md)：安装 sidecar、配置 STDIO MCP 和排查连接问题。

## 产品与设计

- [正式产品规格](prompt-hub-product-spec.md)：目标、范围、数据模型、安全边界和验收标准；这是唯一的产品需求基线。
- [Skill 库设计](skill-library-design.md)：Skill 收集、快照、人工审核、哈希校验和受控安装设计。

## 数据、安全与恢复

- [本地隐私与数据边界](privacy.md)：本地存储、凭据、导入网络边界和日志脱敏规则。
- [安全审查记录](security-review.md)：已验证的安全边界、发布阻断项与复查命令。
- [数据恢复运行手册](recovery-runbook.md)：迁移失败、启动恢复页、备份校验和人工恢复流程。

## 发布与验证证据

- [Windows 发布清单](release-checklist.md)：候选包与公开发布的必需门禁。
- [发布证据目录说明](release-evidence/README.md)：版本证据的内容、命名和脱敏要求。
- [0.1.10 候选包证据](release-evidence/0.1.10/packaging.md)：构建、哈希、安装/启动/卸载烟测与已知限制。
- [0.1.10 Stage A 证据](release-evidence/0.1.10/stage-a.md)：数据库迁移与恢复验证。
- [0.1.10 Stage B/C 证据](release-evidence/0.1.10/stage-bc.md)：离线工作流与提示词元数据验证。
- [0.1.10 Stage D/E 安全证据](release-evidence/0.1.10/stage-de-security.md)：MCP 与 Skill 安全边界验证。
- [搜索性能基线](search-baseline.md)：本地数据规模、检索和备份基准记录。

## 计划与设计追踪

`superpowers/plans/` 和 `superpowers/specs/` 保存历史计划与设计规格，用于追踪决策和验证范围。新改动应先阅读正式产品规格，再引用对应计划；已经完成的计划不删除，以保留实现依据。

- [模型分批执行总计划](superpowers/plans/2026-07-15-prompt-hub-model-batched-execution.md)
- [迁移血缘与发布完整性](superpowers/plans/2026-08-24-migration-lineage-and-release-integrity.md)
- [提示词元数据与离线工作流](superpowers/plans/2026-08-29-prompt-metadata-and-offline-workflow.md)
- [阶段 A 数据库恢复](superpowers/plans/2026-08-29-stage-a-database-recovery.md)
- [正式产品缺口收敛](superpowers/plans/2026-08-31-formal-product-gap-closure.md)
- [文档整理与 GitHub 同步](superpowers/plans/2026-09-04-documentation-sync.md)

## 可复用数据

- [提示词导入种子](import-seeds/)：带来源标注的示例提示词，仅作为本地审核和导入测试数据。

## 文档维护规则

1. 需求变更先更新正式产品规格，再更新用户指南、设计或实现计划。
2. 发布状态必须以发布清单和对应版本证据为准；候选包不能写成公开正式版。
3. 文档、日志和证据不得提交数据库、备份、密钥、授权头或未脱敏提示词正文。
4. 新增文档后同步更新本索引，并用 `git diff --check` 检查格式。
