import { useEffect, useState } from "react";
import type { ApplicationStatus } from "@prompt-hub/contracts";

import { DiagnosticsPanel } from "../diagnostics/DiagnosticsPanel";
import { OnboardingGuide } from "../onboarding/OnboardingGuide";
import { BackupSettings, type BackupInfo, type RestorePreviewInfo } from "./BackupSettings";

export function SettingsPage({
  createBackup, previewRestore, getApplicationStatus,
}: {
  createBackup: () => Promise<BackupInfo>;
  previewRestore: (path: string) => Promise<RestorePreviewInfo>;
  getApplicationStatus: () => Promise<ApplicationStatus>;
}) {
  const [status, setStatus] = useState<ApplicationStatus | null>(null);
  useEffect(() => { void getApplicationStatus().then(setStatus).catch(() => setStatus(null)); }, [getApplicationStatus]);
  return <><BackupSettings createBackup={createBackup} previewRestore={previewRestore} /><DiagnosticsPanel status={status} /><OnboardingGuide /></>;
}
