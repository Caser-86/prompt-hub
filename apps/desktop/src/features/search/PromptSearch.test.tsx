import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { PromptSearchPage } from "@prompt-hub/contracts";

import { PromptSearch } from "./PromptSearch";

describe("PromptSearch", () => {
  beforeEach(() => window.localStorage.clear());
  afterEach(() => vi.useRealTimers());

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

  it("ignores invalid saved filter values instead of sending them to the database", async () => {
    vi.useFakeTimers();
    window.localStorage.setItem("prompt-hub.search-view", JSON.stringify({
      query: "审查",
      effectiveness: "not-a-status",
      minimumRating: "99",
      status: "not-a-lifecycle",
      sourceKind: "file://",
      updatedAfter: "not-a-date",
      updatedBefore: "2026-99-99",
      sort: "rating",
    }));
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 0 });

    render(<PromptSearch searchPrompts={searchPrompts} />);
    expect(screen.getByLabelText("有效性筛选")).toHaveValue("");
    expect(screen.getByLabelText("生命周期")).toHaveValue("");
    expect(screen.getByLabelText("来源类型")).toHaveValue("");
    expect(screen.getByLabelText("最低评分")).toHaveValue("");
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenCalledWith("审查", 20, 0, undefined, "rating");
    vi.useRealTimers();
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
  });

  it("runs a filter-only search without requiring a keyword", async () => {
    vi.useFakeTimers();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [], total: 0 });
    render(<PromptSearch searchPrompts={searchPrompts} />);

    fireEvent.change(screen.getByLabelText("有效性筛选"), { target: { value: "effective" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenCalledWith("", 20, 0, { effectiveness: "effective" });
    vi.useRealTimers();
  });

  it("does not display a stale response after a filter changes", async () => {
    vi.useFakeTimers();
    const pending = new Map<string, (page: PromptSearchPage) => void>();
    const searchPrompts = vi.fn((text: string, _limit?: number, _offset?: number, filters?: { effectiveness?: string }) => new Promise<PromptSearchPage>((resolve) => {
      pending.set(`${text}:${filters?.effectiveness ?? "all"}`, resolve);
    }));
    render(<PromptSearch searchPrompts={searchPrompts} />);
    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    fireEvent.change(screen.getByLabelText("有效性筛选"), { target: { value: "effective" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    await act(async () => {
      pending.get("审查:effective")?.({ hits: [{
        id: "new", title: "有效结果", snippet: "新片段", status: "published",
        effectiveness: "effective", rating: 5, updatedAt: "2026-07-15T00:00:00Z",
        sourceNames: ["手动录入"], applicableModels: ["gpt-5"],
      }], total: 1 });
    });
    await act(async () => {
      pending.get("审查:all")?.({ hits: [{
        id: "old", title: "旧结果", snippet: "旧片段", status: "published",
        effectiveness: "effective", rating: 5, updatedAt: "2026-07-15T00:00:00Z",
        sourceNames: ["手动录入"], applicableModels: ["gpt-4"],
      }], total: 1 });
    });

    expect(screen.getByText("有效结果")).toBeVisible();
    expect(screen.queryByText("旧结果")).not.toBeInTheDocument();
    vi.useRealTimers();
  });

  it("opens a selected search hit and shows its key metadata", async () => {
    vi.useFakeTimers();
    const onSelectPrompt = vi.fn();
    const searchPrompts = vi.fn().mockResolvedValue({ hits: [{
      id: "prompt-1", title: "会议纪要", snippet: "提取行动项", status: "published",
      effectiveness: "effective", rating: 4, updatedAt: "2026-07-15T00:00:00Z",
      sourceNames: ["内部模板"], applicableModels: ["gpt-5"],
    }], total: 1 });
    render(<PromptSearch onSelectPrompt={onSelectPrompt} searchPrompts={searchPrompts} />);
    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "会议" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(screen.getByText(/内部模板/)).toBeVisible();
    expect(screen.getByText(/gpt-5/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "打开提示词：会议纪要" }));
    expect(onSelectPrompt).toHaveBeenCalledWith("prompt-1");
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

  it("returns to the last valid page when the result set shrinks", async () => {
    vi.useFakeTimers();
    const hit = {
      id: "prompt-1", title: "审查结果", snippet: "结果", status: "published" as const,
      effectiveness: "effective" as const, rating: 5, updatedAt: "2026-07-15T00:00:00Z",
    };
    const searchPrompts = vi.fn()
      .mockResolvedValueOnce({ hits: [hit], total: 21 })
      .mockResolvedValueOnce({ hits: [], total: 3 })
      .mockResolvedValueOnce({ hits: [hit], total: 3 });
    render(<PromptSearch searchPrompts={searchPrompts} />);
    fireEvent.change(screen.getByRole("searchbox", { name: "搜索提示词" }), { target: { value: "审查" } });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    fireEvent.click(screen.getByRole("button", { name: "下一页" }));
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });
    await act(async () => { await vi.advanceTimersByTimeAsync(250); });

    expect(searchPrompts).toHaveBeenLastCalledWith("审查", 20, 0);
    expect(screen.getByText("审查结果")).toBeVisible();
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
