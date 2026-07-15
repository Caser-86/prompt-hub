# 发布证据目录

每个候选版本在此目录建立以版本号命名的子目录，保存可复现但不含数据库、提示词正文或密钥的证据：

- 执行日期、Windows 版本、Node、pnpm、Rust 与 Tauri 版本；
- 完整验证命令及退出状态；
- 安装包 SHA-256；
- 安装、首次启动、卸载后数据保留和 MCP 发现的人工验收记录；
- [security-review.md](../security-review.md) 和 [recovery-runbook.md](../recovery-runbook.md) 的对应版本。

不得提交提示词数据库、备份、真实 API 密钥、未经脱敏的诊断日志或安装后用户数据。
