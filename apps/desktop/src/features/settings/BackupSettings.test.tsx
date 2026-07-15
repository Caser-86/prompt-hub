import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BackupSettings } from "./BackupSettings";

describe("BackupSettings", () => {
  it("creates a backup and displays an integrity-checked confirmation", async () => {
    const createBackup = vi.fn().mockResolvedValue({ path: "C:/data/backups/manual.db", byteLen: 512, schemaVersion: 2 });
    render(<BackupSettings createBackup={createBackup} previewRestore={vi.fn()} pruneBackups={vi.fn()} restoreBackup={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "立即创建备份" }));

    await waitFor(() => expect(createBackup).toHaveBeenCalledOnce());
    expect(screen.getByText(/备份已完成并通过完整性校验/)).toBeInTheDocument();
  });

  it("creates a backup in the directory selected by the user", async () => {
    const createBackup = vi.fn().mockResolvedValue({ path: "D:/backups/manual.db", byteLen: 512, schemaVersion: 2 });
    render(<BackupSettings createBackup={createBackup} previewRestore={vi.fn()} pruneBackups={vi.fn()} restoreBackup={vi.fn()} />);

    fireEvent.change(screen.getByLabelText("备份目录（可选）"), { target: { value: "D:/backups" } });
    fireEvent.click(screen.getByRole("button", { name: "立即创建备份" }));

    await waitFor(() => expect(createBackup).toHaveBeenCalledWith("D:/backups"));
  });

  it("shows a read-only restore preview before a destructive restore", async () => {
    const previewRestore = vi.fn().mockResolvedValue({ targetExists: true, backupSchemaVersion: 2, backupByteLen: 512, promptCount: 2 });
    render(<BackupSettings createBackup={vi.fn()} previewRestore={previewRestore} pruneBackups={vi.fn()} restoreBackup={vi.fn()} />);

    fireEvent.change(screen.getByLabelText("备份文件路径"), { target: { value: "C:/data/backup.db" } });
    fireEvent.click(screen.getByRole("button", { name: "检查恢复内容" }));

    await waitFor(() => expect(previewRestore).toHaveBeenCalledWith("C:/data/backup.db"));
    expect(screen.getByText(/恢复会替换现有数据库/)).toBeInTheDocument();
    expect(screen.getByText(/包含 2 条提示词/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "确认恢复备份" })).toBeInTheDocument();
  });

  it("keeps the user-selected number of application backups", async () => {
    const pruneBackups = vi.fn().mockResolvedValue(3);
    render(<BackupSettings createBackup={vi.fn()} previewRestore={vi.fn()} pruneBackups={pruneBackups} restoreBackup={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("保留最近备份数"), { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "清理旧备份" }));
    await waitFor(() => expect(pruneBackups).toHaveBeenCalledWith(5));
    expect(screen.getByText("已清理 3 个旧备份。")).toBeInTheDocument();
  });
});
