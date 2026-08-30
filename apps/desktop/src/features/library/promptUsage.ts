const storageKey = "prompt-hub.prompt-usage";
const migrationKey = "prompt-hub.prompt-usage-migrated";

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

export function shouldMigratePromptUsage(storage: Storage = window.localStorage) {
  return storage.getItem(migrationKey) !== "1";
}

export function markPromptUsageMigrated(storage: Storage = window.localStorage) {
  storage.setItem(migrationKey, "1");
}
