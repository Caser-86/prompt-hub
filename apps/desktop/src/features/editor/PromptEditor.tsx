import { useState } from "react";

import type { ManualPromptDraft } from "@prompt-hub/contracts";

type PromptEditorProps = {
  saveDraft: (draft: ManualPromptDraft) => Promise<unknown>;
  onSaved?: () => void;
};

export function PromptEditor({ saveDraft, onSaved }: PromptEditorProps) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [category, setCategory] = useState("");

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await saveDraft({
      title,
      body,
      description: null,
      category: category || null,
      tags: [],
    });
    onSaved?.();
  }

  return (
    <form aria-label="提示词编辑器" onSubmit={submit}>
      <label>
        标题
        <input onChange={(event) => setTitle(event.target.value)} required value={title} />
      </label>
      <label>
        正文
        <textarea onChange={(event) => setBody(event.target.value)} required value={body} />
      </label>
      <label>
        分类
        <input onChange={(event) => setCategory(event.target.value)} value={category} />
      </label>
      <button type="submit">保存到收件箱</button>
    </form>
  );
}
