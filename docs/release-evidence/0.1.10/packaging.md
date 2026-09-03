# 0.1.10 候选包构建与安装烟测证据

日期：2026-09-01
分支：`feat/permanent-prompt-deletion`
源码提交：`10d88bb7735ebedbae41a0425fcaefcf2c0f24b8`
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
| gitCommit | `10d88bb7735ebedbae41a0425fcaefcf2c0f24b8` |
| tagCommit | `null`（候选包未使用 release tag） |
| migrationManifestSha256 | `5d80ebbe3c310d153ddd53049e23950e6c90e8fed148a7b9adaae96fb9a6e3df` |

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
`PRAGMA user_version` 为 `9`，并补齐了此前缺失的
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

## 2026-08-31 工作树复验（候选包）

本次针对正式可用性缺口的修复在未提交工作树上重新构建并验证。安装包明确标记为
`workingTree=dirty`，因此不能冒充可追溯的正式发布包；提交后必须重新运行
`scripts/verify-release.mjs` 并再次构建。

| 项目 | 结果 |
| --- | --- |
| HEAD | `10d88bb7735ebedbae41a0425fcaefcf2c0f24b8` |
| 前端测试 | `91 passed`（21 个文件；工作区总计 116 passed） |
| Playwright | `4 passed` |
| Rust | `cargo fmt --check`、Clippy、`cargo test --workspace` 通过 |
| NSIS | `Prompt Hub_0.1.10_x64-setup.exe`，3,718,531 字节，SHA-256 `E775628B8452DE6C949EC62D5F7FDA302B5375DD08D9DC5291E28D599E3FFEA3` |
| MSI | `Prompt Hub_0.1.10_x64_en-US.msi`，6,610,944 字节，SHA-256 `B85324D222BF9ABF95F790DD2AED8EF9F5BFD09A52FCDB22AA9458194EE4C8D2` |

本次复验仍未替代真实安装后的人工启动和外部 Codex 客户端发现测试；两项属于正式发布前的最后证据补齐项。

## 2026-09-02 工作树复验（候选包）

针对详情历史加载错误边界、契约枚举校验和交互后数据刷新修复后，重新完成前端构建与 Tauri 打包。当前工作树仍为未提交状态，包继续标记为本地候选包。

| 项目 | 结果 |
| --- | --- |
| HEAD | `10d88bb7735ebedbae41a0425fcaefcf2c0f24b8`（工作树有未提交改动） |
| 前端测试 | `122 passed`（contracts 26 + desktop 96；21 个文件） |
| Playwright | `4 passed` |
| Rust | `cargo fmt --check`、Clippy、`cargo test --workspace` 通过 |
| 前端构建 | `pnpm --filter @prompt-hub/desktop build` 通过 |
| 打包可执行文件启动 | 直接启动 `target/release/prompt-hub-desktop.exe`，等待 5 秒进程仍存活 |
| NSIS | `Prompt Hub_0.1.10_x64-setup.exe`，3,718,772 字节，SHA-256 `4F436B392E40F3F1B67ED5057E2655B36BF3F2EEDB64BDD685B1CF6280288642`，`NotSigned` |
| MSI | `Prompt Hub_0.1.10_x64_en-US.msi`，6,610,944 字节，SHA-256 `C3886B1A39203C3FF1E9B69A11632625A38E1D88CC18223A83DCFFBA6DC8278F`，`NotSigned` |

校验清单由 `pnpm hash:bundles` 重新生成。尚未执行干净 Windows 用户配置文件人工验收、代码签名和真实外部 Codex 客户端发现测试，因此本包仍只建议本地自用。
