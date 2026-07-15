import { useEffect, useState } from "react";

import type { ImportResult, PromptListItem } from "@prompt-hub/contracts";

type ImportKind = "file" | "folder" | "url";

type InboxImportProps = {
  importFile: (path: string) => Promise<ImportResult>;
  importFolder: (path: string) => Promise<ImportResult>;
  importUrl: (url: string) => Promise<ImportResult>;
  loadPrompts: () => Promise<PromptListItem[]>;
  onReview: (prompt: PromptListItem) => void;
};

export function InboxImport({ importFile, importFolder, importUrl, loadPrompts, onReview }: InboxImportProps) {
  const [filePath, setFilePath] = useState("");
  const [folderPath, setFolderPath] = useState("");
  const [url, setUrl] = useState("");
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [lastImportKind, setLastImportKind] = useState<ImportKind | null>(null);
  const [drafts, setDrafts] = useState<PromptListItem[]>([]);

  useEffect(() => {
    void loadPrompts()
      .then((prompts) => setDrafts(prompts.filter((prompt) => prompt.status === "inbox")))
      .catch(() => setError(true));
  }, [loadPrompts, result]);

  async function run(kind: ImportKind) {
    const path = kind === "file" ? filePath : kind === "folder" ? folderPath : url;
    setError(false);
    setLastImportKind(kind);
    setIsImporting(true);
    try {
      setResult(await (kind === "file" ? importFile(path) : kind === "folder" ? importFolder(path) : importUrl(path)));
    } catch {
      setError(true);
    } finally {
      setIsImporting(false);
    }
  }

  return (
    <section aria-labelledby="inbox-title">
      <h1 id="inbox-title">收件箱</h1>
      <p>导入内容会先进入收件箱，审核并补齐信息后再发布。</p>
      <label>
        文件路径
        <input onChange={(event) => setFilePath(event.target.value)} value={filePath} />
      </label>
      <button disabled={!filePath || isImporting} onClick={() => void run("file")} type="button">
        导入到收件箱
      </button>
      <label>
        文件夹路径
        <input onChange={(event) => setFolderPath(event.target.value)} value={folderPath} />
      </label>
      <button disabled={!folderPath || isImporting} onClick={() => void run("folder")} type="button">
        扫描文件夹到收件箱
      </button>
      <label>
        网页 URL
        <input onChange={(event) => setUrl(event.target.value)} type="url" value={url} />
      </label>
      <button disabled={!url || isImporting} onClick={() => void run("url")} type="button">
        导入网页到收件箱
      </button>
      {result ? (
        <p role="status">
          已创建 {result.imported} 条待审核草稿；跳过 {result.skippedDuplicates} 条完全重复内容。
          {result.failed > 0 ? ` 有 ${result.failed} 条未导入，请查看诊断后重试。` : ""}
        </p>
      ) : null}
      {drafts.length ? (
        <ul aria-label="待审核草稿">
          {drafts.map((draft) => (
            <li key={draft.id}>
              <strong>{draft.title}</strong>
              <button onClick={() => onReview(draft)} type="button">查看并审核</button>
            </li>
          ))}
        </ul>
      ) : <p>没有待审核草稿。</p>}
      {error ? (
        <>
          <p role="alert">无法导入内容。仅支持 Markdown、TXT、JSON 和 CSV。</p>
          {lastImportKind ? (
            <button disabled={isImporting} onClick={() => void run(lastImportKind)} type="button">重试导入</button>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
