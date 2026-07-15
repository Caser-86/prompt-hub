import { useEffect, useState } from "react";

import type { PromptListItem } from "@prompt-hub/contracts";

import { filterAndSortPrompts, formatLibraryUpdatedAt, type PromptLibraryFilter } from "./libraryView";

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
  const [filter, setFilter] = useState<PromptLibraryFilter>("all");

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
  const visiblePrompts = prompts ? filterAndSortPrompts(prompts, filter) : [];

  return (
    <section aria-labelledby="library-title" className="prompt-library">
      <div className="feature-heading library-toolbar">
        <div>
          <p className="eyebrow">LOCAL LIBRARY</p>
          <h1 id="library-title">提示词库</h1>
        </div>
        <button className="button-primary" onClick={onCreate} type="button">创建提示词</button>
      </div>
      {prompts?.length === 0 ? (
        <section aria-label="空提示词库" className="empty-library-state surface-card">
          <span aria-hidden="true" className="empty-library-icon">＋</span>
          <h2>从第一条提示词开始</h2>
          <p>记录来源、适用模型和使用效果，让以后每一次搜索都有依据。</p>
          <button className="button-primary" onClick={onCreate} type="button">创建第一条提示词</button>
        </section>
      ) : null}
      {prompts?.length ? (
        <>
          <div aria-label="提示词库筛选" className="library-quick-filters">
            <div aria-label="筛选提示词" className="library-filter-options" role="group">
              {(["all", "favorite", "effective", "needs_retest"] as const).map((value) => (
                <button
                  aria-pressed={filter === value}
                  className="library-filter-button"
                  key={value}
                  onClick={() => setFilter(value)}
                  type="button"
                >
                  {filterLabel(value)}
                </button>
              ))}
            </div>
            <p aria-live="polite" className="library-result-count">共 {visiblePrompts.length} 条提示词</p>
          </div>
          <ul aria-label="提示词列表" className="prompt-list">
            {visiblePrompts.map((prompt) => (
              <li className="prompt-list-item surface-card" key={prompt.id}>
                <div className="prompt-list-primary">
                  <button aria-label={`打开提示词：${prompt.title}`} className="prompt-list-title" onClick={() => onSelect?.(prompt)} type="button">
                    <strong>{prompt.title}</strong>
                  </button>
                  {prompt.tags.length ? <div aria-label="标签" className="prompt-list-tags">{prompt.tags.map((tag) => <span key={tag}>{tag}</span>)}</div> : null}
                </div>
                <div className="prompt-list-meta">
                  {prompt.sourceNames.length ? <span>来源：{prompt.sourceNames.join("、")}</span> : null}
                  <span className={`status-pill status-${prompt.effectiveness}`}>{effectivenessLabel(prompt.effectiveness)}</span>
                  {prompt.applicableTools?.length ? <span>工具：{prompt.applicableTools.join("、")}</span> : null}
                  <time dateTime={prompt.updatedAt}>更新于 {formatLibraryUpdatedAt(prompt.updatedAt)}</time>
                </div>
                <div className="prompt-list-actions">
                <label><input aria-label={`选择提示词：${prompt.title}`} checked={selected.includes(prompt.id)} onChange={() => toggleSelected(prompt.id)} type="checkbox" /></label>
                <button
                  aria-label={`${prompt.favorite ? "取消收藏" : "收藏"}提示词：${prompt.title}`}
                  onClick={() => toggleFavorite(prompt)}
                  type="button"
                >{prompt.favorite ? "★" : "☆"}</button>
                </div>
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

function filterLabel(filter: PromptLibraryFilter) {
  return {
    all: "全部",
    favorite: "收藏",
    effective: "已验证",
    needs_retest: "待复测",
  }[filter];
}
