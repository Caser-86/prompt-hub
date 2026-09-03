import { useEffect, useRef, useState } from "react";

import type { PromptSearchFilters, PromptSearchPage, PromptSearchSort } from "@prompt-hub/contracts";
import "./search.css";

type PromptSearchProps = {
  searchPrompts: (
    text: string,
    limit?: number,
    offset?: number,
    filters?: PromptSearchFilters,
    sort?: PromptSearchSort,
  ) => Promise<PromptSearchPage>;
  onSelectPrompt?: (id: string) => void | Promise<void>;
};

export function PromptSearch({ onSelectPrompt, searchPrompts }: PromptSearchProps) {
  const savedView = readSavedView();
  const [query, setQuery] = useState(savedView.query);
  const [page, setPage] = useState<PromptSearchPage | null>(null);
  const [isLoading, setLoading] = useState(false);
  const [hasError, setError] = useState(false);
  const [effectiveness, setEffectiveness] = useState(savedView.effectiveness);
  const [minimumRating, setMinimumRating] = useState(savedView.minimumRating);
  const [status, setStatus] = useState(savedView.status);
  const [sourceKind, setSourceKind] = useState(savedView.sourceKind);
  const [category, setCategory] = useState(savedView.category);
  const [tagText, setTagText] = useState(savedView.tagText);
  const [tool, setTool] = useState(savedView.tool);
  const [model, setModel] = useState(savedView.model);
  const [updatedAfter, setUpdatedAfter] = useState(savedView.updatedAfter);
  const [updatedBefore, setUpdatedBefore] = useState(savedView.updatedBefore);
  const [favoritesOnly, setFavoritesOnly] = useState(savedView.favoritesOnly);
  const [sort, setSort] = useState<PromptSearchSort>(savedView.sort);
  const [offset, setOffset] = useState(0);
  const generation = useRef(0);

  useEffect(() => {
    const trimmedQuery = query.trim();
    const requestGeneration = ++generation.current;
    const filters: PromptSearchFilters = {
      ...(effectiveness ? { effectiveness: effectiveness as PromptSearchFilters["effectiveness"] } : {}),
      ...(minimumRating ? { minimumRating: Number(minimumRating) } : {}),
      ...(status ? { status: status as PromptSearchFilters["status"] } : {}),
      ...(sourceKind ? { sourceKind: sourceKind as PromptSearchFilters["sourceKind"] } : {}),
      ...(category ? { category } : {}),
      ...(tagText ? { tags: tagText.split(",").map((tag) => tag.trim()).filter(Boolean) } : {}),
      ...(tool ? { tool } : {}),
      ...(model ? { model } : {}),
      ...(updatedAfter ? { updatedAfter: `${updatedAfter}T00:00:00Z` } : {}),
      ...(updatedBefore ? { updatedBefore: `${updatedBefore}T23:59:59Z` } : {}),
      ...(favoritesOnly ? { favorite: true } : {}),
    };
    const hasFilters = Object.keys(filters).length > 0;
    if (!trimmedQuery && !hasFilters) {
      setPage(null);
      setLoading(false);
      setError(false);
      return;
    }
    setLoading(true);
    setError(false);
    const timeout = window.setTimeout(() => {
      void (sort === "relevance"
        ? (hasFilters ? searchPrompts(trimmedQuery, 20, offset, filters) : searchPrompts(trimmedQuery, 20, offset))
        : searchPrompts(trimmedQuery, 20, offset, hasFilters ? filters : undefined, sort))
        .then((result) => {
          if (generation.current === requestGeneration) {
            if (result.hits.length === 0 && result.total > 0 && offset >= result.total) {
              setOffset(Math.floor((result.total - 1) / 20) * 20);
              return;
            }
            setPage(result);
          }
        })
        .catch(() => {
          if (generation.current === requestGeneration) {
            setError(true);
          }
        })
        .finally(() => {
          if (generation.current === requestGeneration) {
            setLoading(false);
          }
        });
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [category, effectiveness, favoritesOnly, minimumRating, model, offset, query, searchPrompts, sort, sourceKind, status, tagText, tool, updatedAfter, updatedBefore]);

  const resetPage = () => setOffset(0);
  const saveView = () => window.localStorage.setItem("prompt-hub.search-view", JSON.stringify({ query, effectiveness, minimumRating, status, sourceKind, category, tagText, tool, model, updatedAfter, updatedBefore, favoritesOnly, sort }));
  const selectPrompt = async (id: string) => {
    try {
      await onSelectPrompt?.(id);
    } catch {
      setError(true);
    }
  };

  return (
    <section aria-labelledby="search-title">
      <header className="page-header">
        <p className="eyebrow">LIBRARY FILTERS</p>
        <h1 id="search-title">高级筛选</h1>
        <p>在提示词库内按来源、状态、工具、模型与时间精确筛选。</p>
      </header>
      <div className="filter-panel surface-card">
        <label className="search-query-field">
          搜索提示词
          <input
            onChange={(event) => {
              setQuery(event.target.value);
              resetPage();
            }}
            role="searchbox"
            value={query}
          />
        </label>
        <fieldset aria-label="常用筛选" className="primary-search-filters">
          <legend>常用筛选</legend>
          <label>分类<input onChange={(event) => { setCategory(event.target.value); resetPage(); }} value={category} /></label>
          <label>适用工具<input onChange={(event) => { setTool(event.target.value); resetPage(); }} value={tool} /></label>
          <label>适用模型<input onChange={(event) => { setModel(event.target.value); resetPage(); }} value={model} /></label>
          <label>排序方式<select onChange={(event) => { setSort(event.target.value as PromptSearchSort); resetPage(); }} value={sort}>
            <option value="relevance">相关度</option><option value="updated_at">最近更新</option><option value="rating">最高评分</option>
          </select></label>
        </fieldset>
        <details className="advanced-search-filters">
          <summary>更多筛选</summary>
          <div>
      <label>生命周期<select onChange={(event) => { setStatus(event.target.value); resetPage(); }} value={status}>
        <option value="">全部</option><option value="inbox">收件箱</option><option value="published">已发布</option><option value="archived">已归档</option><option value="deleted">已删除</option>
      </select></label>
      <label>来源类型<select onChange={(event) => { setSourceKind(event.target.value); resetPage(); }} value={sourceKind}>
        <option value="">全部</option><option value="manual">手动录入</option><option value="file_import">文件导入</option><option value="web_url">网页链接</option><option value="ai_generated">AI 生成</option><option value="mcp">MCP</option>
      </select></label>
      <label>标签<input onChange={(event) => { setTagText(event.target.value); resetPage(); }} placeholder="多个标签用逗号分隔" value={tagText} /></label>
      <label>更新开始日期<input onChange={(event) => { setUpdatedAfter(event.target.value); resetPage(); }} type="date" value={updatedAfter} /></label>
      <label>更新结束日期<input onChange={(event) => { setUpdatedBefore(event.target.value); resetPage(); }} type="date" value={updatedBefore} /></label>
      <label><input checked={favoritesOnly} onChange={(event) => { setFavoritesOnly(event.target.checked); resetPage(); }} type="checkbox" />仅看收藏</label>
      <label>
        有效性筛选
        <select onChange={(event) => { setEffectiveness(event.target.value); resetPage(); }} value={effectiveness}>
          <option value="">全部</option>
          <option value="unverified">未验证</option>
          <option value="effective">有效</option>
          <option value="ineffective">失效</option>
          <option value="needs_retest">待复测</option>
        </select>
      </label>
      <label>
        最低评分
        <select onChange={(event) => { setMinimumRating(event.target.value); resetPage(); }} value={minimumRating}>
          <option value="">不限</option>
          <option value="1">1</option>
          <option value="2">2</option>
          <option value="3">3</option>
          <option value="4">4</option>
          <option value="5">5</option>
        </select>
      </label>
          </div>
        </details>
        <button className="button-secondary save-search-view" onClick={saveView} type="button">保存当前视图</button>
      </div>
      {isLoading ? <p role="status">正在搜索本地提示词库…</p> : null}
      {hasError ? <p role="alert">搜索失败，请重试。</p> : null}
      {page?.total === 0 ? <p>没有匹配的提示词。</p> : null}
      {page?.hits.length ? (
        <ol aria-label="搜索结果" className="search-results">
          {page.hits.map((hit) => (
            <li key={hit.id}>
              <h2><button aria-label={`打开提示词：${hit.title}`} className="search-result-title" onClick={() => { void selectPrompt(hit.id); }} type="button">{hit.title}</button></h2>
              <p>{hit.snippet}</p>
              <p>{hit.sourceNames?.length ? `来源：${hit.sourceNames.join("、")} · ` : ""}{hit.applicableTools?.length ? `工具：${hit.applicableTools.join("、")} · ` : ""}{hit.applicableModels?.length ? `模型：${hit.applicableModels.join("、")} · ` : ""}{effectivenessLabel(hit.effectiveness)} · {hit.rating ?? "未评分"}</p>
              <time dateTime={hit.updatedAt}>{formatSearchUpdatedAt(hit.updatedAt)}</time>
            </li>
          ))}
        </ol>
      ) : null}
      {page && page.total > 20 ? <nav aria-label="搜索结果分页">
        <button disabled={offset === 0} onClick={() => setOffset((current) => Math.max(0, current - 20))} type="button">上一页</button>
        <span>第 {Math.floor(offset / 20) + 1} 页</span>
        <button disabled={offset + 20 >= page.total} onClick={() => setOffset((current) => current + 20)} type="button">下一页</button>
      </nav> : null}
    </section>
  );
}

function effectivenessLabel(status: string) {
  return { unverified: "未验证", effective: "有效", ineffective: "失效", needs_retest: "待复测" }[status] ?? "未知";
}

function formatSearchUpdatedAt(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "更新时间未知" : `更新于 ${date.toLocaleString("zh-CN", { hour12: false })}`;
}

type SavedSearchView = {
  query: string; effectiveness: string; minimumRating: string; status: string; sourceKind: string;
  category: string; tagText: string; tool: string; model: string; updatedAfter: string;
  updatedBefore: string; favoritesOnly: boolean; sort: PromptSearchSort;
};

function readSavedView(): SavedSearchView {
  const defaults: SavedSearchView = { query: "", effectiveness: "", minimumRating: "", status: "", sourceKind: "", category: "", tagText: "", tool: "", model: "", updatedAfter: "", updatedBefore: "", favoritesOnly: false, sort: "relevance" };
  try {
    const value: unknown = JSON.parse(window.localStorage.getItem("prompt-hub.search-view") ?? "{}");
    if (typeof value !== "object" || value === null) return defaults;
    const view = value as Record<string, unknown>;
    const text = (key: keyof SavedSearchView) => typeof view[key] === "string" ? view[key] : defaults[key] as string;
    const oneOf = <T extends string>(key: keyof SavedSearchView, allowed: readonly T[]): string => {
      const candidate = view[key];
      return typeof candidate === "string" && allowed.includes(candidate as T) ? candidate : defaults[key] as string;
    };
    const date = (key: "updatedAfter" | "updatedBefore") => {
      const candidate = view[key];
      return typeof candidate === "string" && isValidDateInput(candidate) ? candidate : defaults[key];
    };
    const sort = view.sort === "updated_at" || view.sort === "rating" || view.sort === "relevance" ? view.sort : defaults.sort;
    return {
      query: text("query"),
      effectiveness: oneOf("effectiveness", ["", "unverified", "effective", "ineffective", "needs_retest"] as const),
      minimumRating: oneOf("minimumRating", ["", "1", "2", "3", "4", "5"] as const),
      status: oneOf("status", ["", "inbox", "published", "archived", "deleted"] as const),
      sourceKind: oneOf("sourceKind", ["", "manual", "file_import", "web_url", "ai_generated", "mcp"] as const),
      category: text("category"), tagText: text("tagText"), tool: text("tool"), model: text("model"),
      updatedAfter: date("updatedAfter"), updatedBefore: date("updatedBefore"),
      favoritesOnly: view.favoritesOnly === true, sort,
    };
  } catch {
    return defaults;
  }
}

function isValidDateInput(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const date = new Date(`${value}T00:00:00Z`);
  if (Number.isNaN(date.getTime())) return false;
  return date.toISOString().slice(0, 10) === value;
}
