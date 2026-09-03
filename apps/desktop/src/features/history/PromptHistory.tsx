import { useState } from "react";

export type PromptHistoryItem = {
  number: number;
  body: string;
  createdAt: string;
  effectiveness?: string;
  sourceNames?: string[];
  applicableTools?: string[];
  rating?: number | null;
};

type PromptHistoryProps = {
  history: PromptHistoryItem[];
  restoreVersion: (versionNumber: number) => Promise<unknown>;
};

function lineDiff(before: string, after: string): string[] {
  const beforeLines = before.split("\n");
  const afterLines = after.split("\n");
  const table = Array.from({ length: beforeLines.length + 1 }, () =>
    Array<number>(afterLines.length + 1).fill(0),
  );

  for (let beforeIndex = beforeLines.length - 1; beforeIndex >= 0; beforeIndex -= 1) {
    for (let afterIndex = afterLines.length - 1; afterIndex >= 0; afterIndex -= 1) {
      table[beforeIndex][afterIndex] = beforeLines[beforeIndex] === afterLines[afterIndex]
        ? table[beforeIndex + 1][afterIndex + 1] + 1
        : Math.max(table[beforeIndex + 1][afterIndex], table[beforeIndex][afterIndex + 1]);
    }
  }

  const changes: string[] = [];
  let beforeIndex = 0;
  let afterIndex = 0;
  while (beforeIndex < beforeLines.length || afterIndex < afterLines.length) {
    if (beforeLines[beforeIndex] === afterLines[afterIndex]) {
      changes.push(`  ${beforeLines[beforeIndex]}`);
      beforeIndex += 1;
      afterIndex += 1;
    } else if (afterIndex === afterLines.length || (
      beforeIndex < beforeLines.length && table[beforeIndex + 1][afterIndex] >= table[beforeIndex][afterIndex + 1]
    )) {
      changes.push(`- ${beforeLines[beforeIndex]}`);
      beforeIndex += 1;
    } else {
      changes.push(`+ ${afterLines[afterIndex]}`);
      afterIndex += 1;
    }
  }
  return changes;
}

export function PromptHistory({ history, restoreVersion }: PromptHistoryProps) {
  const [pendingRestore, setPendingRestore] = useState<number | null>(null);
  const [restoreBusy, setRestoreBusy] = useState(false);
  const [restoreError, setRestoreError] = useState(false);
  const [baseVersionNumber, setBaseVersionNumber] = useState<number | null>(history[0]?.number ?? null);
  const currentVersion = history.at(-1);
  const baseVersion = history.find((version) => version.number === baseVersionNumber);

  async function confirmRestore() {
    if (pendingRestore === null) {
      return;
    }
    setRestoreBusy(true);
    setRestoreError(false);
    try {
      await restoreVersion(pendingRestore);
      setPendingRestore(null);
    } catch {
      setRestoreError(true);
    } finally {
      setRestoreBusy(false);
    }
  }

  return (
    <details aria-labelledby="history-title" className="prompt-history-disclosure">
      <summary><span aria-level={2} id="history-title" role="heading">版本历史</span></summary>
      <p>恢复不会覆盖历史版本，而是创建一个新的当前版本。</p>
      {baseVersion && currentVersion && baseVersion.number !== currentVersion.number ? (
        <section aria-labelledby="diff-title">
          <h3 id="diff-title">版本差异</h3>
          <label>
            对比基准
            <select
              aria-label="对比基准版本"
              onChange={(event) => setBaseVersionNumber(Number(event.target.value))}
              value={baseVersion.number}
            >
              {history.slice(0, -1).map((version) => (
                <option key={version.number} value={version.number}>版本 {version.number}</option>
              ))}
            </select>
          </label>
          <p>与当前版本 {currentVersion.number} 对比：以 “-” 标记删除、以 “+” 标记新增。</p>
          <pre aria-label="版本正文差异">{lineDiff(baseVersion.body, currentVersion.body).join("\n")}</pre>
        </section>
      ) : null}
      <ol>
        {history.map((version) => (
          <li key={version.number}>
            <h3>版本 {version.number}</h3>
            <time dateTime={version.createdAt}>{version.createdAt}</time>
            {version.effectiveness || version.sourceNames?.length || version.applicableTools?.length || version.rating ? (
              <p className="prompt-history-metadata">
                元数据快照：{effectivenessLabel(version.effectiveness)}{version.rating ? ` · ${version.rating}/5` : ""}
                {version.applicableTools?.length ? ` · 工具：${version.applicableTools.join("、")}` : ""}
                {version.sourceNames?.length ? ` · 来源：${version.sourceNames.join("、")}` : ""}
              </p>
            ) : null}
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
          <button disabled={restoreBusy} onClick={() => setPendingRestore(null)} type="button">取消</button>
          <button disabled={restoreBusy} onClick={() => void confirmRestore()} type="button">{restoreBusy ? "恢复中…" : `确认恢复版本 ${pendingRestore}`}</button>
          {restoreError ? <p role="alert">恢复失败，请重试。</p> : null}
        </section>
      ) : null}
    </details>
  );
}

function effectivenessLabel(status?: string) {
  return {
    unverified: "未验证",
    effective: "有效",
    ineffective: "失效",
    needs_retest: "待复测",
  }[status ?? ""] ?? "元数据未记录";
}
