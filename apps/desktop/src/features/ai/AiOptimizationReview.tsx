import { useState } from "react";

export function AiOptimizationReview({ body, optimize, promptId }: { body: string; optimize: (id: string, instruction: string) => Promise<{ body?: string }>; promptId: string }) {
  const [instruction, setInstruction] = useState("");
  const [candidate, setCandidate] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  async function submit(event: React.FormEvent) {
    event.preventDefault(); setFailed(false);
    try { setCandidate((await optimize(promptId, instruction)).body ?? ""); } catch { setFailed(true); }
  }
  return <form aria-label="AI 优化" onSubmit={submit}>
    <h3>AI 优化</h3><label>优化指令<textarea onChange={(event) => setInstruction(event.target.value)} required value={instruction} /></label>
    <button type="submit">生成优化草稿</button>
    {failed ? <p role="alert">无法生成优化草稿，请检查配置后重试。</p> : null}
    {candidate !== null ? <pre aria-label="优化前后正文差异">- {body}{"\n"}+ {candidate}</pre> : null}
  </form>;
}
