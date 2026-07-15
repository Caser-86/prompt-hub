import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptLibrary } from "./PromptLibrary";

describe("PromptLibrary", () => {
  it("shows an accessible empty library with a creation entry point", async () => {
    render(<PromptLibrary loadPrompts={async () => []} />);

    expect(await screen.findByRole("heading", { name: "提示词库" })).toBeVisible();
    expect(screen.getByRole("button", { name: "创建提示词" })).toBeVisible();
    expect(screen.getByText("还没有提示词资产")).toBeVisible();
  });

  it("opens the editor from the creation entry point", async () => {
    const onCreate = vi.fn();
    render(<PromptLibrary loadPrompts={async () => []} onCreate={onCreate} />);

    fireEvent.click(await screen.findByRole("button", { name: "创建提示词" }));
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
            favorite: false,
            createdAt: "2026-07-15T00:00:00Z",
            updatedAt: "2026-07-15T00:01:00Z",
          },
        ]}
        onSelect={onSelect}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：代码审查" }));
    expect(screen.getByText("来源：手动录入")).toBeVisible();
    expect(screen.getByText("有效")).toBeVisible();
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ id: "prompt-1" }));
  });
});
