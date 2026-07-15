import { useEffect, useState } from "react";
import type { ApplicationStatus } from "@prompt-hub/contracts";

import { DiagnosticsPanel } from "../diagnostics/DiagnosticsPanel";
import { OnboardingGuide } from "../onboarding/OnboardingGuide";
import { BackupSettings, type BackupInfo, type RestorePreviewInfo } from "./BackupSettings";
import { AiSettings } from "./AiSettings";

export function SettingsPage({
  createBackup, previewRestore, restoreBackup, getApplicationStatus, getAiCredentialStatus, saveAiCredential,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  restoreBackup: (path: string) => Promise<BackupInfo>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
  getAiCredentialStatus: (providerId: string) => Promise<{ configured: boolean }>;
  saveAiCredential: (providerId: string, secret: string) => Promise<{ configured: boolean }>;
}) {
  const [status, setStatus] = useState<ApplicationStatus | null>(null);
  useEffect(() => { void getApplicationStatus().then(setStatus).catch(() => setStatus(null)); }, [getApplicationStatus]);
  return <><BackupSettings createBackup={createBackup} previewRestore={previewRestore} restoreBackup={restoreBackup} /><AiSettings getStatus={getAiCredentialStatus} saveCredential={saveAiCredential} /><DiagnosticsPanel status={status} /><OnboardingGuide /></>;
}
