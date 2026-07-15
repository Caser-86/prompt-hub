import { useState } from "react";

export type PromptHistoryItem = {
  number: number;
  body: string;
  createdAt: string;
};

type PromptHistoryProps = {
  history: PromptHistoryItem[];
  restoreVersion: (versionNumber: number) => Promise<unknown>;
};

export function PromptHistory({ history, restoreVersion }: PromptHistoryProps) {
  const [pendingRestore, setPendingRestore] = useState<number | null>(null);

  async function confirmRestore() {
    if (pendingRestore === null) {
      return;
    }
    await restoreVersion(pendingRestore);
    setPendingRestore(null);
  }

  return (
    <section aria-labelledby="history-title">
      <h2 id="history-title">版本历史</h2>
      <p>恢复不会覆盖历史版本，而是创建一个新的当前版本。</p>
      <ol>
        {history.map((version) => (
          <li key={version.number}>
            <h3>版本 {version.number}</h3>
            <time dateTime={version.createdAt}>{version.createdAt}</time>
            <pre>{version.body}</pre>
            <button onClick={() => setPendingRestore(version.number)} type="button">
              恢复版本 {version.number}
            </button>
          </li>
        ))}
      </ol>
      {pendingRestore !== null ? (
        <section aria-label="确认恢复" role="alertdialog">
          <p>将版本 {pendingRestore} 恢复为新的当前版本。</p>
          <button onClick={() => setPendingRestore(null)} type="button">取消</button>
          <button onClick={() => void confirmRestore()} type="button">确认恢复版本 {pendingRestore}</button>
        </section>
      ) : null}
    </section>
  );
}
