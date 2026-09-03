import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { CommandPalette } from "./CommandPalette";

describe("CommandPalette", () => {
  it("uses the indexed prompt search so body-only matches are discoverable", async () => {
    const onSelectPrompt = vi.fn();
    const prompt = {
      id: "prompt-1", title: "会议模板", status: "published", effectiveness: "effective", category: null,
      tags: [], sourceNames: [], favorite: false, createdAt: "2026-07-15T00:00:00Z", updatedAt: "2026-07-15T00:00:00Z",
    };
    const searchPrompts = vi.fn().mockResolvedValue({
      hits: [{ id: "prompt-1", title: "会议模板", snippet: "包含正文命中", status: "published", effectiveness: "effective", rating: null, updatedAt: prompt.updatedAt }],
      total: 1,
    });
    render(<CommandPalette
      loadPrompts={async () => [prompt]}
      onClose={vi.fn()}
      onCreate={vi.fn()}
      onNavigate={vi.fn()}
      onSelectPrompt={onSelectPrompt}
      searchPrompts={searchPrompts}
    />);

    fireEvent.change(screen.getByLabelText("快速操作"), { target: { value: "正文命中" } });
    await waitFor(() => expect(searchPrompts).toHaveBeenCalledWith("正文命中", 8, 0, undefined, "relevance"));
    fireEvent.click(await screen.findByRole("button", { name: "打开提示词：会议模板" }));
    expect(onSelectPrompt).toHaveBeenCalledWith(prompt);
  });

  it("shows a recoverable state when the local prompt index cannot be loaded", async () => {
    render(<CommandPalette
      loadPrompts={async () => { throw new Error("database unavailable"); }}
      onClose={vi.fn()}
      onCreate={vi.fn()}
      onNavigate={vi.fn()}
      onSelectPrompt={vi.fn()}
    />);

    expect(await screen.findByRole("alert")).toHaveTextContent("提示词索引暂时不可用");
  });
});
