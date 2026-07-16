const storageKey = "prompt-hub.prompt-usage";

export type PromptUsage = Record<string, number>;

export function readPromptUsage(storage: Storage = window.localStorage): PromptUsage {
  try {
    const value: unknown = JSON.parse(storage.getItem(storageKey) ?? "{}");
    if (!value || typeof value !== "object") return {};
    return Object.fromEntries(Object.entries(value).filter(([, count]) => Number.isFinite(count) && Number(count) > 0).map(([id, count]) => [id, Number(count)]));
  } catch {
    return {};
  }
}

export function recordPromptUsage(id: string, storage: Storage = window.localStorage) {
  const usage = readPromptUsage(storage);
  const next = { ...usage, [id]: (usage[id] ?? 0) + 1 };
  storage.setItem(storageKey, JSON.stringify(next));
  return next;
}
