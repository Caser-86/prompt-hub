import { useEffect, useState } from "react";
import type { AiConnectionRequest, AiGenerationRequest, ApplicationStatus, DiagnosticsStatus, ImportJobSummary, RedactedDiagnosticEvent } from "@prompt-hub/contracts";

import { DiagnosticsPanel } from "../diagnostics/DiagnosticsPanel";
import { OnboardingGuide } from "../onboarding/OnboardingGuide";
import { BackupSettings, type BackupInfo, type RestorePreviewInfo } from "./BackupSettings";
import { AiSettings } from "./AiSettings";
import { McpSettings } from "./McpSettings";
import { AiDraftGenerator } from "../ai/AiDraftGenerator";

export function SettingsPage({
  createBackup, previewRestore, restoreBackup, pruneLocalBackups, getApplicationStatus, getDiagnosticsStatus, getRedactedDiagnosticEvents, rebuildSearchIndex, getAiCredentialStatus, saveAiCredential, recentImportJobs, generateAiDraft, cancelAiGeneration, testAiConnection, getMcpSetup,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  pruneLocalBackups: (retain: number) => Promise<number>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
  getDiagnosticsStatus: () => Promise<DiagnosticsStatus>;
  getRedactedDiagnosticEvents: () => Promise<RedactedDiagnosticEvent[]>;
  rebuildSearchIndex: () => Promise<void>;
  getAiCredentialStatus: (providerId: string) => Promise<{ configured: boolean }>;
  saveAiCredential: (providerId: string, secret: string) => Promise<{ configured: boolean }>;
  recentImportJobs: () => Promise<ImportJobSummary[]>;
  generateAiDraft: (request: AiGenerationRequest) => Promise<unknown>;
  cancelAiGeneration: (taskId: string) => Promise<void>;
  testAiConnection: (request: AiConnectionRequest) => Promise<unknown>;
  getMcpSetup: () => Promise<import("@prompt-hub/contracts").McpSetupInfo>;
}) {
  const [status, setStatus] = useState<ApplicationStatus | null>(null);
  const [importJobs, setImportJobs] = useState<ImportJobSummary[] | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsStatus | null>(null);
  const [logs, setLogs] = useState<RedactedDiagnosticEvent[]>([]);
  useEffect(() => { void getApplicationStatus().then(setStatus).catch(() => setStatus(null)); }, [getApplicationStatus]);
  useEffect(() => { void recentImportJobs().then(setImportJobs).catch(() => setImportJobs([])); }, [recentImportJobs]);
  useEffect(() => { void getDiagnosticsStatus().then(setDiagnostics).catch(() => setDiagnostics(null)); }, [getDiagnosticsStatus]);
  useEffect(() => { void getRedactedDiagnosticEvents().then(setLogs).catch(() => setLogs([])); }, [getRedactedDiagnosticEvents]);
  return <><BackupSettings createBackup={createBackup} previewRestore={previewRestore} restoreBackup={restoreBackup} pruneBackups={pruneLocalBackups} /><AiSettings getStatus={getAiCredentialStatus} saveCredential={saveAiCredential} /><AiDraftGenerator cancelGeneration={cancelAiGeneration} generateDraft={generateAiDraft} testConnection={testAiConnection} /><McpSettings getSetup={getMcpSetup} /><DiagnosticsPanel diagnostics={diagnostics} importJobs={importJobs} logs={logs} rebuildSearchIndex={rebuildSearchIndex} status={status} /><OnboardingGuide /></>;
}
