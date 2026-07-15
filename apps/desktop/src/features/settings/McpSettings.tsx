import { useEffect, useState } from "react";

import type { McpSetupInfo } from "@prompt-hub/contracts";

export function McpSettings({ getSetup }: { getSetup: () => Promise<McpSetupInfo> }) {
  const [setup, setSetup] = useState<McpSetupInfo | null>(null);

  useEffect(() => { void getSetup().then(setSetup).catch(() => setSetup(null)); }, [getSetup]);

  return <section aria-labelledby="mcp-settings-title">
    <h2 id="mcp-settings-title">Codex MCP 设置</h2>
    <p>MCP 只读取本地库或将新内容写入收件箱；不会覆盖已发布提示词。</p>
    {setup ? <>
      <p role="status">本地数据库：{setup.databaseAvailable ? "可用" : "不可用"}</p>
      <p>确保 <code>prompt-mcp</code> 已在系统 PATH 中，然后将下列配置添加到 Codex。</p>
      <pre aria-label="Codex MCP 配置">{setup.configuration}</pre>
    </> : <p>无法读取 MCP 设置。请确认本地数据库可用后重试。</p>}
  </section>;
}
