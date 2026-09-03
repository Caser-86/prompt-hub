import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { RecoveryScreen } from "./RecoveryScreen";

describe("RecoveryScreen", () => {
  it("explains recovery, offers retry, and exports only a safe diagnostic summary", async () => {
    const retry = vi.fn().mockResolvedValue({
      state: "ready", code: null, safeMessage: null, backupName: null,
    });
    const exportDiagnostics = vi.fn().mockResolvedValue('{"state":"recovery"}');
    const listRecoveryBackups = vi.fn().mockResolvedValue([{
      path: "C:/data/prompt-hub.v8.pre-migration.200.bak", byteLen: 123, schemaVersion: 8,
    }]);
    const previewRecoveryBackup = vi.fn().mockResolvedValue({
      targetExists: true, backupSchemaVersion: 8, backupByteLen: 123, promptCount: 4,
    });
    const restoreRecoveryBackup = vi.fn().mockResolvedValue(undefined);

    render(<RecoveryScreen
      status={{
        state: "recovery",
        code: "migration_failed",
        safeMessage: "本地数据升级失败，原数据未被替换。",
        backupName: "prompt-hub.db.v6.pre-migration.bak",
      }}
      retry={retry}
      exportDiagnostics={exportDiagnostics}
      listRecoveryBackups={listRecoveryBackups}
      previewRecoveryBackup={previewRecoveryBackup}
      restoreRecoveryBackup={restoreRecoveryBackup}
      onRecovered={vi.fn()}
    />);

    expect(screen.getByRole("heading", { name: "需要恢复本地数据" })).toBeVisible();
    expect(screen.getByText("prompt-hub.db.v6.pre-migration.bak")).toBeVisible();
    expect(screen.getByText("原数据未被替换。你可以重试启动，或导出不含敏感内容的诊断摘要。")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "重试启动" }));
    await waitFor(() => expect(retry).toHaveBeenCalledOnce());
    fireEvent.click(screen.getByRole("button", { name: "导出诊断摘要" }));
    await waitFor(() => expect(exportDiagnostics).toHaveBeenCalledOnce());

    await waitFor(() => expect(listRecoveryBackups).toHaveBeenCalledOnce());
    fireEvent.click(screen.getByRole("button", { name: "检查恢复备份" }));
    await waitFor(() => expect(previewRecoveryBackup).toHaveBeenCalledWith("C:/data/prompt-hub.v8.pre-migration.200.bak"));
    fireEvent.click(screen.getByRole("button", { name: "恢复此备份" }));
    await waitFor(() => expect(restoreRecoveryBackup).toHaveBeenCalledWith("C:/data/prompt-hub.v8.pre-migration.200.bak"));
  });
});
