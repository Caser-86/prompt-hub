import { useState } from "react";

import type { PromptCompatibilityDraft, PromptValidationDraft } from "@prompt-hub/contracts";

type PromptMetadataEditorProps = {
  promptId: string;
  saveCompatibility: (id: string, metadata: PromptCompatibilityDraft) => Promise<unknown>;
  saveValidation: (id: string, metadata: PromptValidationDraft) => Promise<unknown>;
};

export function PromptMetadataEditor({
  promptId,
  saveCompatibility,
  saveValidation,
}: PromptMetadataEditorProps) {
  const [tool, setTool] = useState("");
  const [model, setModel] = useState("");
  const [effectiveness, setEffectiveness] = useState<PromptValidationDraft["status"]>("unverified");
  const [rating, setRating] = useState("");
  const [error, setError] = useState(false);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(false);
    try {
      await Promise.all([
        saveCompatibility(promptId, {
          tool,
          model: model || null,
          status: "confirmed",
          notes: null,
        }),
        saveValidation(promptId, {
          status: effectiveness,
          rating: rating ? Number(rating) : null,
          notes: null,
        }),
      ]);
    } catch {
      setError(true);
    }
  }

  return (
    <details className="prompt-metadata-editor">
      <summary>编辑提示词信息</summary>
    <form aria-label="提示词元数据" onSubmit={submit}>
      <label>
        适用工具
        <input onChange={(event) => setTool(event.target.value)} required value={tool} />
      </label>
      <label>
        适用模型
        <input onChange={(event) => setModel(event.target.value)} value={model} />
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
      {error ? <p role="alert">无法保存元数据，请重试。</p> : null}
      <button type="submit">保存元数据</button>
    </form>
    </details>
  );
}
