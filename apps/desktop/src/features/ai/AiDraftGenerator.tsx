import { useRef, useState } from "react";

import type { AiConnectionRequest, AiGenerationRequest } from "@prompt-hub/contracts";

const preferencesKey = "prompt-hub.ai.draft-settings";

function preferences() {
  try {
    const stored = JSON.parse(localStorage.getItem(preferencesKey) ?? "{}") as { endpoint?: unknown; model?: unknown };
    return { endpoint: typeof stored.endpoint === "string" ? stored.endpoint : "https://api.openai.com", model: typeof stored.model === "string" ? stored.model : "" };
  } catch { return { endpoint: "https://api.openai.com", model: "" }; }
}

export function AiDraftGenerator({ cancelGeneration, generateDraft, testConnection }: { cancelGeneration?: (taskId: string) => Promise<void>; generateDraft: (request: AiGenerationRequest) => Promise<unknown>; testConnection: (request: AiConnectionRequest) => Promise<unknown> }) {
  const [endpoint, setEndpoint] = useState(() => preferences().endpoint);
  const [model, setModel] = useState(() => preferences().model);
  const [instruction, setInstruction] = useState("");
  const [inputSummary, setInputSummary] = useState("");
  const [error, setError] = useState(false);
  const [complete, setComplete] = useState(false);
  const [connectionComplete, setConnectionComplete] = useState(false);
  const [connectionError, setConnectionError] = useState(false);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [cancelled, setCancelled] = useState(false);
  const cancelledTasks = useRef(new Set<string>());

  async function testConnectionRequest() {
    setConnectionComplete(false);
    setConnectionError(false);
    try {
      await testConnection({ endpoint, providerId: "openai-compatible", model });
      localStorage.setItem(preferencesKey, JSON.stringify({ endpoint, model }));
      setConnectionComplete(true);
    } catch {
      setConnectionError(true);
    }
  }

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(false);
    setComplete(false);
    setCancelled(false);
    const taskId = crypto.randomUUID();
    setActiveTaskId(taskId);
    try {
      await generateDraft({ taskId, endpoint, providerId: "openai-compatible", instruction, inputSummary, model });
      if (cancelledTasks.current.has(taskId)) return;
      localStorage.setItem(preferencesKey, JSON.stringify({ endpoint, model }));
      setComplete(true);
    } catch {
      if (!cancelledTasks.current.has(taskId)) setError(true);
    } finally {
      setActiveTaskId((active) => active === taskId ? null : active);
    }
  }

  async function cancelActiveGeneration() {
    if (activeTaskId === null || cancelGeneration === undefined) return;
    try {
      await cancelGeneration(activeTaskId);
      cancelledTasks.current.add(activeTaskId);
      setActiveTaskId(null);
      setCancelled(true);
    } catch {
      setError(true);
    }
  }

  return <form aria-label="AI 草稿生成" className="settings-card surface-card ai-draft-generator" onSubmit={submit}>
    <h2>AI 草稿生成</h2>
    <p>成功结果只会创建收件箱草稿，绝不会覆盖已发布提示词。</p>
    <label>兼容 API 地址<input onChange={(event) => setEndpoint(event.target.value)} required type="url" value={endpoint} /></label>
    <label>模型<input onChange={(event) => setModel(event.target.value)} required value={model} /></label>
    <label>生成指令<textarea onChange={(event) => setInstruction(event.target.value)} required value={instruction} /></label>
    <label>输入摘要<textarea onChange={(event) => setInputSummary(event.target.value)} required value={inputSummary} /></label>
    <button className="button-secondary" onClick={() => { void testConnectionRequest(); }} type="button">测试连接</button>
    <button className="button-primary" type="submit">生成收件箱草稿</button>
    {activeTaskId !== null && cancelGeneration !== undefined ? <button onClick={() => { void cancelActiveGeneration(); }} type="button">取消生成</button> : null}
    {connectionComplete ? <p role="status">连接测试成功，未写入提示词库。</p> : null}
    {connectionError ? <p role="alert">连接测试失败。请检查 API 地址、密钥、模型和网络后重试。</p> : null}
    {complete ? <p role="status">草稿已创建到收件箱，请审核后发布。</p> : null}
    {cancelled ? <p role="status">生成已取消，已保留当前输入。</p> : null}
    {error ? <p role="alert">无法生成草稿。请检查 API 地址、密钥、模型和网络后重试。</p> : null}
  </form>;
}
