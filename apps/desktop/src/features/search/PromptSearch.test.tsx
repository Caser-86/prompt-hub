import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { PromptSearchPage } from "@prompt-hub/contracts";

import { PromptSearch } from "./PromptSearch";

describe("PromptSearch", () => {
  it("debounces input and never displays a stale response", async () => {
    vi.useFakeTimers();
    const pending = new Map<string, (page: PromptSearchPage) => void>();
    const searchPrompts = vi.fn((text: string) => new Promise<PromptSearchPage>((resolve) => {
      pending.set(text, resolve);
    }));
    render(<PromptSearch searchPrompts={searchPrompts} />);

    const input = screen.getByRole("searchbox", { name: "搜索提示词" });
    fireEvent.change(input, { target: { value: "旧查询" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    fireEvent.change(input, { target: { value: "新查询" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    await act(async () => {
      pending.get("新查询")?.({
        hits: [{
          id: "new", title: "新结果", snippet: "新片段", status: "published",
          effectiveness: "effective", rating: 5, updatedAt: "2026-07-15T00:00:00Z",
        }], total: 1,
      });
    });
    await act(async () => {
      pending.get("旧查询")?.({
        hits: [{
          id: "old", title: "旧结果", snippet: "旧片段", status: "published",
          effectiveness: "effective", rating: 5, updatedAt: "2026-07-15T00:00:00Z",
        }], total: 1,
      });
    });

    expect(searchPrompts).toHaveBeenCalledWith("新查询", 20, 0);
    expect(screen.getByText("新结果")).toBeVisible();
    expect(screen.queryByText("旧结果")).not.toBeInTheDocument();
    vi.useRealTimers();
  });
});
