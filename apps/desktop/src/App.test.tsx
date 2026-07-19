import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const desktopMock = vi.hoisted(() => ({
  createManualPromptDraft: vi.fn(),
  publishPrompt: vi.fn(),
  importFileToInbox: vi.fn(),
  importFolderToInbox: vi.fn(),
  importUrlToInbox: vi.fn(),
  recentImportJobs: vi.fn(),
  listPrompts: vi.fn(),
  collectSkillFolder: vi.fn(),
  collectGitSkill: vi.fn(),
  listSkills: vi.fn(),
  getSkill: vi.fn(),
  reviewSkill: vi.fn(),
  setSkillFavorite: vi.fn(),
  installSkill: vi.fn(),
  verifySkillInstallation: vi.fn(),
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
  pruneLocalBackups: vi.fn(),
  getAiCredentialStatus: vi.fn(),
  saveAiCredential: vi.fn(),
  generateAiDraft: vi.fn(),
  getMcpSetup: vi.fn(),
  getApplicationStatus: vi.fn(),
  getDiagnosticsStatus: vi.fn(),
  getRedactedDiagnosticEvents: vi.fn(),
  rebuildSearchIndex: vi.fn(),
}));
vi.mock("./services/desktop", () => ({
  desktopCommands: desktopMock,
}));

import { App } from "./App";

describe("App", () => {
  beforeEach(() => {
    desktopMock.listPrompts.mockImplementation(() => new Promise<never[]>(() => undefined));
    desktopMock.getDiagnosticsStatus.mockResolvedValue({ databaseAvailable: true, searchIndexConsistent: true, mcpDatabaseAvailable: true });
    desktopMock.getRedactedDiagnosticEvents.mockResolvedValue([]);
    desktopMock.promptHistory.mockResolvedValue([]);
    desktopMock.getApplicationStatus.mockResolvedValue({ appVersion: "0.1.0", databaseSchemaVersion: 2, offlineCapable: true });
    desktopMock.getAiCredentialStatus.mockResolvedValue({ configured: false });
    desktopMock.recentImportJobs.mockResolvedValue([]);
    desktopMock.getMcpSetup.mockResolvedValue({ databasePath: "C:/data/prompt-hub.db", databaseAvailable: true, configuration: "{}" });
    desktopMock.listSkills.mockResolvedValue([]);
  });

  it("loads the global application stylesheet from the React entry point", () => {
    const entry = readFileSync(resolve(import.meta.dirname, "main.tsx"), "utf8");

    expect(entry).toContain('import "./styles.css"');
  });

  it("provides accessible primary navigation and a command palette", () => {
    render(<App />);

    expect(screen.getByRole("link", { name: "Prompt Hub" })).toBeVisible();
    expect(screen.getByRole("navigation", { name: "主导航" })).toBeVisible();
    expect(screen.getByRole("link", { name: "提示词库" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    fireEvent.click(screen.getByRole("button", { name: "打开命令面板" }));

    expect(screen.getByRole("dialog", { name: "命令面板" })).toBeVisible();
    expect(screen.getByRole("status", { name: "通知" })).toBeVisible();
  });

  it("renders a persistent workspace header and primary creation action", () => {
    render(<App />);

    expect(screen.getByRole("banner")).toHaveClass("workspace-header");
    expect(screen.getByRole("button", { name: "新建提示词" })).toHaveClass("button-primary");
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

  it("uses command palette actions to create a draft", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "打开命令面板" }));
    fireEvent.click(within(screen.getByRole("dialog", { name: "命令面板" })).getByRole("button", { name: "新建提示词" }));
    expect(screen.getByRole("form", { name: "提示词编辑器" })).toBeVisible();
  });

  it("searches prompt titles from the command palette and opens the selected prompt", async () => {
    desktopMock.listPrompts.mockResolvedValue([{
      id: "prompt-1", title: "可复现故障排查", status: "inbox", effectiveness: "unverified", category: "开发",
      tags: ["排查"], sourceNames: ["官方资料整理"], favorite: false,
      createdAt: "2026-07-16T00:00:00Z", updatedAt: "2026-07-16T00:00:00Z",
    }]);
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "打开命令面板" }));
    fireEvent.change(screen.getByLabelText("快速操作"), { target: { value: "排查" } });
    fireEvent.click(await within(screen.getByRole("dialog", { name: "命令面板" })).findByRole("button", { name: "打开提示词：可复现故障排查" }));

    expect(await screen.findByRole("heading", { name: "可复现故障排查" })).toBeVisible();
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
        sources: [{ kind: "web_url", name: "工程规范", location: "https://example.com/guide", collectedAt: "2026-07-15T00:00:00Z" }],
        favorite: false,
        createdAt: "2026-07-15T00:00:00Z",
        updatedAt: "2026-07-15T00:01:00Z",
      },
    ]);
    desktopMock.promptHistory.mockResolvedValueOnce([
      {
        number: 1,
        body: [
          "第一版",
          "",
          "参考来源： Google Prompt design strategies",
          "https://ai.google.dev/gemini-api/docs/prompting-strategies",
          "采集时间： 2026-07-16",
        ].join("\n"),
        createdAt: "2026-07-15T00:00:00Z",
      },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：代码审查" }));
    expect(await screen.findByLabelText("提示词正文")).toHaveTextContent("第一版");
    expect(screen.getByLabelText("提示词正文")).not.toHaveTextContent("参考来源： Google Prompt design strategies");
    expect(screen.getByLabelText("提示词主操作")).toContainElement(
      screen.getByRole("button", { name: "复制提示词正文" }),
    );
    expect(screen.getByText("编辑提示词信息")).toBeVisible();
    expect(await screen.findByRole("heading", { name: "版本历史" })).toBeVisible();
    fireEvent.click(screen.getByText("完整来源").closest("summary")!);
    expect(screen.getByText(/工程规范 · https:\/\/example.com\/guide/)).toBeVisible();
    expect(within(screen.getByLabelText("提示词信息")).getAllByText("2026-07-15T00:00:00Z").at(-1)).toBeVisible();
    expect(within(screen.getByLabelText("提示词信息")).getByText("审查")).toBeVisible();
    expect(screen.getByLabelText("提示词信息")).toHaveClass("prompt-detail-info");
    expect(screen.getByLabelText("更多操作")).toHaveClass("prompt-detail-more");
    expect(screen.getByRole("button", { name: "软删除提示词" })).toBeVisible();
  });

  it("opens advanced filters from the library instead of a separate navigation item", () => {
    render(<App />);

    expect(screen.queryByRole("link", { name: "搜索" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "高级筛选" }));
    expect(screen.getByRole("heading", { name: "高级筛选" })).toBeVisible();
  });

  it("uses page headers and grouped panels on secondary workspace routes", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "高级筛选" }));
    expect(screen.getByRole("heading", { name: "高级筛选" }).closest("header")).toHaveClass("page-header");

    fireEvent.click(screen.getByRole("link", { name: "收件箱" }));
    expect(screen.getByLabelText("导入操作")).toHaveClass("import-panel");
  });

  it("opens the Skill library from primary navigation", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("link", { name: "Skill 库" }));
    expect(await screen.findByRole("heading", { name: "Skill 库" })).toBeVisible();
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
