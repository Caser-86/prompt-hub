import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const desktopMock = vi.hoisted(() => ({
  createManualPromptDraft: vi.fn(),
  publishPrompt: vi.fn(),
  importFileToInbox: vi.fn(),
  importFolderToInbox: vi.fn(),
  recentImportJobs: vi.fn(),
  listPrompts: vi.fn(),
  promptHistory: vi.fn(),
  restorePromptVersion: vi.fn(),
  recordPromptCompatibility: vi.fn(),
  recordPromptValidation: vi.fn(),
  archivePrompt: vi.fn(),
  batchArchivePrompts: vi.fn(),
  softDeletePrompt: vi.fn(),
  recoverPrompt: vi.fn(),
  setPromptFavorite: vi.fn(),
  searchPrompts: vi.fn(),
  createManualBackup: vi.fn(),
  previewBackupRestore: vi.fn(),
  restoreBackup: vi.fn(),
  getAiCredentialStatus: vi.fn(),
  saveAiCredential: vi.fn(),
  getApplicationStatus: vi.fn(),
}));
vi.mock("./services/desktop", () => ({
  desktopCommands: desktopMock,
}));

import { App } from "./App";

describe("App", () => {
  beforeEach(() => {
    desktopMock.listPrompts.mockImplementation(() => new Promise<never[]>(() => undefined));
    desktopMock.promptHistory.mockResolvedValue([]);
    desktopMock.getApplicationStatus.mockResolvedValue({ appVersion: "0.1.0", databaseSchemaVersion: 2, offlineCapable: true });
    desktopMock.getAiCredentialStatus.mockResolvedValue({ configured: false });
    desktopMock.recentImportJobs.mockResolvedValue([]);
  });
  it("provides accessible primary navigation and a command palette", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Prompt Hub" })).toBeVisible();
    expect(screen.getByRole("navigation", { name: "主导航" })).toBeVisible();
    expect(screen.getByRole("link", { name: "提示词库" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    fireEvent.click(screen.getByRole("button", { name: "打开命令面板" }));

    expect(screen.getByRole("dialog", { name: "命令面板" })).toBeVisible();
    expect(screen.getByRole("status", { name: "通知" })).toBeVisible();
  });

  it("opens the command palette by keyboard and restores trigger focus on escape", () => {
    render(<App />);
    const trigger = screen.getByRole("button", { name: "打开命令面板" });
    trigger.focus();

    fireEvent.keyDown(window, { ctrlKey: true, key: "k" });
    expect(screen.getByRole("dialog", { name: "命令面板" })).toBeVisible();

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "命令面板" })).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it("opens metadata editing for a selected library prompt", async () => {
    desktopMock.listPrompts.mockResolvedValueOnce([
      {
        id: "prompt-1",
        title: "代码审查",
        status: "published",
        effectiveness: "effective",
        category: "开发",
        tags: ["审查"],
        sourceNames: ["手动录入"],
        favorite: false,
        createdAt: "2026-07-15T00:00:00Z",
        updatedAt: "2026-07-15T00:01:00Z",
      },
    ]);
    desktopMock.promptHistory.mockResolvedValueOnce([
      { number: 1, body: "第一版", createdAt: "2026-07-15T00:00:00Z" },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：代码审查" }));
    expect(screen.getByRole("form", { name: "提示词元数据" })).toBeVisible();
    expect(await screen.findByRole("heading", { name: "版本历史" })).toBeVisible();
    expect(screen.getByRole("button", { name: "软删除提示词" })).toBeVisible();
  });

  it("opens local prompt search from navigation", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("link", { name: "搜索" }));
    expect(screen.getByRole("heading", { name: "搜索提示词" })).toBeVisible();
  });

  it("shows the import and review inbox from navigation", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("link", { name: "收件箱" }));
    expect(screen.getByRole("heading", { name: "收件箱" })).toBeVisible();
    expect(screen.getByRole("button", { name: "导入到收件箱" })).toBeDisabled();
  });

  it("opens local backup, diagnostics, and onboarding guidance from settings", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("link", { name: "设置" }));
    expect(screen.getByRole("heading", { name: "备份与恢复" })).toBeVisible();
    expect(await screen.findByRole("heading", { name: "诊断信息" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "开始使用" })).toBeVisible();
  });
});
