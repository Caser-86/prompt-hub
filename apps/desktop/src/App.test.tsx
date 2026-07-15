import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const desktopMock = vi.hoisted(() => ({
  createManualPromptDraft: vi.fn(),
  listPrompts: vi.fn(),
  recordPromptCompatibility: vi.fn(),
  recordPromptValidation: vi.fn(),
}));
vi.mock("./services/desktop", () => ({
  desktopCommands: desktopMock,
}));

import { App } from "./App";

describe("App", () => {
  beforeEach(() => {
    desktopMock.listPrompts.mockImplementation(() => new Promise<never[]>(() => undefined));
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
        createdAt: "2026-07-15T00:00:00Z",
        updatedAt: "2026-07-15T00:01:00Z",
      },
    ]);
    render(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：代码审查" }));
    expect(screen.getByRole("form", { name: "提示词元数据" })).toBeVisible();
  });
});
