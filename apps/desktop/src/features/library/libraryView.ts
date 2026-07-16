import type { PromptListItem } from "@prompt-hub/contracts";

export type PromptLibraryFilter = "all" | "favorite" | "effective" | "needs_retest";

export function filterAndSortPrompts(prompts: PromptListItem[], filter: PromptLibraryFilter, usage: Record<string, number> = {}) {
  return prompts
    .filter((prompt) => filter === "all" || (filter === "favorite" ? prompt.favorite : prompt.effectiveness === filter))
    .sort((left, right) => {
      const favoriteDifference = Number(right.favorite) - Number(left.favorite);
      if (favoriteDifference) return favoriteDifference;
      const usageDifference = (usage[right.id] ?? 0) - (usage[left.id] ?? 0);
      if (usageDifference) return usageDifference;
      return Date.parse(right.updatedAt) - Date.parse(left.updatedAt);
    });
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
