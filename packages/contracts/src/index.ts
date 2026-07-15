export type ApplicationStatus = {
  appVersion: string;
  databaseSchemaVersion: number;
  offlineCapable: boolean;
};

export type CommandInvoker = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

export type ManualPromptDraft = {
  title: string;
  body: string;
  description: string | null;
  category: string | null;
  tags: string[];
  variables: PromptVariableDraft[];
};

export type PromptVariableDraft = {
  name: string;
  kind: "text" | "number" | "boolean";
  description: string | null;
  defaultValue: string | null;
  required: boolean;
};

export type PromptSearchHit = {
  id: string;
  title: string;
  snippet: string;
  status: string;
  effectiveness: string;
  rating: number | null;
  updatedAt: string;
};

export type PromptSearchPage = {
  hits: PromptSearchHit[];
  total: number;
};

export type PromptListItem = {
  id: string;
  title: string;
  status: string;
  effectiveness: string;
  category: string | null;
  tags: string[];
  sourceNames: string[];
  createdAt: string;
  updatedAt: string;
};

export type PromptHistoryItem = {
  number: number;
  body: string;
  createdAt: string;
};

export type PromptCompatibilityDraft = {
  tool: string;
  model: string | null;
  status: "unknown" | "confirmed" | "unsupported";
  notes: string | null;
};

export type PromptValidationDraft = {
  status: "unverified" | "effective" | "ineffective" | "needs_retest";
  rating: number | null;
  notes: string | null;
};

export type DesktopCommandClient = {
  getApplicationStatus: () => Promise<ApplicationStatus>;
  listPrompts: () => Promise<PromptListItem[]>;
  promptHistory: (id: string) => Promise<PromptHistoryItem[]>;
  restorePromptVersion: (id: string, versionNumber: number) => Promise<unknown>;
  searchPrompts: (text: string, limit?: number, offset?: number) => Promise<PromptSearchPage>;
  recordPromptCompatibility: (id: string, metadata: PromptCompatibilityDraft) => Promise<unknown>;
  recordPromptValidation: (id: string, metadata: PromptValidationDraft) => Promise<unknown>;
  createManualPromptDraft: (draft: ManualPromptDraft) => Promise<unknown>;
};

export function createDesktopCommandClient(invoke: CommandInvoker): DesktopCommandClient {
  return {
    async getApplicationStatus() {
      const result = await invoke("get_application_status");
      if (!isApplicationStatus(result)) {
        throw new Error("get_application_status returned an invalid response");
      }
      return result;
    },
    async listPrompts() {
      const result = await invoke("list_prompts");
      if (!Array.isArray(result) || !result.every(isPromptListItem)) {
        throw new Error("list_prompts returned an invalid response");
      }
      return result;
    },
    async promptHistory(id) {
      const result = await invoke("prompt_history", { id });
      if (!Array.isArray(result) || !result.every(isPromptHistoryItem)) {
        throw new Error("prompt_history returned an invalid response");
      }
      return result;
    },
    restorePromptVersion(id, versionNumber) {
      return invoke("restore_prompt_version", { id, versionNumber });
    },
    async searchPrompts(text, limit = 20, offset = 0) {
      const result = await invoke("search_prompts", { text, limit, offset });
      if (!isPromptSearchPage(result)) {
        throw new Error("search_prompts returned an invalid response");
      }
      return result;
    },
    recordPromptCompatibility(id, metadata) {
      return invoke("record_prompt_compatibility", { id, metadata });
    },
    recordPromptValidation(id, metadata) {
      return invoke("record_prompt_validation", { id, metadata });
    },
    createManualPromptDraft(draft) {
      return invoke("create_manual_prompt_draft", { draft });
    },
  };
}

function isPromptListItem(value: unknown): value is PromptListItem {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const item = value as Record<string, unknown>;
  return (
    typeof item.id === "string" &&
    typeof item.title === "string" &&
    typeof item.status === "string" &&
    typeof item.effectiveness === "string" &&
    (typeof item.category === "string" || item.category === null) &&
    Array.isArray(item.tags) && item.tags.every((tag) => typeof tag === "string") &&
    Array.isArray(item.sourceNames) && item.sourceNames.every((name) => typeof name === "string") &&
    typeof item.createdAt === "string" &&
    typeof item.updatedAt === "string"
  );
}

function isPromptHistoryItem(value: unknown): value is PromptHistoryItem {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const item = value as Record<string, unknown>;
  return (
    typeof item.number === "number" &&
    typeof item.body === "string" &&
    typeof item.createdAt === "string"
  );
}

function isPromptSearchPage(value: unknown): value is PromptSearchPage {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const page = value as Record<string, unknown>;
  return (
    typeof page.total === "number" &&
    Array.isArray(page.hits) &&
    page.hits.every(isPromptSearchHit)
  );
}

function isPromptSearchHit(value: unknown): value is PromptSearchHit {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const hit = value as Record<string, unknown>;
  return (
    typeof hit.id === "string" &&
    typeof hit.title === "string" &&
    typeof hit.snippet === "string" &&
    typeof hit.status === "string" &&
    typeof hit.effectiveness === "string" &&
    (typeof hit.rating === "number" || hit.rating === null) &&
    typeof hit.updatedAt === "string"
  );
}

function isApplicationStatus(value: unknown): value is ApplicationStatus {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const status = value as Record<string, unknown>;
  return (
    typeof status.appVersion === "string" &&
    typeof status.databaseSchemaVersion === "number" &&
    typeof status.offlineCapable === "boolean"
  );
}
