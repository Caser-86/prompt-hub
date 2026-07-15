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

export type DesktopCommandClient = {
  getApplicationStatus: () => Promise<ApplicationStatus>;
  listPrompts: () => Promise<unknown[]>;
  searchPrompts: (text: string, limit?: number, offset?: number) => Promise<PromptSearchPage>;
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
      if (!Array.isArray(result)) {
        throw new Error("list_prompts returned an invalid response");
      }
      return result;
    },
    async searchPrompts(text, limit = 20, offset = 0) {
      const result = await invoke("search_prompts", { text, limit, offset });
      if (!isPromptSearchPage(result)) {
        throw new Error("search_prompts returned an invalid response");
      }
      return result;
    },
    createManualPromptDraft(draft) {
      return invoke("create_manual_prompt_draft", { draft });
    },
  };
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
