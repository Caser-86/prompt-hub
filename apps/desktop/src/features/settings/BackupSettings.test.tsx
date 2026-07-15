import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BackupSettings } from "./BackupSettings";

describe("BackupSettings", () => {
  it("creates a backup and displays an integrity-checked confirmation", async () => {
    const createBackup = vi.fn().mockResolvedValue({ path: "C:/data/backups/manual.db", byteLen: 512, schemaVersion: 2 });
    render(<BackupSettings createBackup={createBackup} previewRestore={vi.fn()} restoreBackup={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "立即创建备份" }));

    await waitFor(() => expect(createBackup).toHaveBeenCalledOnce());
    expect(screen.getByText(/备份已完成并通过完整性校验/)).toBeInTheDocument();
  });

  it("shows a read-only restore preview before a destructive restore", async () => {
    const previewRestore = vi.fn().mockResolvedValue({ targetExists: true, backupSchemaVersion: 2, backupByteLen: 512 });
    render(<BackupSettings createBackup={vi.fn()} previewRestore={previewRestore} restoreBackup={vi.fn()} />);

    fireEvent.change(screen.getByLabelText("备份文件路径"), { target: { value: "C:/data/backup.db" } });
    fireEvent.click(screen.getByRole("button", { name: "检查恢复内容" }));

    await waitFor(() => expect(previewRestore).toHaveBeenCalledWith("C:/data/backup.db"));
    expect(screen.getByText(/恢复会替换现有数据库/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "确认恢复备份" })).toBeInTheDocument();
  });
});
