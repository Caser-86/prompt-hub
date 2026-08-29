import type { PromptListItem } from "@prompt-hub/contracts";

export type PromptLibraryFilter = "all" | "favorite" | "effective" | "needs_retest";
export type PromptLibrarySort = "default" | "recently_used" | "recently_added" | "recently_updated" | "most_used";

export function filterAndSortPrompts(prompts: PromptListItem[], filter: PromptLibraryFilter, sort: PromptLibrarySort = "default") {
  return [...prompts]
    .filter((prompt) => filter === "all" || (filter === "favorite" ? prompt.favorite : prompt.effectiveness === filter))
    .sort((left, right) => {
      const countDifference = (right.useCount ?? 0) - (left.useCount ?? 0);
      const lastUsedDifference = dateValue(right.lastUsedAt) - dateValue(left.lastUsedAt);
      const createdDifference = dateValue(right.createdAt) - dateValue(left.createdAt);
      const updatedDifference = dateValue(right.updatedAt) - dateValue(left.updatedAt);
      if (sort === "recently_used") return lastUsedDifference || countDifference || updatedDifference || left.id.localeCompare(right.id);
      if (sort === "recently_added") return createdDifference || left.id.localeCompare(right.id);
      if (sort === "recently_updated") return updatedDifference || left.id.localeCompare(right.id);
      if (sort === "most_used") return countDifference || lastUsedDifference || Number(right.favorite) - Number(left.favorite) || left.id.localeCompare(right.id);
      return Number(right.favorite) - Number(left.favorite) || countDifference || lastUsedDifference || updatedDifference || left.id.localeCompare(right.id);
    });
}

function dateValue(value: string | null | undefined) {
  if (!value) return 0;
  const parsed = Date.parse(value);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export function formatLibraryUpdatedAt(value: string, now = new Date()) {
  const updatedAt = new Date(value);
  const age = now.getTime() - updatedAt.getTime();

  if (Number.isNaN(updatedAt.getTime())) return "时间未知";
  if (age < 60 * 60 * 1000) return "刚刚";
  if (age < 24 * 60 * 60 * 1000) return "今天";
  if (age < 48 * 60 * 60 * 1000) return "昨天";
  return `${updatedAt.getMonth() + 1}月${updatedAt.getDate()}日`;
}
