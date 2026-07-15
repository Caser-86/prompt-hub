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
  const [status, setStatus] = useState("");
  const [sourceKind, setSourceKind] = useState("");
  const [category, setCategory] = useState("");
  const [tagText, setTagText] = useState("");
  const [tool, setTool] = useState("");
  const [model, setModel] = useState("");
  const [updatedAfter, setUpdatedAfter] = useState("");
  const [updatedBefore, setUpdatedBefore] = useState("");
  const [offset, setOffset] = useState(0);
  const generation = useRef(0);

  useEffect(() => {
    const trimmedQuery = query.trim();
    const requestGeneration = generation.current;
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
  }, [category, effectiveness, minimumRating, model, offset, query, searchPrompts, sourceKind, status, tagText, tool, updatedAfter, updatedBefore]);

  const resetPage = () => setOffset(0);
  const saveView = () => window.localStorage.setItem("prompt-hub.search-view", JSON.stringify({ query, effectiveness, minimumRating, status, sourceKind, category, tagText, tool, model, updatedAfter, updatedBefore }));

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
      <label>生命周期<select onChange={(event) => { setStatus(event.target.value); resetPage(); }} value={status}>
        <option value="">全部</option><option value="inbox">收件箱</option><option value="published">已发布</option><option value="archived">已归档</option><option value="deleted">已删除</option>
      </select></label>
      <label>来源类型<select onChange={(event) => { setSourceKind(event.target.value); resetPage(); }} value={sourceKind}>
        <option value="">全部</option><option value="manual">手动录入</option><option value="file_import">文件导入</option><option value="web_url">网页链接</option><option value="ai_generated">AI 生成</option><option value="mcp">MCP</option>
      </select></label>
      <label>分类<input onChange={(event) => { setCategory(event.target.value); resetPage(); }} value={category} /></label>
      <label>标签<input onChange={(event) => { setTagText(event.target.value); resetPage(); }} placeholder="多个标签用逗号分隔" value={tagText} /></label>
      <label>适用工具<input onChange={(event) => { setTool(event.target.value); resetPage(); }} value={tool} /></label>
      <label>适用模型<input onChange={(event) => { setModel(event.target.value); resetPage(); }} value={model} /></label>
      <label>更新开始日期<input onChange={(event) => { setUpdatedAfter(event.target.value); resetPage(); }} type="date" value={updatedAfter} /></label>
      <label>更新结束日期<input onChange={(event) => { setUpdatedBefore(event.target.value); resetPage(); }} type="date" value={updatedBefore} /></label>
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
