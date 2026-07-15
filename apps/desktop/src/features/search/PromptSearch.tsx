import { useEffect, useRef, useState } from "react";

import type { PromptSearchFilters, PromptSearchPage } from "@prompt-hub/contracts";

type PromptSearchProps = {
  searchPrompts: (text: string, limit?: number, offset?: number) => Promise<PromptSearchPage>;
};

export function PromptSearch({ searchPrompts }: PromptSearchProps) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState<PromptSearchPage | null>(null);
  const [isLoading, setLoading] = useState(false);
  const [hasError, setError] = useState(false);
  const [effectiveness, setEffectiveness] = useState("");
  const [minimumRating, setMinimumRating] = useState("");
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
        ? searchPrompts(trimmedQuery, 20, 0)
        : searchPrompts(trimmedQuery, 20, 0, filters))
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
  }, [effectiveness, minimumRating, query, searchPrompts]);

  return (
    <section aria-labelledby="search-title">
      <h1 id="search-title">搜索提示词</h1>
      <label>
        搜索提示词
        <input
          onChange={(event) => {
            generation.current += 1;
            setQuery(event.target.value);
          }}
          role="searchbox"
          value={query}
        />
      </label>
      <label>
        有效性筛选
        <select onChange={(event) => setEffectiveness(event.target.value)} value={effectiveness}>
          <option value="">全部</option>
          <option value="unverified">未验证</option>
          <option value="effective">有效</option>
          <option value="ineffective">失效</option>
          <option value="needs_retest">待复测</option>
        </select>
      </label>
      <label>
        最低评分
        <select onChange={(event) => setMinimumRating(event.target.value)} value={minimumRating}>
          <option value="">不限</option>
          <option value="1">1</option>
          <option value="2">2</option>
          <option value="3">3</option>
          <option value="4">4</option>
          <option value="5">5</option>
        </select>
      </label>
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
    </section>
  );
}
