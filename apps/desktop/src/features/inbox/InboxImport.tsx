import { useState } from "react";
import type { ImportResult } from "@prompt-hub/contracts";

export function InboxImport({ importFile }: { importFile: (path: string) => Promise<ImportResult> }) {
  const [path, setPath] = useState(""); const [result, setResult] = useState<ImportResult | null>(null); const [error, setError] = useState(false);
  const run = async () => { setError(false); try { setResult(await importFile(path)); } catch { setError(true); } };
  return <section aria-labelledby="inbox-title"><h1 id="inbox-title">收件箱</h1><p>导入内容会先进入收件箱，审核并补齐信息后再发布。</p><label>文件路径<input onChange={(event) => setPath(event.target.value)} value={path} /></label><button disabled={!path} onClick={() => void run()} type="button">导入到收件箱</button>{result ? <p role="status">已创建 {result.imported} 条待审核草稿；跳过 {result.skippedDuplicates} 条完全重复内容。</p> : null}{error ? <p role="alert">无法导入文件。仅支持 Markdown、TXT、JSON 和 CSV。</p> : null}</section>;
}
