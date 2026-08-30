import { useState } from "react";
import { ClipboardDocumentIcon, EllipsisHorizontalIcon } from "@heroicons/react/24/outline";

export function PromptContentActions({ body, metadataExport, onUsed, title }: { body: string; metadataExport?: string; onUsed?: () => void; title: string }) {
  const [copied, setCopied] = useState(false);
  const [failed, setFailed] = useState(false);

  async function copy() {
    setCopied(false);
    setFailed(false);
    try {
      await navigator.clipboard.writeText(body);
      onUsed?.();
      setCopied(true);
    } catch {
      setFailed(true);
    }
  }

  function exportMarkdown() {
    downloadMarkdown(`# ${title}\n\n${body}\n`, title);
  }

  function exportMarkdownWithMetadata() {
    if (!metadataExport) return;
    downloadMarkdown(`# ${title}\n\n${body}\n\n---\n\n## 来源元数据\n\n${metadataExport.trim()}\n`, title);
  }

  function downloadMarkdown(content: string, title: string) {
    const url = URL.createObjectURL(new Blob([content], { type: "text/markdown;charset=utf-8" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${safeFileName(title)}.md`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  return <section aria-label="提示词使用操作">
    <button aria-label="复制提示词正文" className="button-primary prompt-copy-action" onClick={() => { void copy(); }} type="button"><ClipboardDocumentIcon aria-hidden="true" /> 复制提示词</button>
    <details className="prompt-export-menu">
      <summary aria-label="更多提示词操作"><EllipsisHorizontalIcon aria-hidden="true" /></summary>
      <button onClick={exportMarkdown} type="button">导出 Markdown</button>
      {metadataExport ? <button onClick={exportMarkdownWithMetadata} type="button">导出 Markdown（含来源）</button> : null}
    </details>
    {copied ? <p role="status">已复制提示词正文。</p> : null}
    {failed ? <p role="alert">无法复制提示词正文，请手动复制。</p> : null}
  </section>;
}

function safeFileName(value: string) {
  return value.trim().replace(/[\\/:*?"<>|]/g, "-") || "prompt";
}
