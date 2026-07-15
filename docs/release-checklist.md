# Windows 发布清单

此清单是发布阻断清单，不是“尽力而为”的建议。未满足任一必需项时只能发布内部测试包，不得宣称公开正式版。

## 构建与自动化

- [ ] 在干净检出运行 `pnpm install --frozen-lockfile`。
- [ ] 运行 lint、类型检查、前端/端到端测试、Rust fmt/Clippy/workspace 测试。
- [ ] 运行 `pnpm --filter @prompt-hub/desktop exec tauri build` 并记录生成的 NSIS/MSI 文件与 SHA-256。
- [ ] 使用 [search-baseline.md](search-baseline.md) 的 release 基准命令记录当前性能。

## 安全与恢复

- [ ] 复查 [security-review.md](security-review.md)；所有标为发布阻断的项目必须关闭或在发布说明中明确限制。
- [ ] 在新 Windows 用户配置文件执行安装、首次启动、创建/搜索/导入/导出、备份/恢复及 MCP STDIO 发现。
- [ ] 验证卸载不删除应用数据和备份；验证恢复前安全备份可被再次恢复。
- [ ] 确认安装包、日志和证据目录不含密钥或提示词正文。

## 签名、更新与发布

- [ ] 使用用户控制的有效代码签名证书签名安装包，并验证签名链；没有证书则阻断公开发布。
- [ ] 配置经用户确认的更新源、签名密钥、升级前自动备份和回滚说明；未配置则不得声称自动更新可用。
- [ ] 更新 [CHANGELOG.md](../CHANGELOG.md)、数据库兼容性说明与恢复链接。
- [ ] 将命令输出、校验和与人工验收放入 `docs/release-evidence/<version>/`，不包含敏感数据。
