import { expect, test } from "@playwright/test";

test("previews a verified backup before restoring and shows its safety copy", async ({ page }) => {
  await page.addInitScript(() => {
    const invoke = async (command: string) => {
      if (command === "get_bootstrap_status" || command === "retry_database_bootstrap") return { state: "ready", code: null, safeMessage: null, backupName: null };
      if (command === "export_bootstrap_diagnostics") return '{"state":"ready"}';
      if (command === "preview_backup_restore") {
        return { targetExists: true, backupSchemaVersion: 4, backupByteLen: 2048, promptCount: 3 };
      }
      if (command === "restore_backup") {
        return { path: "C:/data/backups/pre-restore.db", byteLen: 2048, schemaVersion: 4 };
      }
      if (command === "get_application_status") return { appVersion: "0.1.0", databaseSchemaVersion: 4, offlineCapable: true };
      if (command === "get_diagnostics_status") return { databaseAvailable: true, searchIndexConsistent: true, mcpDatabaseAvailable: true };
      if (command === "get_redacted_diagnostic_events" || command === "recent_import_jobs") return [];
      if (command === "get_ai_credential_status") return { configured: false };
      if (command === "get_mcp_setup") return { databasePath: "C:/data/prompt-hub.db", databaseAvailable: true, configuration: "{}" };
      return undefined;
    };
    (window as unknown as { __TAURI_INTERNALS__: { invoke: typeof invoke } }).__TAURI_INTERNALS__ = { invoke };
  });

  await page.goto("/");
  await page.getByRole("link", { name: "设置" }).click();
  await page.getByLabel("备份文件路径").fill("C:/data/backups/library.db");
  await page.getByRole("button", { name: "检查恢复内容" }).click();
  await expect(page.getByText(/备份包含 3 条提示词/)).toBeVisible();
  await page.getByRole("button", { name: "确认恢复备份" }).click();
  await expect(page.getByText(/恢复前安全备份：C:\/data\/backups\/pre-restore.db/)).toBeVisible();
});
