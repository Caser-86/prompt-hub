import { useState } from "react";
import { useEffect } from "react";
import type { BackupRestorePreview, BootstrapStatus, RecoveryBackupCandidate } from "@prompt-hub/contracts";
import "./recovery.css";

type RecoveryScreenProps = {
  status: BootstrapStatus;
  retry: () => Promise<BootstrapStatus>;
  exportDiagnostics: () => Promise<string>;
  listRecoveryBackups: () => Promise<RecoveryBackupCandidate[]>;
  previewRecoveryBackup: (path: string) => Promise<BackupRestorePreview>;
  restoreRecoveryBackup: (path: string) => Promise<void>;
  onRecovered: () => void;
};

export function RecoveryScreen({ status, retry, exportDiagnostics, listRecoveryBackups, previewRecoveryBackup, restoreRecoveryBackup, onRecovered }: RecoveryScreenProps) {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [backups, setBackups] = useState<RecoveryBackupCandidate[]>([]);
  const [path, setPath] = useState("");
  const [preview, setPreview] = useState<BackupRestorePreview | null>(null);

  useEffect(() => {
    void listRecoveryBackups().then((items) => {
      setBackups(items);
      if (items[0]) setPath(items[0].path);
    }).catch(() => setBackups([]));
  }, [listRecoveryBackups]);

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

  const handlePreview = async () => {
    const normalizedPath = path.trim();
    if (!normalizedPath) return;
    setBusy(true);
    setMessage(null);
    setPreview(null);
    try {
      setPreview(await previewRecoveryBackup(normalizedPath));
    } catch {
      setMessage("备份无法通过完整性校验，未执行恢复。");
    } finally {
      setBusy(false);
    }
  };

  const handleRestore = async () => {
    const normalizedPath = path.trim();
    if (!normalizedPath || !preview) return;
    setBusy(true);
    setMessage(null);
    try {
      await restoreRecoveryBackup(normalizedPath);
      const next = await retry();
      if (next.state === "ready") onRecovered();
      else setMessage(next.safeMessage ?? "恢复完成，但应用仍需要重试启动。");
    } catch {
      setMessage("恢复失败；原数据库和恢复前安全备份均已保留。");
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
      <section aria-labelledby="recovery-restore-title" className="recovery-restore">
        <h2 id="recovery-restore-title">从备份恢复</h2>
        <p>先检查备份内容，再替换当前数据库。恢复前会自动保留安全副本。</p>
        {backups.length ? <label>检测到的升级备份
          <select aria-label="检测到的升级备份" onChange={(event) => { setPath(event.target.value); setPreview(null); }} value={path}>
            {backups.map((backup) => <option key={backup.path} value={backup.path}>{backup.path} · 架构 {backup.schemaVersion}</option>)}
          </select>
        </label> : null}
        <label>备份文件路径
          <input aria-label="备份文件路径" onChange={(event) => { setPath(event.target.value); setPreview(null); }} placeholder="粘贴 .bak 文件的完整路径" value={path} />
        </label>
        <button disabled={busy || !path.trim()} onClick={() => { void handlePreview(); }} type="button">检查恢复备份</button>
        {preview ? <div className="recovery-preview" role="status"><p>备份通过校验，包含 {preview.promptCount} 条提示词，架构版本 {preview.backupSchemaVersion}。</p><button className="button-primary" disabled={busy} onClick={() => { void handleRestore(); }} type="button">恢复此备份</button></div> : null}
      </section>
      <details className="recovery-help">
        <summary>打开恢复说明</summary>
        <p>请保留当前数据目录和备份文件，不要手动删除数据库。若重试仍失败，请把诊断摘要提供给维护者。</p>
      </details>
    </section>
  );
}
