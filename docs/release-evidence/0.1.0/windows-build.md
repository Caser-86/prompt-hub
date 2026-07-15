# 0.1.0 Windows 构建证据

日期：2026-07-15。本记录仅证明本地构建成功，不证明签名、公开发布、更新或干净用户配置文件验收已完成。

## 命令

```powershell
pnpm --filter @prompt-hub/desktop exec tauri build
```

结果：退出状态 0；生成 x64 MSI 与 NSIS 安装器。

| 文件 | SHA-256 |
| --- | --- |
| `Prompt Hub_0.1.0_x64_en-US.msi` | `29D88684644F1CC53BF4633081945A5E3E5B1E539E7E335FD852529973CDA45A` |
| `Prompt Hub_0.1.0_x64-setup.exe` | `0C726A225A68B637D85E147686196B40CD6CAA7E928E3523B83B16C2B3FED549` |

## 未完成的发布验收

- 没有提供代码签名证书，两个安装器均未验证签名。
- 尚未在干净 Windows 用户配置文件安装、首次启动、卸载后数据保留和 MCP 发现。
- 未配置自动更新服务、更新签名密钥或用户确认升级流程。
