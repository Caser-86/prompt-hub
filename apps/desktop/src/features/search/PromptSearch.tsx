import { useEffect, useRef, useState } from "react";

import type { PromptSearchPage } from "@prompt-hub/contracts";

type PromptSearchProps = {
  searchPrompts: (text: string, limit?: number, offset?: number) => Promise<PromptSearchPage>;
};

export function PromptSearch({ searchPrompts }: PromptSearchProps) {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState<PromptSearchPage | null>(null);
  const [isLoading, setLoading] = useState(false);
  const [hasError, setError] = useState(false);
  const generation = useRef(0);

  useEffect(() => {
    const trimmedQuery = query.trim();
    const requestGeneration = generation.current;
    if (!trimmedQuery) {
      setPage(null);
      setLoading(false);
      setError(false);
      return;
    }
    setLoading(true);
    setError(false);
    const timeout = window.setTimeout(() => {
      void searchPrompts(trimmedQuery, 20, 0)
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
  }, [query, searchPrompts]);

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
