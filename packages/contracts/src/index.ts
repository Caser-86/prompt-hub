export type ApplicationStatus = {
  appVersion: string;
  databaseSchemaVersion: number;
  offlineCapable: boolean;
};

export type CommandInvoker = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

export type DesktopCommandClient = {
  getApplicationStatus: () => Promise<ApplicationStatus>;
  listPrompts: () => Promise<unknown[]>;
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
  };
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
