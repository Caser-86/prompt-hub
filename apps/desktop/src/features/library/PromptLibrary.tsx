import { useEffect, useState } from "react";

import type { PromptListItem } from "@prompt-hub/contracts";

type PromptLibraryProps = {
  loadPrompts: () => Promise<PromptListItem[]>;
  onCreate?: () => void;
  onSelect?: (prompt: PromptListItem) => void;
  onFavorite?: (prompt: PromptListItem, favorite: boolean) => Promise<void>;
  batchArchive?: (ids: string[]) => Promise<void>;
};

export function PromptLibrary({ loadPrompts, onCreate, onSelect, onFavorite, batchArchive }: PromptLibraryProps) {
  const [prompts, setPrompts] = useState<PromptListItem[] | null>(null);
  const [error, setError] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [confirmBatchArchive, setConfirmBatchArchive] = useState(false);

  useEffect(() => {
    void loadPrompts()
      .then(setPrompts)
      .catch(() => setError(true));
  }, [loadPrompts]);

  const toggleFavorite = (prompt: PromptListItem) => {
    if (!onFavorite) return;
    const favorite = !prompt.favorite;
    void onFavorite(prompt, favorite).then(() => {
      setPrompts((current) => current?.map((item) => item.id === prompt.id ? { ...item, favorite } : item) ?? null);
    });
  };

  const toggleSelected = (id: string) => setSelected((current) => current.includes(id) ? current.filter((value) => value !== id) : [...current, id]);
  const archiveSelected = () => {
    if (!batchArchive) return;
    void batchArchive(selected).then(() => {
      setPrompts((current) => current?.map((prompt) => selected.includes(prompt.id) ? { ...prompt, status: "archived" } : prompt) ?? null);
      setSelected([]);
      setConfirmBatchArchive(false);
    });
  };

  return (
    <section aria-labelledby="library-title" className="prompt-library">
      <div className="feature-heading">
        <div>
          <p className="eyebrow">LOCAL LIBRARY</p>
          <h1 id="library-title">提示词库</h1>
        </div>
        <button onClick={onCreate} type="button">创建提示词</button>
      </div>
      {prompts?.length === 0 ? <p>还没有提示词资产</p> : null}
      {prompts?.length ? (
        <><ul aria-label="提示词列表">
          {prompts.map((prompt) => (
            <li key={prompt.id}>
              <label><input aria-label={`选择提示词：${prompt.title}`} checked={selected.includes(prompt.id)} onChange={() => toggleSelected(prompt.id)} type="checkbox" /></label>
              <button aria-label={`打开提示词：${prompt.title}`} onClick={() => onSelect?.(prompt)} type="button">
                <strong>{prompt.title}</strong>
              </button>
              <button
                aria-label={`${prompt.favorite ? "取消收藏" : "收藏"}提示词：${prompt.title}`}
                onClick={() => toggleFavorite(prompt)}
                type="button"
              >{prompt.favorite ? "已收藏" : "收藏"}</button>
              <p>来源：{prompt.sourceNames.join("、") || "未记录"}</p>
              <p>{effectivenessLabel(prompt.effectiveness)}</p>
              <time dateTime={prompt.updatedAt}>更新于 {prompt.updatedAt}</time>
            </li>
          ))}
        </ul>
        {selected.length ? <div>
          <button onClick={() => setConfirmBatchArchive(true)} type="button">批量归档 {selected.length} 条提示词</button>
          {confirmBatchArchive ? <div role="dialog" aria-label="确认批量归档"><p>批量归档可在提示词详情中恢复。</p><button onClick={archiveSelected} type="button">确认归档</button><button onClick={() => setConfirmBatchArchive(false)} type="button">取消</button></div> : null}
        </div> : null}</>
      ) : null}
      {error ? <p role="alert">无法读取本地提示词库，请重试。</p> : null}
    </section>
  );
}

function effectivenessLabel(status: string) {
  return {
    unverified: "未验证",
    effective: "有效",
    ineffective: "失效",
    needs_retest: "待复测",
  }[status] ?? "未知";
}
