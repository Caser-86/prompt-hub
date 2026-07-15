import { expect, test } from "@playwright/test";

test("shows first-run privacy, backup, and MCP guidance", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "设置" }).click();

  await expect(page.getByRole("heading", { name: "备份与恢复" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "诊断信息" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "开始使用" })).toBeVisible();
  await expect(page.getByText(/提示词库与备份保存在此设备/)).toBeVisible();
  await expect(page.getByText(/MCP 仅能读取内容或创建收件箱草稿/)).toBeVisible();
});
