import { useRef, useState } from "react";

export function AiOptimizationReview({ body, cancel, optimize, promptId }: { body: string; cancel?: (taskId: string) => Promise<void>; optimize: (id: string, instruction: string, taskId: string) => Promise<{ body?: string }>; promptId: string }) {
  const [instruction, setInstruction] = useState("");
  const [candidate, setCandidate] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [cancelled, setCancelled] = useState(false);
  const cancelledTasks = useRef(new Set<string>());
  async function submit(event: React.FormEvent) {
    event.preventDefault(); setFailed(false); setCancelled(false);
    const taskId = crypto.randomUUID();
    setActiveTaskId(taskId);
    try {
      const result = await optimize(promptId, instruction, taskId);
      if (!cancelledTasks.current.has(taskId)) setCandidate(result.body ?? "");
    } catch {
      if (!cancelledTasks.current.has(taskId)) setFailed(true);
    } finally {
      setActiveTaskId((active) => active === taskId ? null : active);
    }
  }
  async function cancelOptimization() {
    if (activeTaskId === null || cancel === undefined) return;
    try {
      await cancel(activeTaskId);
      cancelledTasks.current.add(activeTaskId);
      setActiveTaskId(null);
      setCancelled(true);
    } catch { setFailed(true); }
  }
  return <form aria-label="AI 优化" onSubmit={submit}>
    <h3>AI 优化</h3><label>优化指令<textarea onChange={(event) => setInstruction(event.target.value)} required value={instruction} /></label>
    <button type="submit">生成优化草稿</button>
    {activeTaskId !== null && cancel !== undefined ? <button onClick={() => { void cancelOptimization(); }} type="button">取消优化</button> : null}
    {failed ? <p role="alert">无法生成优化草稿，请检查配置后重试。</p> : null}
    {cancelled ? <p role="status">优化已取消，已保留当前指令。</p> : null}
    {candidate !== null ? <pre aria-label="优化前后正文差异">- {body}{"\n"}+ {candidate}</pre> : null}
  </form>;
}
