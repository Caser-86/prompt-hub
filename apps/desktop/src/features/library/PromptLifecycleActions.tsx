import { useState } from "react";

type PromptLifecycleActionsProps = {
  initialStatus: string;
  archive: () => Promise<unknown>;
  softDelete: () => Promise<unknown>;
  recover: () => Promise<unknown>;
};

export function PromptLifecycleActions({
  initialStatus,
  archive,
  softDelete,
  recover,
}: PromptLifecycleActionsProps) {
  const [status, setStatus] = useState(initialStatus);
  const [isDeleteConfirmationOpen, setDeleteConfirmationOpen] = useState(false);
  const [error, setError] = useState(false);

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
        <button onClick={() => void run(recover, "inbox")} type="button">恢复提示词</button>
      ) : (
        <>
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
      {error ? <p role="alert">操作未完成，请重试。</p> : null}
    </section>
  );
}
