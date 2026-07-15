# Codex MCP 设置

Prompt Hub MCP 服务器使用 STDIO，不会开启网络端口。在 Prompt Hub 的“设置 → Codex MCP”复制生成的配置；它会填入当前数据库文件的绝对路径。确保 `prompt-mcp` 可执行文件已安装且位于系统 `PATH`，再将配置加入 Codex。

## 启用

在 Prompt Hub 的“设置 → Codex MCP”确认本地数据库为“可用”，复制生成的配置。确保
`prompt-mcp` 可执行文件已安装且位于系统 `PATH`，再将配置加入 Codex 并重启或重新加载
Codex 的 MCP 配置。

配置示例：

```json
{
  "mcpServers": {
    "prompt-hub": {
      "command": "prompt-mcp",
      "env": {
        "PROMPT_HUB_DATABASE_PATH": "C:\\path\\to\\prompt-hub.db"
      }
    }
  }
}
```

可用工具：

- `search_prompts`：读取本地提示词摘要。
- `get_prompt`：读取完整提示词记录。
- `render_prompt`：按变量渲染提示词，不执行提示词中的任何内容。
- `save_prompt_draft`：只能创建 MCP 来源的收件箱草稿。

服务器未配置数据库路径或数据库不可用时会返回稳定的 `database_unavailable` 错误；不会回退到云端或创建替代数据库。应用的 MCP 设置页会显示当前数据库是否可用。

## 禁用与卸载

要暂时禁用，请从 Codex 的 MCP 配置中移除 `prompt-hub` 条目并重新加载配置；这不会删除
Prompt Hub 数据库或备份。要恢复使用，再加入同一条目即可。

要卸载 MCP 集成，先移除该条目，再从系统中卸载或移除 `prompt-mcp` 可执行文件。只有在不再
需要任何本地提示词和备份时，才应通过 Prompt Hub 的备份与恢复流程单独处理数据；卸载 MCP
不会也不应删除本地数据。

## 诊断

当 Prompt Hub 正在迁移数据库、恢复备份、数据库被锁定或路径不可用时，MCP 调用可能返回
`database_unavailable`。请等待该操作结束，或在应用“设置 → Codex MCP”确认数据库状态后
重试。不要通过创建空数据库或改写环境变量来绕过该错误。
