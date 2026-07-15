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
    createdAt: "2026-07-15T00:00:00Z",
    updatedAt: "2026-07-15T00:00:00Z",
  },
];

describe("library view helpers", () => {
  it("filters favorites and verification states while preserving newest-first order", () => {
    const visible = filterAndSortPrompts(fixtures, "effective");

    expect(visible.map((prompt) => prompt.id)).toEqual(["new-effective", "old-effective"]);
    expect(filterAndSortPrompts(fixtures, "favorite").map((prompt) => prompt.id)).toEqual(["favorite"]);
  });

  it("formats recent update times without exposing ISO timestamps", () => {
    const now = new Date("2026-07-16T12:00:00Z");

    expect(formatLibraryUpdatedAt("2026-07-16T11:58:00Z", now)).toBe("刚刚");
    expect(formatLibraryUpdatedAt("2026-07-15T12:00:00Z", now)).toBe("昨天");
    expect(formatLibraryUpdatedAt("2026-07-10T12:00:00Z", now)).toBe("7月10日");
  });
});
