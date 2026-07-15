import { useEffect, useState } from "react";
import type { ApplicationStatus, ImportJobSummary } from "@prompt-hub/contracts";

import { DiagnosticsPanel } from "../diagnostics/DiagnosticsPanel";
import { OnboardingGuide } from "../onboarding/OnboardingGuide";
import { BackupSettings, type BackupInfo, type RestorePreviewInfo } from "./BackupSettings";
import { AiSettings } from "./AiSettings";

export function SettingsPage({
  createBackup, previewRestore, restoreBackup, getApplicationStatus, getAiCredentialStatus, saveAiCredential, recentImportJobs,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
  getAiCredentialStatus: (providerId: string) => Promise<{ configured: boolean }>;
  saveAiCredential: (providerId: string, secret: string) => Promise<{ configured: boolean }>;
  recentImportJobs: () => Promise<ImportJobSummary[]>;
}) {
  const [status, setStatus] = useState<ApplicationStatus | null>(null);
  const [importJobs, setImportJobs] = useState<ImportJobSummary[] | null>(null);
  useEffect(() => { void getApplicationStatus().then(setStatus).catch(() => setStatus(null)); }, [getApplicationStatus]);
  useEffect(() => { void recentImportJobs().then(setImportJobs).catch(() => setImportJobs([])); }, [recentImportJobs]);
  return <><BackupSettings createBackup={createBackup} previewRestore={previewRestore} restoreBackup={restoreBackup} /><AiSettings getStatus={getAiCredentialStatus} saveCredential={saveAiCredential} /><DiagnosticsPanel importJobs={importJobs} status={status} /><OnboardingGuide /></>;
}
