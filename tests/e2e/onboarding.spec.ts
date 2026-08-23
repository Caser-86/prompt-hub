import { expect, test } from "@playwright/test";

test("shows first-run privacy, backup, and MCP guidance", async ({ page }) => {
  await page.addInitScript(() => {
    const invoke = async (command: string) => {
      if (command === "get_bootstrap_status" || command === "retry_database_bootstrap") return { state: "ready", code: null, safeMessage: null, backupName: null };
      if (command === "export_bootstrap_diagnostics") return '{"state":"ready"}';
      if (command === "get_application_status") return { appVersion: "0.1.0", databaseSchemaVersion: 7, offlineCapable: true };
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

  await expect(page.getByRole("heading", { name: "备份与恢复" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "诊断信息" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "开始使用" })).toBeVisible();
  await expect(page.getByText(/提示词库与备份保存在此设备/)).toBeVisible();
  await expect(page.getByText(/MCP 仅能读取内容或创建收件箱草稿/)).toBeVisible();
});
