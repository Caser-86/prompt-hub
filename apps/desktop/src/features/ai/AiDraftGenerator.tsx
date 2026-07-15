import { useState } from "react";

import type { AiGenerationRequest } from "@prompt-hub/contracts";

export function AiDraftGenerator({ generateDraft }: { generateDraft: (request: AiGenerationRequest) => Promise<unknown> }) {
  const [endpoint, setEndpoint] = useState("https://api.openai.com");
  const [model, setModel] = useState("");
  const [instruction, setInstruction] = useState("");
  const [inputSummary, setInputSummary] = useState("");
  const [error, setError] = useState(false);
  const [complete, setComplete] = useState(false);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(false);
    setComplete(false);
    try {
      await generateDraft({ endpoint, providerId: "openai-compatible", instruction, inputSummary, model });
      setComplete(true);
    } catch {
      setError(true);
    }
  }

  return <form aria-label="AI 草稿生成" onSubmit={submit}>
    <h2>AI 草稿生成</h2>
    <p>成功结果只会创建收件箱草稿，绝不会覆盖已发布提示词。</p>
    <label>兼容 API 地址<input onChange={(event) => setEndpoint(event.target.value)} required type="url" value={endpoint} /></label>
    <label>模型<input onChange={(event) => setModel(event.target.value)} required value={model} /></label>
    <label>生成指令<textarea onChange={(event) => setInstruction(event.target.value)} required value={instruction} /></label>
    <label>输入摘要<textarea onChange={(event) => setInputSummary(event.target.value)} required value={inputSummary} /></label>
    <button type="submit">生成收件箱草稿</button>
    {complete ? <p role="status">草稿已创建到收件箱，请审核后发布。</p> : null}
    {error ? <p role="alert">无法生成草稿。请检查 API 地址、密钥、模型和网络后重试。</p> : null}
  </form>;
}
