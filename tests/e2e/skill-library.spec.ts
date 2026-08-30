import { expect, test } from "@playwright/test";

test("collects, reviews, and explicitly replaces a Skill without executing its content", async ({ page }) => {
  await page.addInitScript(() => {
    const timestamp = "2026-08-29T00:00:00Z";
    const summary = {
      id: "skill-1",
      name: "reviewed-skill",
      description: "A Skill that must be reviewed",
      source: { kind: "local_directory", location: "C:/skills/reviewed-skill", revision: null },
      risks: ["contains_script"],
      reviewStatus: "pending_review",
      favorite: false,
      updatedAt: timestamp,
    };
    const detail = {
      ...summary,
      reviewNotes: null,
      skillMarkdown: "# Reviewed Skill\n<img src=x onerror=alert('executed')>\n",
      files: [
        { relativePath: "SKILL.md", bytes: 64, sha256: "a".repeat(64), kind: "skill_markdown" },
        { relativePath: "scripts/run.cmd", bytes: 12, sha256: "b".repeat(64), kind: "script" },
      ],
      contentHash: "c".repeat(64),
      createdAt: timestamp,
    };
    let collected = false;
    const invoke = async (command: string, args?: Record<string, unknown>) => {
      if (command === "get_bootstrap_status") return { state: "ready", code: null, safeMessage: null, backupName: null };
      if (command === "migrate_legacy_prompt_usage") return undefined;
      if (command === "list_prompts") return [];
      if (command === "list_skills") return collected ? [summary] : [];
      if (command === "collect_skill_folder") { collected = true; return summary; }
      if (command === "get_skill") return detail;
      if (command === "review_skill") {
        const review = args?.review as { status: string };
        summary.reviewStatus = review.status;
        detail.reviewStatus = review.status;
        return undefined;
      }
      if (command === "install_skill") {
        const installation = args?.installation as { replaceAfterBackup: boolean };
        if (!installation.replaceAfterBackup) throw new Error("destination exists");
        return {
          installPath: "C:/codex/skills/reviewed-skill",
          backupPath: "C:/backups/reviewed-skill-old",
          installedHash: detail.contentHash,
        };
      }
      if (command === "verify_skill_installation") return { state: "matching" };
      return undefined;
    };
    (window as unknown as { __TAURI_INTERNALS__: { invoke: typeof invoke } }).__TAURI_INTERNALS__ = { invoke };
  });

  await page.goto("/");
  await page.getByRole("link", { name: "Skill 库" }).click();
  await page.getByLabel("Skill 文件夹路径").fill("C:/skills/reviewed-skill");
  await page.getByRole("button", { name: "扫描本地 Skill" }).click();
  await page.getByRole("button", { name: "打开 Skill：reviewed-skill" }).click();

  await expect(page.getByText("含脚本")).toBeVisible();
  await expect(page.getByLabel("Skill 正文")).toContainText("<img src=x onerror=alert('executed')>");
  await expect(page.locator(".skill-markdown-card img")).toHaveCount(0);
  await page.getByRole("button", { name: "审核通过" }).click();

  await page.getByLabel("目标目录").fill("C:/codex/skills");
  await page.getByRole("button", { name: "安装 Skill" }).click();
  await expect(page.getByRole("alert")).toContainText("同名冲突");
  await page.getByLabel("同名时先备份再替换").check();
  let confirmationSeen = false;
  page.once("dialog", async (dialog) => {
    confirmationSeen = true;
    expect(dialog.message()).toContain("备份");
    await dialog.accept();
  });
  await page.getByRole("button", { name: "安装 Skill" }).click();
  await expect.poll(() => confirmationSeen).toBe(true);
  await expect(page.getByText(/原版本已备份/)).toBeVisible();
  await page.getByRole("button", { name: "检查本地漂移" }).click();
  await expect(page.getByText("安装内容一致")).toBeVisible();
});
