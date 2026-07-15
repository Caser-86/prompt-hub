import { useState } from "react";

export type BackupInfo = { path: string; byteLen: number; schemaVersion: number };
export type RestorePreviewInfo = { targetExists: boolean; backupSchemaVersion: number; backupByteLen: number; promptCount: number };

export function BackupSettings({
  createBackup,
  previewRestore,
  restoreBackup,
  pruneBackups,
}: {
  createBackup: (directory?: string) => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  pruneBackups: (retain: number) => Promise<number>;
}) {
  const [backup, setBackup] = useState<BackupInfo | null>(null);
  const [backupDirectory, setBackupDirectory] = useState("");
  const [path, setPath] = useState("");
  const [preview, setPreview] = useState<RestorePreviewInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [restored, setRestored] = useState<BackupInfo | null>(null);
  const [retain, setRetain] = useState("10");
  const [pruned, setPruned] = useState<number | null>(null);

  const makeBackup = async () => {
    setError(null);
    try { setBackup(await createBackup(backupDirectory.trim() || undefined)); } catch { setError("无法创建备份，请检查备份目录和可用磁盘空间。"); }
  };
  const inspectBackup = async () => {
    setError(null);
    setPreview(null);
    try { setPreview(await previewRestore(path)); } catch { setError("备份无法通过完整性校验，未执行恢复。"); }
  };
  const restore = async () => {
    setError(null);
    try { setRestored(await restoreBackup(path)); } catch { setError("恢复失败；当前数据库保持不变，检查恢复前安全备份后再试。"); }
  };
  const prune = async () => {
    setError(null);
    try { setPruned(await pruneBackups(Number(retain))); } catch { setError("无法清理备份，请检查备份目录权限。 "); }
  };

  return <section aria-labelledby="backup-settings-title" className="settings-card surface-card backup-settings">
    <h2 id="backup-settings-title">备份与恢复</h2>
    <p>备份仅保存在本机。恢复前会先检查备份完整性并显示预览。</p>
    <label>备份目录（可选）<input onChange={(event) => setBackupDirectory(event.target.value)} placeholder="留空则保存到本机默认备份目录" value={backupDirectory} /></label>
    <button onClick={() => void makeBackup()} type="button">立即创建备份</button>
    {backup ? <p role="status">备份已完成并通过完整性校验：{backup.path}（架构版本 {backup.schemaVersion}）</p> : null}
    <label>备份文件路径<input onChange={(event) => setPath(event.target.value)} value={path} /></label>
    <button disabled={!path} onClick={() => void inspectBackup()} type="button">检查恢复内容</button>
    {preview ? <><p role="status">恢复会替换现有数据库；备份包含 {preview.promptCount} 条提示词，架构版本 {preview.backupSchemaVersion}，大小 {preview.backupByteLen} 字节。</p><button onClick={() => void restore()} type="button">确认恢复备份</button></> : null}
    {restored ? <p role="status">恢复完成。恢复前安全备份：{restored.path}</p> : null}
    <label>保留最近备份数<input min="0" onChange={(event) => setRetain(event.target.value)} type="number" value={retain} /></label>
    <button disabled={!/^\d+$/.test(retain)} onClick={() => void prune()} type="button">清理旧备份</button>
    {pruned !== null ? <p role="status">已清理 {pruned} 个旧备份。</p> : null}
    {error ? <p role="alert">{error}</p> : null}
  </section>;
}
