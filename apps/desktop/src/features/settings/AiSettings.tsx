import { useEffect, useState } from "react";

import type { AiCredentialStatus } from "@prompt-hub/contracts";

export function AiSettings({ getStatus, saveCredential }: {
  getStatus: (providerId: string) => Promise<AiCredentialStatus>;
  saveCredential: (providerId: string, secret: string) => Promise<AiCredentialStatus>;
}) {
  const [configured, setConfigured] = useState<boolean | null>(null);
  const [secret, setSecret] = useState("");
  const [error, setError] = useState(false);
  useEffect(() => { void getStatus("openai-compatible").then((status) => setConfigured(status.configured)).catch(() => setError(true)); }, [getStatus]);
  const save = async () => { setError(false); try { const status = await saveCredential("openai-compatible", secret); setConfigured(status.configured); setSecret(""); } catch { setError(true); } };
  return <section aria-labelledby="ai-settings-title"><h2 id="ai-settings-title">AI 设置</h2><p>密钥仅保存在 Windows 凭据管理器，不会保存到提示词库或显示在此页面。</p><p role="status">{configured === null ? "正在读取凭据状态…" : configured ? "已配置 AI 凭据" : "尚未配置 AI 凭据"}</p><label>OpenAI 兼容 API 密钥<input autoComplete="off" onChange={(event) => setSecret(event.target.value)} type="password" value={secret} /></label><button disabled={!secret} onClick={() => void save()} type="button">安全保存密钥</button>{error ? <p role="alert">无法访问系统凭据存储。</p> : null}</section>;
}
