# Codex MCP 设置

Prompt Hub MCP 服务器使用 STDIO。设置 `PROMPT_HUB_DATABASE_PATH` 为 Prompt Hub 数据库文件的绝对路径，然后将 `prompt-mcp` 可执行文件配置为 MCP 命令。

可用工具：

- `search_prompts`：读取本地提示词摘要。
- `get_prompt`：读取完整提示词记录。
- `render_prompt`：按变量渲染提示词，不执行提示词中的任何内容。
- `save_prompt_draft`：只能创建 MCP 来源的收件箱草稿。

服务器未配置数据库路径或数据库不可用时会返回稳定的 `database_unavailable` 错误；不会回退到云端或创建替代数据库。
