import { useState } from "react";

import type { PromptCompatibilityDraft, PromptMetadataDraft, PromptValidationDraft } from "@prompt-hub/contracts";

type PromptMetadataEditorProps = {
  promptId: string;
  initial?: {
    tool?: string;
    model?: string;
    compatibilityStatus?: PromptCompatibilityDraft["status"];
    effectiveness?: PromptValidationDraft["status"];
    rating?: number | null;
  };
  saveMetadata: (id: string, metadata: PromptMetadataDraft) => Promise<unknown>;
};

export function PromptMetadataEditor({
  initial,
  promptId,
  saveMetadata,
}: PromptMetadataEditorProps) {
  const [tool, setTool] = useState(initial?.tool ?? "");
  const [model, setModel] = useState(initial?.model ?? "");
  const [compatibilityStatus, setCompatibilityStatus] = useState<PromptCompatibilityDraft["status"]>(initial?.compatibilityStatus ?? "unknown");
  const [effectiveness, setEffectiveness] = useState<PromptValidationDraft["status"]>(initial?.effectiveness ?? "unverified");
  const [rating, setRating] = useState(initial?.rating == null ? "" : String(initial.rating));
  const [isSaving, setSaving] = useState(false);
  const [error, setError] = useState(false);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(false);
    const normalizedTool = tool.trim();
    const normalizedModel = model.trim();
    if (!normalizedTool && normalizedModel) {
      setError(true);
      return;
    }
    const numericRating = rating.trim() ? Number(rating) : null;
    if (numericRating !== null && (!Number.isInteger(numericRating) || numericRating < 1 || numericRating > 5)) {
      setError(true);
      return;
    }
    setSaving(true);
    try {
      await saveMetadata(promptId, {
        tool: normalizedTool || null,
        model: normalizedModel || null,
        compatibilityStatus: normalizedTool ? compatibilityStatus : "unknown",
        effectiveness,
        rating: numericRating,
        notes: null,
      });
    } catch {
      setError(true);
    } finally {
      setSaving(false);
    }
  }

  return (
    <details className="prompt-metadata-editor">
      <summary>编辑提示词信息</summary>
    <form aria-label="提示词元数据" onSubmit={submit}>
      <label>
        适用工具
        <input onChange={(event) => setTool(event.target.value)} placeholder="可选，例如 Codex" value={tool} />
      </label>
      <label>
        适用模型
        <input onChange={(event) => setModel(event.target.value)} value={model} />
      </label>
      <label>
        兼容性状态
        <select aria-label="兼容性状态" onChange={(event) => setCompatibilityStatus(event.target.value as PromptCompatibilityDraft["status"])} value={compatibilityStatus}>
          <option value="unknown">未知（尚未验证）</option>
          <option value="confirmed">已确认</option>
          <option value="unsupported">不支持</option>
        </select>
      </label>
      <label>
        有效性
        <select
          onChange={(event) => setEffectiveness(event.target.value as PromptValidationDraft["status"])}
          value={effectiveness}
        >
          <option value="unverified">未验证</option>
          <option value="effective">有效</option>
          <option value="ineffective">失效</option>
          <option value="needs_retest">待复测</option>
        </select>
      </label>
      <label>
        评分
        <input max="5" min="1" onChange={(event) => setRating(event.target.value)} type="number" value={rating} />
      </label>
      {error ? <p role="alert">无法保存元数据，请检查输入后重试。</p> : null}
      <button disabled={isSaving} type="submit">{isSaving ? "保存中…" : "保存元数据"}</button>
    </form>
    </details>
  );
}
