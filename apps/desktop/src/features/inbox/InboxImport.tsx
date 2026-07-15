import { useEffect, useState } from "react";
import type { ImportResult, PromptListItem } from "@prompt-hub/contracts";

export function InboxImport({ importFile, loadPrompts, onReview }: { importFile: (path: string) => Promise<ImportResult>; loadPrompts: () => Promise<PromptListItem[]>; onReview: (prompt: PromptListItem) => void }) {
  const [path, setPath] = useState(""); const [result, setResult] = useState<ImportResult | null>(null); const [error, setError] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [drafts, setDrafts] = useState<PromptListItem[]>([]);
  useEffect(() => { void loadPrompts().then((prompts) => setDrafts(prompts.filter((prompt) => prompt.status === "inbox"))).catch(() => setError(true)); }, [loadPrompts, result]);
  const run = async () => { setError(false); setIsImporting(true); try { setResult(await importFile(path)); } catch { setError(true); } finally { setIsImporting(false); } };
  return <section aria-labelledby="inbox-title"><h1 id="inbox-title">收件箱</h1><p>导入内容会先进入收件箱，审核并补齐信息后再发布。</p><label>文件路径<input onChange={(event) => setPath(event.target.value)} value={path} /></label><button disabled={!path || isImporting} onClick={() => void run()} type="button">导入到收件箱</button>{result ? <p role="status">已创建 {result.imported} 条待审核草稿；跳过 {result.skippedDuplicates} 条完全重复内容。</p> : null}{drafts.length ? <ul aria-label="待审核草稿">{drafts.map((draft) => <li key={draft.id}><strong>{draft.title}</strong><button onClick={() => onReview(draft)} type="button">查看并审核</button></li>)}</ul> : <p>没有待审核草稿。</p>}{error ? <><p role="alert">无法导入文件。仅支持 Markdown、TXT、JSON 和 CSV。</p><button disabled={isImporting} onClick={() => void run()} type="button">重试导入</button></> : null}</section>;
}
