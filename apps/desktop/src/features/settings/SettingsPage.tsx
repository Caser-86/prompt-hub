import { useEffect, useState } from "react";
import type { AiConnectionRequest, AiGenerationRequest, ApplicationStatus, DiagnosticsStatus, ImportJobSummary } from "@prompt-hub/contracts";

import { DiagnosticsPanel } from "../diagnostics/DiagnosticsPanel";
import { OnboardingGuide } from "../onboarding/OnboardingGuide";
import { BackupSettings, type BackupInfo, type RestorePreviewInfo } from "./BackupSettings";
import { AiSettings } from "./AiSettings";
import { McpSettings } from "./McpSettings";
import { AiDraftGenerator } from "../ai/AiDraftGenerator";

export function SettingsPage({
  createBackup, previewRestore, restoreBackup, pruneLocalBackups, getApplicationStatus, getDiagnosticsStatus, rebuildSearchIndex, getAiCredentialStatus, saveAiCredential, recentImportJobs, generateAiDraft, testAiConnection, getMcpSetup,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  pruneLocalBackups: (retain: number) => Promise<number>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
  getDiagnosticsStatus: () => Promise<DiagnosticsStatus>;
  rebuildSearchIndex: () => Promise<void>;
  getAiCredentialStatus: (providerId: string) => Promise<{ configured: boolean }>;
  saveAiCredential: (providerId: string, secret: string) => Promise<{ configured: boolean }>;
  recentImportJobs: () => Promise<ImportJobSummary[]>;
  generateAiDraft: (request: AiGenerationRequest) => Promise<unknown>;
  testAiConnection: (request: AiConnectionRequest) => Promise<unknown>;
  getMcpSetup: () => Promise<import("@prompt-hub/contracts").McpSetupInfo>;
}) {
  const [status, setStatus] = useState<ApplicationStatus | null>(null);
  const [importJobs, setImportJobs] = useState<ImportJobSummary[] | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsStatus | null>(null);
  useEffect(() => { void getApplicationStatus().then(setStatus).catch(() => setStatus(null)); }, [getApplicationStatus]);
  useEffect(() => { void recentImportJobs().then(setImportJobs).catch(() => setImportJobs([])); }, [recentImportJobs]);
  useEffect(() => { void getDiagnosticsStatus().then(setDiagnostics).catch(() => setDiagnostics(null)); }, [getDiagnosticsStatus]);
  return <><BackupSettings createBackup={createBackup} previewRestore={previewRestore} restoreBackup={restoreBackup} pruneBackups={pruneLocalBackups} /><AiSettings getStatus={getAiCredentialStatus} saveCredential={saveAiCredential} /><AiDraftGenerator generateDraft={generateAiDraft} testConnection={testAiConnection} /><McpSettings getSetup={getMcpSetup} /><DiagnosticsPanel diagnostics={diagnostics} importJobs={importJobs} rebuildSearchIndex={rebuildSearchIndex} status={status} /><OnboardingGuide /></>;
}
