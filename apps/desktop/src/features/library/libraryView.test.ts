import { describe, expect, it } from "vitest";

import type { PromptListItem } from "@prompt-hub/contracts";

import { filterAndSortPrompts, formatLibraryUpdatedAt } from "./libraryView";

const fixtures: PromptListItem[] = [
  {
    id: "old-effective",
    title: "旧的有效提示词",
    status: "published",
    effectiveness: "effective",
    category: null,
    tags: [],
    sourceNames: [],
    favorite: false,
    useCount: 0,
    lastUsedAt: null,
    createdAt: "2026-07-10T00:00:00Z",
    updatedAt: "2026-07-10T00:00:00Z",
  },
  {
    id: "favorite",
    title: "收藏提示词",
    status: "published",
    effectiveness: "unverified",
    category: null,
    tags: [],
    sourceNames: [],
    favorite: true,
    useCount: 0,
    lastUsedAt: null,
    createdAt: "2026-07-12T00:00:00Z",
    updatedAt: "2026-07-12T00:00:00Z",
  },
  {
    id: "new-effective",
    title: "新的有效提示词",
    status: "published",
    effectiveness: "effective",
    category: null,
    tags: [],
    sourceNames: [],
    favorite: false,
    useCount: 8,
    lastUsedAt: "2026-07-15T00:01:00Z",
    createdAt: "2026-07-15T00:00:00Z",
    updatedAt: "2026-07-15T00:00:00Z",
  },
];

describe("library view helpers", () => {
  it("keeps favorites first, then ranks commonly used prompts before recency", () => {
    expect(filterAndSortPrompts(fixtures, "all", "default").map((prompt) => prompt.id)).toEqual([
      "favorite", "new-effective", "old-effective",
    ]);
    const visible = filterAndSortPrompts(fixtures, "effective");

    expect(visible.map((prompt) => prompt.id)).toEqual(["new-effective", "old-effective"]);
    expect(filterAndSortPrompts(fixtures, "favorite").map((prompt) => prompt.id)).toEqual(["favorite"]);
  });

  it("supports explicit recently-used, recently-added, recently-updated, and most-used orders", () => {
    expect(filterAndSortPrompts(fixtures, "all", "recently_used").map((prompt) => prompt.id)).toEqual(["new-effective", "favorite", "old-effective"]);
    expect(filterAndSortPrompts(fixtures, "all", "recently_added").map((prompt) => prompt.id)).toEqual(["new-effective", "favorite", "old-effective"]);
    expect(filterAndSortPrompts(fixtures, "all", "most_used").map((prompt) => prompt.id)).toEqual(["new-effective", "favorite", "old-effective"]);
  });

  it("formats recent update times without exposing ISO timestamps", () => {
    const now = new Date("2026-07-16T12:00:00Z");

    expect(formatLibraryUpdatedAt("2026-07-16T11:58:00Z", now)).toBe("刚刚");
    expect(formatLibraryUpdatedAt("2026-07-15T12:00:00Z", now)).toBe("昨天");
    expect(formatLibraryUpdatedAt("2026-07-10T12:00:00Z", now)).toBe("7月10日");
  });
});
