import { useState } from "react";

export type BackupInfo = { path: string; byteLen: number; schemaVersion: number };
export type RestorePreviewInfo = { targetExists: boolean; backupSchemaVersion: number; backupByteLen: number };

export function BackupSettings({
  createBackup,
  previewRestore,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
}) {
  const [backup, setBackup] = useState<BackupInfo | null>(null);
  const [path, setPath] = useState("");
  const [preview, setPreview] = useState<RestorePreviewInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  const makeBackup = async () => {
    setError(null);
    try { setBackup(await createBackup()); } catch { setError("无法创建备份，请检查数据目录和可用磁盘空间。"); }
  };
  const inspectBackup = async () => {
    setError(null);
    setPreview(null);
    try { setPreview(await previewRestore(path)); } catch { setError("备份无法通过完整性校验，未执行恢复。"); }
  };

  return <section aria-labelledby="backup-settings-title">
    <h2 id="backup-settings-title">备份与恢复</h2>
    <p>备份仅保存在本机。恢复前会先检查备份完整性并显示预览。</p>
    <button onClick={() => void makeBackup()} type="button">立即创建备份</button>
    {backup ? <p role="status">备份已完成并通过完整性校验：{backup.path}（架构版本 {backup.schemaVersion}）</p> : null}
    <label>备份文件路径<input onChange={(event) => setPath(event.target.value)} value={path} /></label>
    <button disabled={!path} onClick={() => void inspectBackup()} type="button">检查恢复内容</button>
    {preview ? <p role="status">恢复会替换现有数据库；备份架构版本 {preview.backupSchemaVersion}，大小 {preview.backupByteLen} 字节。</p> : null}
    {error ? <p role="alert">{error}</p> : null}
  </section>;
}
