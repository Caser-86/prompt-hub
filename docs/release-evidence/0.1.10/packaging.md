# 0.1.10 候选包构建与安装烟测证据

日期：2026-08-30
分支：`feat/permanent-prompt-deletion`
源码提交：`dfab89025e1851d95abf9b69536664f4df9d1008`
通道：`candidate`（自用内部候选包，不是公开正式发布包）

## 构建元数据

执行：

```powershell
node scripts/verify-release.mjs --channel=candidate --json-out=apps/desktop/src-tauri/resources/build-info.json
```

生成的元数据核对结果：

| 字段 | 值 |
| --- | --- |
| version | `0.1.10` |
| channel | `candidate` |
| gitCommit | `dfab89025e1851d95abf9b69536664f4df9d1008` |
| tagCommit | `null`（候选包未使用 release tag） |
| migrationManifestSha256 | `8cdee8a7a5d368eeeb0002e3e4cce5bbe59668d9f01863e392fdf27c68b53cd8` |

## 安装包

执行：

```powershell
pnpm --filter @prompt-hub/desktop exec tauri build
```

构建完成并生成两个 Windows 包：

| 包 | 大小（字节） | SHA-256 | 签名状态 |
| --- | ---: | --- | --- |
| `target/release/bundle/nsis/Prompt Hub_0.1.10_x64-setup.exe` | 3,694,544 | `58d7761bcfa336e7ebc45053dd6c0a0f0fb430a4a2886bceb2988133d826dd74` | `NotSigned` |
| `target/release/bundle/msi/Prompt Hub_0.1.10_x64_en-US.msi` | 6,578,176 | `2df61371d10480b4634c321e911c20fd7fee423eb38cbfb666fa3c4a8afc0bd3` | `NotSigned` |

对应校验清单：`target/release/bundle/SHA256SUMS.txt`。签名检查使用 PowerShell `Get-AuthenticodeSignature`，两个包均无签名证书，因此只能作为本地候选包使用。

MCP sidecar 单独构建于 `target/release/prompt-mcp.exe`（8,160,768 字节，SHA-256
`2ce8a5620075c4aba20d88cd4bd5a6ba20b21cb51990a0fd42995b804b82bf4e`，签名状态
`NotSigned`）。当前 Tauri 安装包不自动把 sidecar 放入系统 `PATH`，需按
[mcp-setup.md](../../mcp-setup.md) 单独安装或配置其绝对路径。

本次迁移修复还在现有本地数据库上完成了启动验收：应用启动后保持进程存活，
`PRAGMA user_version` 为 `8`，并补齐了此前缺失的
`20260829_01_prompt_metadata_and_usage` 账本记录（`provenance=legacy_recovery`）。
原有数据未被替换；安装前另行保存了本地数据库副本。

对 release sidecar 直接发送 `tools/list` 的协议验收返回 4 个工具：
`search_prompts`、`get_prompt`、`render_prompt`、`save_prompt_draft`。

## NSIS 安装/启动/卸载烟测

在隔离临时目录
`C:\Users\MR\AppData\Local\Temp\PromptHub-install-smoke-0.1.10-r4-d98a316b69624a0c9660d9ac1aa1143e`
执行静默安装、启动和卸载：

| 检查 | 结果 |
| --- | --- |
| 安装器退出码 | `0` |
| 安装文件 | `prompt-hub-desktop.exe`、`uninstall.exe`、`resources/build-info.json` 均生成 |
| 启动 | 已启动安装后的 `prompt-hub-desktop.exe`，等待 5 秒进程仍存活 |
| 卸载器退出码 | `0` |
| 卸载后安装目录 | 不存在 |

这证明候选 NSIS 包可以完成基本安装生命周期。它不等同于干净 Windows 用户配置文件的完整人工验收，也不覆盖 MSI 安装路径。

## 发布判断

- 候选包构建、版本元数据、SHA-256 和 NSIS 生命周期烟测已留证。
- 0.1.10 候选包还完成了 1,000、10,000 和 50,000 条本地数据的冷/热检索、组合筛选、索引重建和备份基准复测；结果见 [search-baseline.md](../../search-baseline.md) 的 2026-08-30 记录。
- 代码签名证书、自动更新源/签名密钥、干净用户配置文件的完整人工验收、真实外部 Codex MCP 发现仍未完成。
- 在上述外部条件完成前，不得将本包标为公开正式版本；本地自用可以使用 NSIS 候选包。

本记录不包含密钥、授权头或提示词正文。
