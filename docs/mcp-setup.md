# Codex MCP 设置

Prompt Hub MCP 服务器使用 STDIO，不会开启网络端口。在 Prompt Hub 的“设置 → Codex MCP”复制生成的配置；它会填入当前数据库文件的绝对路径。确保 `prompt-mcp` 可执行文件已安装且位于系统 `PATH`，再将配置加入 Codex。

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
