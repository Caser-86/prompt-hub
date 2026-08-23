import { expect, test } from "@playwright/test";

test("creates, reviews, and searches a local prompt through the desktop command boundary", async ({ page }) => {
  await page.addInitScript(() => {
    const prompts: Array<Record<string, unknown>> = [];
    const history = new Map<string, Array<Record<string, unknown>>>();
    const timestamp = "2026-07-15T00:00:00Z";
    const invoke = async (command: string, args?: Record<string, unknown>) => {
      if (command === "list_prompts") return prompts;
      if (command === "create_manual_prompt_draft") {
        const draft = args?.draft as { title: string; body: string; category: string | null };
        const id = `prompt-${prompts.length + 1}`;
        const prompt = {
          id, title: draft.title, status: "inbox", effectiveness: "unverified", category: draft.category,
          tags: [], sourceNames: ["手动录入"], favorite: false, createdAt: timestamp, updatedAt: timestamp,
          applicableTools: [], applicableModels: [], rating: null,
        };
        prompts.push(prompt);
        history.set(id, [{ number: 1, body: draft.body, createdAt: timestamp }]);
        return prompt;
      }
      if (command === "prompt_history") return history.get(args?.id as string) ?? [];
      if (command === "search_prompts") {
        const text = String(args?.text ?? "").toLowerCase();
        const hits = prompts.filter((prompt) => String(prompt.title).toLowerCase().includes(text)).map((prompt) => ({
          id: prompt.id, title: prompt.title, snippet: "本地草稿", status: prompt.status,
          effectiveness: prompt.effectiveness, rating: prompt.rating, updatedAt: prompt.updatedAt,
        }));
        return { total: hits.length, hits };
      }
      if (command === "get_bootstrap_status" || command === "retry_database_bootstrap") return { state: "ready", code: null, safeMessage: null, backupName: null };
      if (command === "export_bootstrap_diagnostics") return '{"state":"ready"}';
      if (command === "get_application_status") return { appVersion: "0.1.0", databaseSchemaVersion: 2, offlineCapable: true };
      if (command === "get_diagnostics_status") return { databaseAvailable: true, searchIndexConsistent: true, mcpDatabaseAvailable: true };
      if (command === "get_redacted_diagnostic_events" || command === "recent_import_jobs") return [];
      if (command === "get_ai_credential_status") return { configured: false };
      if (command === "get_mcp_setup") return { databasePath: "C:/data/prompt-hub.db", databaseAvailable: true, configuration: "{}" };
      return undefined;
    };
    (window as unknown as { __TAURI_INTERNALS__: { invoke: typeof invoke } }).__TAURI_INTERNALS__ = { invoke };
  });

  await page.goto("/");
  await page.getByRole("button", { name: "创建提示词" }).click();
  await page.getByLabel("标题").fill("本地代码审查");
  await page.getByRole("textbox", { name: "正文" }).fill("请审查 {{language}} 代码");
  await page.getByLabel("分类").fill("开发");
  await page.getByRole("button", { name: "保存到收件箱" }).click();

  await page.getByRole("button", { name: "打开提示词：本地代码审查" }).click();
  await expect(page.getByRole("heading", { name: "本地代码审查" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "版本历史" })).toBeVisible();

  await page.getByRole("button", { name: "返回" }).click();
  await page.getByRole("button", { name: "高级筛选" }).click();
  await page.getByRole("searchbox", { name: "搜索提示词" }).fill("代码审查");
  await expect(page.getByRole("heading", { name: "本地代码审查" })).toBeVisible();
});
