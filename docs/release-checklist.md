# Windows 发布清单

此清单是发布阻断清单，不是“尽力而为”的建议。未满足任一必需项时只能发布内部测试包，不得宣称公开正式版。

## 构建与自动化

- [ ] 运行 `node scripts/verify-release.mjs --channel=candidate`；正式 tag 构建必须使用 `--channel=release`，并通过版本一致、干净工作树、注释 tag 及 `origin/main` 血缘校验。
- [ ] 保存构建生成的 `target/release/build-info.json`，核对版本、通道、Git commit 与迁移清单 SHA-256。
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

### 0.1.10 当前记录

0.1.10 候选包的构建、SHA-256、签名状态和 NSIS 安装/启动/卸载烟测已记录在 [packaging.md](release-evidence/0.1.10/packaging.md)。清单中的勾选项仍保持未勾选，因为代码签名、自动更新源、干净用户配置文件验收和完整人工发布验收尚未完成。
