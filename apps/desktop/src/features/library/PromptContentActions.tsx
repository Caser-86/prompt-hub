import { useState } from "react";

export function PromptContentActions({ body, title }: { body: string; title: string }) {
  const [copied, setCopied] = useState(false);
  const [failed, setFailed] = useState(false);

  async function copy() {
    setCopied(false);
    setFailed(false);
    try {
      await navigator.clipboard.writeText(body);
      setCopied(true);
    } catch {
      setFailed(true);
    }
  }

  function exportMarkdown() {
    const content = `# ${title}\n\n${body}\n`;
    const url = URL.createObjectURL(new Blob([content], { type: "text/markdown;charset=utf-8" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${safeFileName(title)}.md`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  return <section aria-label="提示词使用操作">
    <button onClick={() => { void copy(); }} type="button">复制提示词正文</button>
    <button onClick={exportMarkdown} type="button">导出 Markdown</button>
    {copied ? <p role="status">已复制提示词正文。</p> : null}
    {failed ? <p role="alert">无法复制提示词正文，请手动复制。</p> : null}
  </section>;
}

function safeFileName(value: string) {
  return value.trim().replace(/[\\/:*?"<>|]/g, "-") || "prompt";
}
