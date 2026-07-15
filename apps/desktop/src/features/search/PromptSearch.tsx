import { useEffect, useRef, useState } from "react";

import type { PromptSearchFilters, PromptSearchPage } from "@prompt-hub/contracts";

type PromptSearchProps = {
  searchPrompts: (
    text: string,
    limit?: number,
    offset?: number,
    filters?: PromptSearchFilters,
  ) => Promise<PromptSearchPage>;
};

export function PromptSearch({ searchPrompts }: PromptSearchProps) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState<PromptSearchPage | null>(null);
  const [isLoading, setLoading] = useState(false);
  const [hasError, setError] = useState(false);
  const [effectiveness, setEffectiveness] = useState("");
  const [minimumRating, setMinimumRating] = useState("");
  const [offset, setOffset] = useState(0);
  const generation = useRef(0);

  useEffect(() => {
    const trimmedQuery = query.trim();
    const requestGeneration = generation.current;
    const filters: PromptSearchFilters = {
      ...(effectiveness ? { effectiveness: effectiveness as PromptSearchFilters["effectiveness"] } : {}),
      ...(minimumRating ? { minimumRating: Number(minimumRating) } : {}),
    };
    if (!trimmedQuery) {
      setPage(null);
      setLoading(false);
      setError(false);
      return;
    }
    setLoading(true);
    setError(false);
    const timeout = window.setTimeout(() => {
      void (Object.keys(filters).length === 0
        ? searchPrompts(trimmedQuery, 20, offset)
        : searchPrompts(trimmedQuery, 20, offset, filters))
        .then((result) => {
          if (generation.current === requestGeneration) {
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
  }, [effectiveness, minimumRating, offset, query, searchPrompts]);

  const resetPage = () => setOffset(0);
  const saveView = () => window.localStorage.setItem("prompt-hub.search-view", JSON.stringify({ query, effectiveness, minimumRating }));

  return (
    <section aria-labelledby="search-title">
      <h1 id="search-title">搜索提示词</h1>
      <label>
        搜索提示词
        <input
          onChange={(event) => {
            generation.current += 1;
            setQuery(event.target.value);
            resetPage();
          }}
          role="searchbox"
          value={query}
        />
      </label>
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
      <button onClick={saveView} type="button">保存当前视图</button>
      {isLoading ? <p role="status">正在搜索本地提示词库…</p> : null}
      {hasError ? <p role="alert">搜索失败，请重试。</p> : null}
      {page?.total === 0 ? <p>没有匹配的提示词。</p> : null}
      {page?.hits.length ? (
        <ol aria-label="搜索结果">
          {page.hits.map((hit) => (
            <li key={hit.id}>
              <h2>{hit.title}</h2>
              <p>{hit.snippet}</p>
              <p>{hit.effectiveness} · {hit.rating ?? "未评分"}</p>
              <time dateTime={hit.updatedAt}>{hit.updatedAt}</time>
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
