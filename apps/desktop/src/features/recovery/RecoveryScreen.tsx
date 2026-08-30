import { useState } from "react";
import type { BootstrapStatus } from "@prompt-hub/contracts";
import "./recovery.css";

type RecoveryScreenProps = {
  status: BootstrapStatus;
  retry: () => Promise<BootstrapStatus>;
  exportDiagnostics: () => Promise<string>;
  onRecovered: () => void;
};

export function RecoveryScreen({ status, retry, exportDiagnostics, onRecovered }: RecoveryScreenProps) {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const handleRetry = async () => {
    setBusy(true);
    setMessage(null);
    try {
      const next = await retry();
      if (next.state === "ready") onRecovered();
      else setMessage(next.safeMessage ?? "仍需要恢复本地数据。");
    } catch {
      setMessage("重试未成功，请检查诊断信息后再试。");
    } finally {
      setBusy(false);
    }
  };

  const handleExport = async () => {
    setBusy(true);
    try {
      const diagnostics = await exportDiagnostics();
      const blob = new Blob([diagnostics], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "prompt-hub-recovery-diagnostics.json";
      link.click();
      URL.revokeObjectURL(url);
      setMessage("诊断摘要已导出，不包含提示词正文或密钥。");
    } catch {
      setMessage("无法导出诊断摘要，请重试。");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section aria-labelledby="recovery-title" className="recovery-screen surface-card">
      <div className="recovery-icon" aria-hidden="true">!</div>
      <p className="eyebrow">LOCAL DATA RECOVERY</p>
      <h1 id="recovery-title">需要恢复本地数据</h1>
      <p className="recovery-message">{status.safeMessage ?? "应用暂时无法打开本地数据。"}</p>
      <p className="recovery-safe-note">原数据未被替换。你可以重试启动，或导出不含敏感内容的诊断摘要。</p>
      {status.backupName ? <p className="recovery-backup">已创建升级前备份：<strong>{status.backupName}</strong></p> : null}
      {message ? <p aria-live="polite" className="recovery-feedback">{message}</p> : null}
      <div className="recovery-actions">
        <button className="button-primary" disabled={busy} onClick={() => { void handleRetry(); }} type="button">{busy ? "处理中…" : "重试启动"}</button>
        <button className="button-secondary" disabled={busy} onClick={() => { void handleExport(); }} type="button">导出诊断摘要</button>
      </div>
      <details className="recovery-help">
        <summary>打开恢复说明</summary>
        <p>请保留当前数据目录和备份文件，不要手动删除数据库。若重试仍失败，请把诊断摘要提供给维护者。</p>
      </details>
    </section>
  );
}
