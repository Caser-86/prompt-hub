# 0.1.10 候选包构建与安装烟测证据

日期：2026-08-30  
分支：`codex/legacy-v5-schema-recovery`  
源码提交：`f68b59281a435f87b29958ff32881f3b208b62df`
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
| gitCommit | `7e1f671f7a2ccaee7680fb9ebd06daff6f262ca2` |
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
| `target/release/bundle/nsis/Prompt Hub_0.1.10_x64-setup.exe` | 3,691,936 | `03f603119fd875170f2e6ccb0f095db936686023129b54860abe2f3326372fea` | `NotSigned` |
| `target/release/bundle/msi/Prompt Hub_0.1.10_x64_en-US.msi` | 4,964,352 | `f5263bd218208933568be6bfde3c3a658e91b5baa2e933f1c3a61d080eac99aa` | `NotSigned` |

对应校验清单：`target/release/bundle/SHA256SUMS.txt`。签名检查使用 PowerShell `Get-AuthenticodeSignature`，两个包均无签名证书，因此只能作为本地候选包使用。

## NSIS 安装/启动/卸载烟测

在隔离临时目录
`C:\Users\MR\AppData\Local\Temp\PromptHub-install-smoke-0.1.10-20260830-r2`
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
