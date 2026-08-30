export type ApplicationStatus = {
  appVersion: string;
  databaseSchemaVersion: number;
  offlineCapable: boolean;
};
export type BootstrapStatus = {
  state: "ready" | "recovery";
  code: string | null;
  safeMessage: string | null;
  backupName: string | null;
};

export type BackupInfo = { path: string; byteLen: number; schemaVersion: number };
export type BackupRestorePreview = { targetExists: boolean; backupSchemaVersion: number; backupByteLen: number; promptCount: number };
export type ImportResult = { imported: number; skippedDuplicates: number; failed: number };
export type AiCredentialStatus = { configured: boolean };
export type AiConnectionRequest = { endpoint: string; providerId: string; model: string };
export type AiConnectionStatus = { connected: boolean };
export type AiGenerationRequest = {
  taskId: string;
  endpoint: string;
  providerId: string;
  instruction: string;
  inputSummary: string;
  model: string;
};
export type ImportJobSummary = {
  id: string;
  sourceKind: string;
  sourcePath: string | null;
  status: string;
  startedAt: string;
  completedAt: string | null;
  imported: number;
  skippedDuplicates: number;
  failed: number;
};
export type McpSetupInfo = { databasePath: string; databaseAvailable: boolean; configuration: string };
export type DiagnosticsStatus = { databaseAvailable: boolean; searchIndexConsistent: boolean; mcpDatabaseAvailable: boolean };
export type RedactedDiagnosticEvent = { occurredAt: string; event: string; recommendation: string };

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

export type PromptSearchFilters = {
  favorite?: boolean;
  status?: "inbox" | "published" | "archived" | "deleted";
  effectiveness?: "unverified" | "effective" | "ineffective" | "needs_retest";
  sourceKind?: "manual" | "file_import" | "web_url" | "ai_generated" | "mcp";
  category?: string;
  tags?: string[];
  tool?: string;
  model?: string;
  minimumRating?: number;
  updatedAfter?: string;
  updatedBefore?: string;
};
export type PromptSearchSort = "relevance" | "updated_at" | "rating";

export type PromptListItem = {
  id: string;
  title: string;
  status: string;
  effectiveness: string;
  category: string | null;
  tags: string[];
  sourceNames: string[];
  sources?: Array<{ kind: string; name: string; location: string | null; collectedAt: string; rawExcerpt?: string | null; importJobId?: string | null }>;
  applicableTools?: string[];
  applicableModels?: string[];
  rating?: number | null;
  favorite: boolean;
  useCount?: number;
  lastUsedAt?: string | null;
  importedAt?: string | null;
  lastValidatedAt?: string | null;
  createdAt: string;
  updatedAt: string;
};

export type PromptHistoryItem = {
  number: number;
  body: string;
  createdAt: string;
};

export type SkillReviewStatus = "pending_review" | "approved" | "rejected" | "risk_pending_confirmation";
export type SkillSource = { kind: string; location: string; revision: string | null };
export type SkillListItem = {
  id: string;
  name: string;
  description: string;
  source: SkillSource;
  risks: string[];
  reviewStatus: SkillReviewStatus;
  favorite: boolean;
  updatedAt: string;
};
export type SkillFileItem = { relativePath: string; bytes: number; sha256: string; kind: string };
export type SkillDetail = SkillListItem & {
  reviewNotes: string | null;
  skillMarkdown: string;
  files: SkillFileItem[];
  contentHash: string;
  createdAt: string;
  installation: SkillInstallation | null;
};
export type SkillReviewDraft = { status: SkillReviewStatus; notes: string | null };
export type SkillInstallDraft = { targetRoot: string; destinationName: string; replaceAfterBackup: boolean };
export type SkillInstallation = { installPath: string; backupPath: string | null; installedHash: string };
export type SkillInstallationVerification = { state: "matching" | "drifted" | "unavailable" };
export type GitSkillCollectionDraft = { repositoryUrl: string; commit: string; subdirectory: string };

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
  getBootstrapStatus: () => Promise<BootstrapStatus>;
  retryDatabaseBootstrap: () => Promise<BootstrapStatus>;
  exportBootstrapDiagnostics: () => Promise<string>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
  getDiagnosticsStatus: () => Promise<DiagnosticsStatus>;
  getRedactedDiagnosticEvents: () => Promise<RedactedDiagnosticEvent[]>;
  rebuildSearchIndex: () => Promise<void>;
  createManualBackup: (directory?: string) => Promise<BackupInfo>;
  previewBackupRestore: (path: string) => Promise<BackupRestorePreview>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  pruneLocalBackups: (retain: number) => Promise<number>;
  listPrompts: () => Promise<PromptListItem[]>;
  recordPromptUse: (id: string) => Promise<{ useCount: number; lastUsedAt: string | null }>;
  migrateLegacyPromptUsage: (entries: Array<{ id: string; useCount: number }>) => Promise<void>;
    collectSkillFolder: (path: string) => Promise<SkillListItem>;
    collectGitSkill: (source: GitSkillCollectionDraft) => Promise<SkillListItem>;
  listSkills: () => Promise<SkillListItem[]>;
  getSkill: (id: string) => Promise<SkillDetail | null>;
    reviewSkill: (id: string, review: SkillReviewDraft) => Promise<void>;
    setSkillFavorite: (id: string, favorite: boolean) => Promise<void>;
    installSkill: (id: string, installation: SkillInstallDraft) => Promise<SkillInstallation>;
    verifySkillInstallation: (id: string) => Promise<SkillInstallationVerification>;
  promptHistory: (id: string) => Promise<PromptHistoryItem[]>;
  restorePromptVersion: (id: string, versionNumber: number) => Promise<unknown>;
  archivePrompt: (id: string) => Promise<unknown>;
  batchArchivePrompts: (ids: string[]) => Promise<unknown>;
  softDeletePrompt: (id: string) => Promise<unknown>;
  permanentlyDeletePrompt: (id: string) => Promise<BackupInfo>;
  recoverPrompt: (id: string) => Promise<unknown>;
  setPromptFavorite: (id: string, favorite: boolean) => Promise<unknown>;
  searchPrompts: (
    text: string,
    limit?: number,
    offset?: number,
    filters?: PromptSearchFilters,
    sort?: PromptSearchSort,
  ) => Promise<PromptSearchPage>;
  recordPromptCompatibility: (id: string, metadata: PromptCompatibilityDraft) => Promise<unknown>;
  recordPromptValidation: (id: string, metadata: PromptValidationDraft) => Promise<unknown>;
  createManualPromptDraft: (draft: ManualPromptDraft) => Promise<unknown>;
  publishPrompt: (id: string) => Promise<unknown>;
  importFileToInbox: (path: string) => Promise<ImportResult>;
  importFolderToInbox: (path: string) => Promise<ImportResult>;
  importUrlToInbox: (url: string) => Promise<ImportResult>;
  recentImportJobs: () => Promise<ImportJobSummary[]>;
  getMcpSetup: () => Promise<McpSetupInfo>;
  getAiCredentialStatus: (providerId: string) => Promise<AiCredentialStatus>;
  saveAiCredential: (providerId: string, secret: string) => Promise<AiCredentialStatus>;
  testAiConnection: (request: AiConnectionRequest) => Promise<AiConnectionStatus>;
  cancelAiGeneration: (taskId: string) => Promise<void>;
  optimizeAiPrompt: (id: string, request: AiGenerationRequest) => Promise<unknown>;
  generateAiDraft: (request: AiGenerationRequest) => Promise<unknown>;
};

export function createDesktopCommandClient(invoke: CommandInvoker): DesktopCommandClient {
  return {
    async getBootstrapStatus() {
      const result = await invoke("get_bootstrap_status");
      if (!isBootstrapStatus(result)) throw new Error("get_bootstrap_status returned an invalid response");
      return result;
    },
    async retryDatabaseBootstrap() {
      const result = await invoke("retry_database_bootstrap");
      if (!isBootstrapStatus(result)) throw new Error("retry_database_bootstrap returned an invalid response");
      return result;
    },
    async exportBootstrapDiagnostics() {
      const result = await invoke("export_bootstrap_diagnostics");
      if (typeof result !== "string") throw new Error("export_bootstrap_diagnostics returned an invalid response");
      return result;
    },
    async getApplicationStatus() {
      const result = await invoke("get_application_status");
      if (!isApplicationStatus(result)) {
        throw new Error("get_application_status returned an invalid response");
      }
      return result;
    },
    async getDiagnosticsStatus() {
      const result = await invoke("get_diagnostics_status");
      if (!isDiagnosticsStatus(result)) throw new Error("get_diagnostics_status returned an invalid response");
      return result;
    },
    async getRedactedDiagnosticEvents() {
      const result = await invoke("get_redacted_diagnostic_events");
      if (!Array.isArray(result) || !result.every(isRedactedDiagnosticEvent)) throw new Error("get_redacted_diagnostic_events returned an invalid response");
      return result;
    },
    async rebuildSearchIndex() {
      await invoke("rebuild_search_index");
    },
    async createManualBackup(directory) {
      const result = await invoke("create_manual_backup", directory ? { directory } : undefined);
      if (!isBackupInfo(result)) throw new Error("create_manual_backup returned an invalid response");
      return result;
    },
    async previewBackupRestore(path) {
      const result = await invoke("preview_backup_restore", { path });
      if (!isBackupRestorePreview(result)) throw new Error("preview_backup_restore returned an invalid response");
      return result;
    },
    async restoreBackup(path) {
      const result = await invoke("restore_backup", { path });
      if (!isBackupInfo(result)) {
        throw new Error("restore_backup returned an invalid response");
      }
      return result;
    },
    async pruneLocalBackups(retain) {
      const result = await invoke("prune_local_backups", { retain });
      if (typeof result !== "number") throw new Error("prune_local_backups returned an invalid response");
      return result;
    },
    async listPrompts() {
      const result = await invoke("list_prompts");
      if (!Array.isArray(result) || !result.every(isPromptListItem)) {
        throw new Error("list_prompts returned an invalid response");
      }
      return result;
    },
    async recordPromptUse(id) {
      const result = await invoke("record_prompt_use", { id });
      if (!isPromptUsageStats(result)) throw new Error("record_prompt_use returned an invalid response");
      return result;
    },
    async migrateLegacyPromptUsage(entries) {
      await invoke("migrate_legacy_prompt_usage", { entries });
    },
      async collectSkillFolder(path) {
      const result = await invoke("collect_skill_folder", { path });
      if (!isSkillListItem(result)) throw new Error("collect_skill_folder returned an invalid response");
      return result;
    },
    async listSkills() {
      const result = await invoke("list_skills");
      if (!Array.isArray(result) || !result.every(isSkillListItem)) throw new Error("list_skills returned an invalid response");
      return result;
    },
    async getSkill(id) {
      const result = await invoke("get_skill", { id });
      if (result !== null && !isSkillDetail(result)) throw new Error("get_skill returned an invalid response");
      return result;
    },
    async reviewSkill(id, review) {
      await invoke("review_skill", { id, review });
    },
      async setSkillFavorite(id, favorite) {
        await invoke("set_skill_favorite", { id, favorite });
      },
      async installSkill(id, installation) {
        const result = await invoke("install_skill", { id, installation });
        if (!isSkillInstallation(result)) throw new Error("install_skill returned an invalid response");
        return result;
      },
      async verifySkillInstallation(id) {
        const result = await invoke("verify_skill_installation", { id });
        if (!isSkillInstallationVerification(result)) throw new Error("verify_skill_installation returned an invalid response");
        return result;
      },
      async collectGitSkill(source) {
        const result = await invoke("collect_git_skill", { source });
        if (!isSkillListItem(result)) throw new Error("collect_git_skill returned an invalid response");
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
    archivePrompt(id) {
      return invoke("archive_prompt", { id });
    },
    batchArchivePrompts(ids) {
      return invoke("batch_archive_prompts", { ids });
    },
    softDeletePrompt(id) {
      return invoke("soft_delete_prompt", { id });
    },
    async permanentlyDeletePrompt(id) {
      const result = await invoke("permanently_delete_prompt", { id });
      if (!isBackupInfo(result)) throw new Error("permanently_delete_prompt returned an invalid response");
      return result;
    },
    recoverPrompt(id) {
      return invoke("recover_prompt", { id });
    },
    setPromptFavorite(id, favorite) {
      return invoke("set_prompt_favorite", { id, favorite });
    },
    async searchPrompts(text, limit = 20, offset = 0, filters, sort) {
      const args = filters === undefined ? { text, limit, offset } : { text, limit, offset, filters };
      const result = await invoke("search_prompts", sort === undefined ? args : { ...args, sort });
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
    publishPrompt(id) {
      return invoke("publish_prompt", { id });
    },
    async importFileToInbox(path) {
      const result = await invoke("import_file_to_inbox", { path });
      if (!isImportResult(result)) {
        throw new Error("import_file_to_inbox returned an invalid response");
      }
      return result as ImportResult;
    },
    async importFolderToInbox(path) {
      const result = await invoke("import_folder_to_inbox", { path });
      if (!isImportResult(result)) {
        throw new Error("import_folder_to_inbox returned an invalid response");
      }
      return result as ImportResult;
    },
    async importUrlToInbox(url) {
      const result = await invoke("import_url_to_inbox", { url });
      if (!isImportResult(result)) throw new Error("import_url_to_inbox returned an invalid response");
      return result as ImportResult;
    },
    async recentImportJobs() {
      const result = await invoke("recent_import_jobs");
      if (!Array.isArray(result) || !result.every(isImportJobSummary)) {
        throw new Error("recent_import_jobs returned an invalid response");
      }
      return result;
    },
    async getMcpSetup() {
      const result = await invoke("get_mcp_setup");
      if (!isMcpSetupInfo(result)) throw new Error("get_mcp_setup returned an invalid response");
      return result;
    },
    async getAiCredentialStatus(providerId) {
      const result = await invoke("get_ai_credential_status", { providerId });
      if (!isAiCredentialStatus(result)) throw new Error("get_ai_credential_status returned an invalid response");
      return result;
    },
    async saveAiCredential(providerId, secret) {
      const result = await invoke("save_ai_credential", { providerId, secret });
      if (!isAiCredentialStatus(result)) throw new Error("save_ai_credential returned an invalid response");
      return result;
    },
    async testAiConnection(request) {
      const result = await invoke("test_ai_connection", { request });
      if (!isAiConnectionStatus(result)) throw new Error("test_ai_connection returned an invalid response");
      return result;
    },
    async cancelAiGeneration(taskId) {
      await invoke("cancel_ai_generation", { taskId });
    },
    optimizeAiPrompt(id, request) {
      return invoke("optimize_ai_prompt", { id, request });
    },
    generateAiDraft(request) {
      return invoke("generate_ai_draft", { request });
    },
  };
}

function isRedactedDiagnosticEvent(value: unknown): value is RedactedDiagnosticEvent {
  if (typeof value !== "object" || value === null) return false;
  const event = value as Record<string, unknown>;
  return typeof event.occurredAt === "string" && typeof event.event === "string" && typeof event.recommendation === "string";
}

function isAiConnectionStatus(value: unknown): value is AiConnectionStatus {
  return typeof value === "object" && value !== null && (value as Record<string, unknown>).connected === true;
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
    (item.sources === undefined || (Array.isArray(item.sources) && item.sources.every(isPromptSourceEvidence))) &&
    (item.applicableTools === undefined || (Array.isArray(item.applicableTools) && item.applicableTools.every((tool) => typeof tool === "string"))) &&
    (item.applicableModels === undefined || (Array.isArray(item.applicableModels) && item.applicableModels.every((model) => typeof model === "string"))) &&
    (item.rating === undefined || item.rating === null || typeof item.rating === "number") &&
    (item.useCount === undefined || (typeof item.useCount === "number" && item.useCount >= 0)) &&
    (item.lastUsedAt === undefined || typeof item.lastUsedAt === "string" || item.lastUsedAt === null) &&
    (item.importedAt === undefined || typeof item.importedAt === "string" || item.importedAt === null) &&
    (item.lastValidatedAt === undefined || typeof item.lastValidatedAt === "string" || item.lastValidatedAt === null) &&
    typeof item.favorite === "boolean" &&
    typeof item.createdAt === "string" &&
    typeof item.updatedAt === "string"
  );
}

function isSkillSource(value: unknown): value is SkillSource {
  if (typeof value !== "object" || value === null) return false;
  const source = value as Record<string, unknown>;
  return typeof source.kind === "string" && typeof source.location === "string"
    && (typeof source.revision === "string" || source.revision === null);
}

function isSkillListItem(value: unknown): value is SkillListItem {
  if (typeof value !== "object" || value === null) return false;
  const skill = value as Record<string, unknown>;
  return typeof skill.id === "string" && typeof skill.name === "string" && typeof skill.description === "string"
    && isSkillSource(skill.source) && Array.isArray(skill.risks) && skill.risks.every((risk) => typeof risk === "string")
    && ["pending_review", "approved", "rejected", "risk_pending_confirmation"].includes(skill.reviewStatus as string)
    && typeof skill.favorite === "boolean" && typeof skill.updatedAt === "string";
}

function isSkillDetail(value: unknown): value is SkillDetail {
  if (!isSkillListItem(value)) return false;
  const skill = value as Record<string, unknown>;
  return (typeof skill.reviewNotes === "string" || skill.reviewNotes === null)
    && typeof skill.skillMarkdown === "string" && typeof skill.contentHash === "string"
    && typeof skill.createdAt === "string" && Array.isArray(skill.files)
    && (skill.installation === null || isSkillInstallation(skill.installation))
    && skill.files.every((file) => typeof file === "object" && file !== null
      && typeof (file as Record<string, unknown>).relativePath === "string"
      && typeof (file as Record<string, unknown>).bytes === "number"
      && typeof (file as Record<string, unknown>).sha256 === "string"
      && typeof (file as Record<string, unknown>).kind === "string");
}

function isSkillInstallation(value: unknown): value is SkillInstallation {
  if (typeof value !== "object" || value === null) return false;
  const installation = value as Record<string, unknown>;
  return typeof installation.installPath === "string" && typeof installation.installedHash === "string"
    && (typeof installation.backupPath === "string" || installation.backupPath === null);
}

function isSkillInstallationVerification(value: unknown): value is SkillInstallationVerification {
  if (typeof value !== "object" || value === null) return false;
  const verification = value as Record<string, unknown>;
  return verification.state === "matching" || verification.state === "drifted" || verification.state === "unavailable";
}

function isPromptSourceEvidence(value: unknown): value is { kind: string; name: string; location: string | null; collectedAt: string } {
  if (typeof value !== "object" || value === null) return false;
  const source = value as Record<string, unknown>;
  return typeof source.kind === "string" && typeof source.name === "string"
    && (typeof source.location === "string" || source.location === null) && typeof source.collectedAt === "string"
    && (source.rawExcerpt === undefined || typeof source.rawExcerpt === "string" || source.rawExcerpt === null)
    && (source.importJobId === undefined || typeof source.importJobId === "string" || source.importJobId === null);
}

function isPromptUsageStats(value: unknown): value is { useCount: number; lastUsedAt: string | null } {
  if (typeof value !== "object" || value === null) return false;
  const stats = value as Record<string, unknown>;
  return typeof stats.useCount === "number" && stats.useCount >= 0
    && (typeof stats.lastUsedAt === "string" || stats.lastUsedAt === null);
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

function isBootstrapStatus(value: unknown): value is BootstrapStatus {
  if (typeof value !== "object" || value === null) return false;
  const status = value as Record<string, unknown>;
  return (status.state === "ready" || status.state === "recovery")
    && (typeof status.code === "string" || status.code === null)
    && (typeof status.safeMessage === "string" || status.safeMessage === null)
    && (typeof status.backupName === "string" || status.backupName === null);
}

function isDiagnosticsStatus(value: unknown): value is DiagnosticsStatus {
  if (typeof value !== "object" || value === null) return false;
  const status = value as Record<string, unknown>;
  return typeof status.databaseAvailable === "boolean"
    && typeof status.searchIndexConsistent === "boolean"
    && typeof status.mcpDatabaseAvailable === "boolean";
}

function isBackupInfo(value: unknown): value is BackupInfo {
  if (typeof value !== "object" || value === null) return false;
  const item = value as Record<string, unknown>;
  return typeof item.path === "string" && typeof item.byteLen === "number" && typeof item.schemaVersion === "number";
}

function isBackupRestorePreview(value: unknown): value is BackupRestorePreview {
  if (typeof value !== "object" || value === null) return false;
  const item = value as Record<string, unknown>;
  return typeof item.targetExists === "boolean" && typeof item.backupSchemaVersion === "number" && typeof item.backupByteLen === "number" && typeof item.promptCount === "number";
}

function isAiCredentialStatus(value: unknown): value is AiCredentialStatus {
  return typeof value === "object" && value !== null && typeof (value as Record<string, unknown>).configured === "boolean";
}

function isImportResult(value: unknown): value is ImportResult {
  return typeof value === "object" && value !== null
    && typeof (value as Record<string, unknown>).imported === "number"
    && typeof (value as Record<string, unknown>).skippedDuplicates === "number"
    && typeof (value as Record<string, unknown>).failed === "number";
}

function isImportJobSummary(value: unknown): value is ImportJobSummary {
  if (typeof value !== "object" || value === null) return false;
  const item = value as Record<string, unknown>;
  return typeof item.id === "string" && typeof item.sourceKind === "string"
    && (typeof item.sourcePath === "string" || item.sourcePath === null)
    && typeof item.status === "string" && typeof item.startedAt === "string"
    && (typeof item.completedAt === "string" || item.completedAt === null)
    && typeof item.imported === "number" && typeof item.skippedDuplicates === "number"
    && typeof item.failed === "number";
}

function isMcpSetupInfo(value: unknown): value is McpSetupInfo {
  if (typeof value !== "object" || value === null) return false;
  const item = value as Record<string, unknown>;
  return typeof item.databasePath === "string" && typeof item.databaseAvailable === "boolean"
    && typeof item.configuration === "string";
}
