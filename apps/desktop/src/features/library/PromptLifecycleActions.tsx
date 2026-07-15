import { useState } from "react";

type PromptLifecycleActionsProps = {
  initialStatus: string;
  publish: () => Promise<unknown>;
  archive: () => Promise<unknown>;
  softDelete: () => Promise<unknown>;
  recover: () => Promise<unknown>;
  promptTitle?: string;
  permanentlyDelete?: () => Promise<{ path: string }>;
  onPermanentlyDeleted?: () => void;
};

export function PromptLifecycleActions({
  initialStatus,
  publish,
  archive,
  softDelete,
  recover,
  promptTitle = "",
  permanentlyDelete,
  onPermanentlyDeleted,
}: PromptLifecycleActionsProps) {
  const [status, setStatus] = useState(initialStatus);
  const [isDeleteConfirmationOpen, setDeleteConfirmationOpen] = useState(false);
  const [error, setError] = useState(false);
  const [isPermanentConfirmationOpen, setPermanentConfirmationOpen] = useState(false);
  const [confirmationTitle, setConfirmationTitle] = useState("");
  const [backupPath, setBackupPath] = useState<string | null>(null);

  async function run(operation: () => Promise<unknown>, nextStatus: string) {
    setError(false);
    try {
      await operation();
      setStatus(nextStatus);
    } catch {
      setError(true);
    }
  }

  return (
    <section aria-label="生命周期操作">
      <h3>生命周期</h3>
      <p>当前状态：{status}</p>
      {status === "deleted" ? (
        <>
          <button onClick={() => void run(recover, "inbox")} type="button">恢复提示词</button>
          {permanentlyDelete ? <button onClick={() => setPermanentConfirmationOpen(true)} type="button">永久清除提示词</button> : null}
        </>
      ) : (
        <>
          {status === "inbox" ? <button onClick={() => void run(publish, "published")} type="button">发布提示词</button> : null}
          <button onClick={() => void run(archive, "archived")} type="button">归档提示词</button>
          <button onClick={() => setDeleteConfirmationOpen(true)} type="button">软删除提示词</button>
        </>
      )}
      {isDeleteConfirmationOpen ? (
        <section aria-label="确认软删除" role="alertdialog">
          <p>软删除后可从恢复入口找回该提示词。</p>
          <button onClick={() => setDeleteConfirmationOpen(false)} type="button">取消</button>
          <button
            onClick={() => {
              setDeleteConfirmationOpen(false);
              void run(softDelete, "deleted");
            }}
            type="button"
          >
            确认软删除
          </button>
        </section>
      ) : null}
      {isPermanentConfirmationOpen ? <section aria-label="确认永久清除" role="alertdialog">
        <p>永久清除不可恢复，系统会先创建本地安全备份。</p>
        <label>输入提示词标题以确认<input onChange={(event) => setConfirmationTitle(event.target.value)} value={confirmationTitle} /></label>
        <button onClick={() => setPermanentConfirmationOpen(false)} type="button">取消</button>
        <button disabled={confirmationTitle !== promptTitle} onClick={() => void permanentlyDelete?.().then((backup) => { setBackupPath(backup.path); setPermanentConfirmationOpen(false); onPermanentlyDeleted?.(); }).catch(() => setError(true))} type="button">确认永久清除</button>
      </section> : null}
      {backupPath ? <p role="status">已永久清除。安全备份：{backupPath}</p> : null}
      {error ? <p role="alert">操作未完成，请重试。</p> : null}
    </section>
  );
}
