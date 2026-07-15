import { useState } from "react";

import type { ManualPromptDraft, PromptVariableDraft } from "@prompt-hub/contracts";

type PromptEditorProps = {
  saveDraft: (draft: ManualPromptDraft) => Promise<unknown>;
  onSaved?: () => void;
};

export function PromptEditor({ saveDraft, onSaved }: PromptEditorProps) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [category, setCategory] = useState("");
  const [variables, setVariables] = useState<PromptVariableDraft[]>([]);

  function addVariable() {
    setVariables((current) => [
      ...current,
      { name: "", kind: "text", description: null, defaultValue: null, required: false },
    ]);
  }

  function updateVariable(index: number, update: Partial<PromptVariableDraft>) {
    setVariables((current) => current.map((variable, currentIndex) => (
      currentIndex === index ? { ...variable, ...update } : variable
    )));
  }

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await saveDraft({
      title,
      body,
      description: null,
      category: category || null,
      tags: [],
      variables,
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
      <fieldset>
        <legend>变量</legend>
        {variables.map((variable, index) => (
          <div key={index}>
            <label>
              变量名称
              <input
                onChange={(event) => updateVariable(index, { name: event.target.value })}
                value={variable.name}
              />
            </label>
            <label>
              变量类型
              <select
                onChange={(event) => updateVariable(index, {
                  kind: event.target.value as PromptVariableDraft["kind"],
                })}
                value={variable.kind}
              >
                <option value="text">文本</option>
                <option value="number">数字</option>
                <option value="boolean">布尔值</option>
              </select>
            </label>
            <label>
              变量默认值
              <input
                onChange={(event) => updateVariable(index, { defaultValue: event.target.value || null })}
                value={variable.defaultValue ?? ""}
              />
            </label>
          </div>
        ))}
        <button onClick={addVariable} type="button">添加变量</button>
      </fieldset>
      <button type="submit">保存到收件箱</button>
    </form>
  );
}
