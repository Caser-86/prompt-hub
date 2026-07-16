import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptLibrary } from "./PromptLibrary";

describe("PromptLibrary", () => {
  it("shows an accessible empty library with a creation entry point", async () => {
    render(<PromptLibrary loadPrompts={async () => []} />);

    expect(await screen.findByRole("heading", { name: "提示词库" })).toBeVisible();
    expect(screen.getByRole("button", { name: "创建第一条提示词" })).toBeVisible();
    expect(screen.getByLabelText("空提示词库")).toHaveClass("empty-library-state");
  });

  it("uses a labeled favorite control instead of a standalone star glyph", async () => {
    render(<PromptLibrary loadPrompts={async () => [{
      id: "favorite", title: "收藏提示词", status: "published", effectiveness: "unverified", category: null,
      tags: [], sourceNames: [], favorite: false, createdAt: "2026-07-15T00:00:00Z", updatedAt: "2026-07-15T00:00:00Z",
    }]} />);

    expect(await screen.findByRole("button", { name: "收藏提示词：收藏提示词" })).not.toHaveTextContent("收藏");
  });

  it("gives saved prompts a distinct favorite state", async () => {
    render(<PromptLibrary loadPrompts={async () => [{
      id: "favorite", title: "重点提示词", status: "published", effectiveness: "unverified", category: null,
      tags: [], sourceNames: [], favorite: true, createdAt: "2026-07-15T00:00:00Z", updatedAt: "2026-07-15T00:00:00Z",
    }]} />);

    expect(await screen.findByRole("button", { name: "取消收藏提示词：重点提示词" })).toHaveClass("is-favorite");
  });

  it("opens the editor from the creation entry point", async () => {
    const onCreate = vi.fn();
    render(<PromptLibrary loadPrompts={async () => []} onCreate={onCreate} />);

    fireEvent.click(await screen.findByRole("button", { name: "创建第一条提示词" }));
    expect(onCreate).toHaveBeenCalledOnce();
  });

  it("shows prompt provenance and opens a selected prompt", async () => {
    const onSelect = vi.fn();
    render(
      <PromptLibrary
        loadPrompts={async () => [
          {
            id: "prompt-1",
            title: "代码审查",
            status: "published",
            effectiveness: "effective",
            category: "开发",
            tags: ["审查"],
            sourceNames: ["手动录入"],
            applicableTools: ["Codex"],
            applicableModels: ["gpt-5"],
            rating: 5,
            favorite: false,
            createdAt: "2026-07-15T00:00:00Z",
            updatedAt: "2026-07-15T00:01:00Z",
          },
        ]}
        onSelect={onSelect}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：代码审查" }));
    expect(screen.getByRole("list", { name: "提示词列表" })).toHaveClass("prompt-list");
    expect(screen.getByRole("listitem")).toHaveClass("prompt-list-item");
    expect(screen.getByText("来源：手动录入")).toBeVisible();
    expect(screen.getByText("有效")).toBeVisible();
    expect(screen.getByText("工具：Codex")).toBeVisible();
    expect(screen.queryByText(/适用模型：/)).not.toBeInTheDocument();
    expect(screen.queryByText(/评分：/)).not.toBeInTheDocument();
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: "prompt-1" }));
  });

  it("renders compact rows without model or rating and filters verified prompts", async () => {
    render(
      <PromptLibrary
        loadPrompts={async () => [
          {
            id: "effective", title: "已验证提示词", status: "published", effectiveness: "effective", category: null, tags: ["会议"], sourceNames: ["手动录入"], applicableTools: ["Codex"], applicableModels: ["gpt-5"], rating: 5, favorite: false,
            createdAt: "2026-07-15T00:00:00Z", updatedAt: "2026-07-15T00:00:00Z",
          },
          {
            id: "favorite", title: "收藏提示词", status: "published", effectiveness: "unverified", category: null, tags: [], sourceNames: ["网页导入"], favorite: true,
            createdAt: "2026-07-14T00:00:00Z", updatedAt: "2026-07-14T00:00:00Z",
          },
          {
            id: "retest", title: "待复测提示词", status: "published", effectiveness: "needs_retest", category: null, tags: [], sourceNames: ["手动录入"], favorite: false,
            createdAt: "2026-07-13T00:00:00Z", updatedAt: "2026-07-13T00:00:00Z",
          },
        ]}
      />,
    );

    expect(await screen.findByText("共 3 条提示词")).toBeVisible();
    expect(screen.getByRole("list", { name: "提示词列表" })).toHaveClass("prompt-list");
    expect(screen.queryByText(/评分：/)).not.toBeInTheDocument();
    expect(screen.queryByText(/适用模型：/)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "已验证" }));
    expect(screen.getAllByRole("listitem")).toHaveLength(1);
    expect(screen.getByText("已验证提示词")).toBeVisible();
  });

  it("confirms a recoverable batch archive for selected prompts", async () => {
    const batchArchive = vi.fn().mockResolvedValue(undefined);
    render(<PromptLibrary batchArchive={batchArchive} loadPrompts={async () => [{
      id: "prompt-1", title: "代码审查", status: "published", effectiveness: "effective", category: "开发", tags: [], sourceNames: [], favorite: false,
      createdAt: "2026-07-15T00:00:00Z", updatedAt: "2026-07-15T00:00:00Z",
    }]} />);
    fireEvent.click(await screen.findByRole("checkbox", { name: "选择提示词：代码审查" }));
    expect(screen.getByRole("status", { name: "批量管理提示" })).toHaveTextContent("已选择 1 条提示词");
    expect(screen.getByRole("status", { name: "批量管理提示" })).toHaveTextContent("归档不会删除");
    fireEvent.click(screen.getByRole("button", { name: "归档已选 1 条提示词" }));
    expect(screen.getByText("批量归档可在提示词详情中恢复。" )).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "确认归档" }));
    await waitFor(() => expect(batchArchive).toHaveBeenCalledWith(["prompt-1"]));
  });
});
