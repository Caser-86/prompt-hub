import { useState } from "react";

import type { ManualPromptDraft, PromptVariableDraft } from "@prompt-hub/contracts";
import { PromptContentActions } from "../library/PromptContentActions";

type PromptEditorProps = {
  saveDraft: (draft: ManualPromptDraft) => Promise<unknown>;
  onSaved?: () => void;
  initial?: ManualPromptDraft;
  submitLabel?: string;
  showPreviewActions?: boolean;
};

export function PromptEditor({
  initial,
  onSaved,
  saveDraft,
  showPreviewActions = true,
  submitLabel = "保存到收件箱",
}: PromptEditorProps) {
  const [title, setTitle] = useState(initial?.title ?? "");
  const [body, setBody] = useState(initial?.body ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [category, setCategory] = useState(initial?.category ?? "");
  const [tagText, setTagText] = useState(initial?.tags.join(", ") ?? "");
  const [variables, setVariables] = useState<PromptVariableDraft[]>(initial?.variables ?? []);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setSaving] = useState(false);

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

  const preview = renderPreview(body, variables);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    const names = variables.map((variable) => variable.name.trim());
    if (names.some((name) => !name)) {
      setError("变量名称不能为空");
      return;
    }
    if (new Set(names).size !== names.length) {
      setError("变量名称不能重复");
      return;
    }
    setSaving(true);
    try {
      await saveDraft({
        title,
        body,
        description: description.trim() || null,
        category: category.trim() || null,
        tags: tagText.split(",").map((tag) => tag.trim()).filter(Boolean),
        variables: variables.map((variable, index) => ({ ...variable, name: names[index] })),
      });
    } catch {
      setError("无法保存提示词，请重试。");
      setSaving(false);
      return;
    }
    try {
      await onSaved?.();
    } catch {
      setError("提示词已保存，但界面刷新失败，请重新打开。");
    } finally {
      setSaving(false);
    }
  }

  return (
    <form aria-label="提示词编辑器" className="prompt-editor" onSubmit={submit}>
      <label className="editor-title-field">
        标题
        <input onChange={(event) => setTitle(event.target.value)} required value={title} />
      </label>
      <label className="editor-body-field">
        正文
        <textarea onChange={(event) => setBody(event.target.value)} required value={body} />
      </label>
      <label className="editor-description-field">
        说明
        <input onChange={(event) => setDescription(event.target.value)} value={description} />
      </label>
      <label className="editor-category-field">
        分类
        <input onChange={(event) => setCategory(event.target.value)} value={category} />
      </label>
      <label className="editor-tags-field">
        标签
        <input aria-label="标签" onChange={(event) => setTagText(event.target.value)} placeholder="多个标签用逗号分隔" value={tagText} />
      </label>
      <fieldset className="variable-editor">
        <legend>变量</legend>
        {variables.map((variable, index) => (
          <div className="variable-row" key={index}>
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
            <label>
              <input
                checked={variable.required}
                onChange={(event) => updateVariable(index, { required: event.target.checked })}
                type="checkbox"
              />
              变量必填
            </label>
            <button aria-label={`删除变量 ${variable.name || index + 1}`} onClick={() => setVariables((current) => current.filter((_, currentIndex) => currentIndex !== index))} type="button">
              删除变量
            </button>
          </div>
        ))}
        <button onClick={addVariable} type="button">添加变量</button>
      </fieldset>
      <section aria-label="渲染预览" className="prompt-preview surface-card">
        <h2>渲染预览</h2>
        {preview.missingRequired.length > 0 ? (
          <p role="alert">缺少必填变量：{preview.missingRequired.join("、")}</p>
        ) : null}
        {preview.unresolved.length > 0 ? (
          <p>未替换变量：{preview.unresolved.join("、")}</p>
        ) : null}
        <pre>{preview.body}</pre>
        {showPreviewActions ? <PromptContentActions body={preview.body} title={title} /> : null}
      </section>
      {error ? <p role="alert">{error}</p> : null}
      <button className="button-primary" disabled={isSaving} type="submit">{isSaving ? "保存中…" : submitLabel}</button>
    </form>
  );
}

function renderPreview(body: string, variables: PromptVariableDraft[]) {
  const variablesByName = new Map(
    variables.filter((variable) => variable.name.trim()).map((variable) => [variable.name.trim(), variable]),
  );
  const unresolved = new Set<string>();
  const missingRequired = new Set<string>();
  const rendered = body.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (placeholder, rawName: string) => {
    const name = rawName.trim();
    const variable = variablesByName.get(name);
    if (!variable || !variable.defaultValue?.trim()) {
      unresolved.add(name);
      if (variable?.required) {
        missingRequired.add(name);
      }
      return placeholder;
    }
    return variable.defaultValue;
  });

  return {
    body: rendered,
    missingRequired: [...missingRequired],
    unresolved: [...unresolved],
  };
}
