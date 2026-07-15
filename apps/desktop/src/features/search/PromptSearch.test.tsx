import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { PromptSearchPage } from "@prompt-hub/contracts";

import { PromptSearch } from "./PromptSearch";

describe("PromptSearch", () => {
  beforeEach(() => window.localStorage.clear());

  it("restores a saved local search view", () => {
    window.localStorage.setItem("prompt-hub.search-view", JSON.stringify({ query: "代码审查", category: "开发", sort: "rating", favoritesOnly: true }));

    render(<PromptSearch searchPrompts={vi.fn()} />);

    expect(screen.getByRole("searchbox", { name: "搜索提示词" })).toHaveValue("代码审查");
    expect(screen.getByRole("searchbox", { name: "搜索提示词" }).closest("label")).toHaveClass("search-query-field");
    expect(screen.getByRole("group", { name: "常用筛选" })).toBeVisible();
    expect(screen.getByText("更多筛选")).toBeVisible();
    expect(screen.getByLabelText("分类")).toHaveValue("开发");
    expect(screen.getByLabelText("排序方式")).toHaveValue("rating");
    expect(screen.getByLabelText("仅看收藏")).toBeChecked();
  });

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

  it("sends effectiveness and minimum-rating filters to local search", async () => {
    vi.useFakeTimers();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 0 });
    render(<PromptSearch searchPrompts={searchPrompts} />);

    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    fireEvent.change(screen.getByLabelText("有效性筛选"), { target: { value: "effective" } });
    fireEvent.change(screen.getByLabelText("最低评分"), { target: { value: "4" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenCalledWith("审查", 20, 0, {
      effectiveness: "effective",
      minimumRating: 4,
    });
    vi.useRealTimers();
  });

  it("paginates deterministically and saves the local view state", async () => {
    vi.useFakeTimers();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 41 });
    render(<PromptSearch searchPrompts={searchPrompts} />);
    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    fireEvent.click(screen.getByRole("button", { name: "下一页" }));
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    expect(searchPrompts).toHaveBeenLastCalledWith("审查", 20, 20);

    fireEvent.click(screen.getByRole("button", { name: "保存当前视图" }));
    expect(window.localStorage.getItem("prompt-hub.search-view")).toContain("审查");
    vi.useRealTimers();
  });

  it("passes lifecycle, provenance, metadata, and time filters to local search", async () => {
    vi.useFakeTimers();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 0 });
    render(<PromptSearch searchPrompts={searchPrompts} />);
    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    fireEvent.change(screen.getByLabelText("生命周期"), { target: { value: "published" } });
    fireEvent.change(screen.getByLabelText("来源类型"), { target: { value: "manual" } });
    fireEvent.change(screen.getByLabelText("分类"), { target: { value: "开发" } });
    fireEvent.change(screen.getByLabelText("标签"), { target: { value: "审查,安全" } });
    fireEvent.change(screen.getByLabelText("适用工具"), { target: { value: "Codex" } });
    fireEvent.change(screen.getByLabelText("适用模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("更新开始日期"), { target: { value: "2026-07-01" } });
    fireEvent.change(screen.getByLabelText("更新结束日期"), { target: { value: "2026-07-15" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenLastCalledWith("审查", 20, 0, {
      status: "published", sourceKind: "manual", category: "开发", tags: ["审查", "安全"],
      tool: "Codex", model: "gpt-5", updatedAfter: "2026-07-01T00:00:00Z", updatedBefore: "2026-07-15T23:59:59Z",
    });
    vi.useRealTimers();
  });

  it("switches the local search to explicit rating ordering", async () => {
    vi.useFakeTimers();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 0 });
    render(<PromptSearch searchPrompts={searchPrompts} />);

    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    fireEvent.change(screen.getByLabelText("排序方式"), { target: { value: "rating" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenLastCalledWith("审查", 20, 0, undefined, "rating");
    vi.useRealTimers();
  });
});
