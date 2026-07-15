import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { DiagnosticsPanel } from "./DiagnosticsPanel";

it("shows redacted database, search index, and MCP health", () => {
  render(<DiagnosticsPanel diagnostics={{ databaseAvailable: true, searchIndexConsistent: true, mcpDatabaseAvailable: true }} importJobs={[]} status={null} />);

  expect(screen.getByText("数据库")).toBeVisible();
  expect(screen.getAllByText("可用")).toHaveLength(2);
  expect(screen.getByText("搜索索引")).toBeVisible();
  expect(screen.getByText("一致")).toBeVisible();
  expect(screen.getByText("MCP 数据库")).toBeVisible();
});

it("renders only redacted diagnostic event summaries", () => {
  render(<DiagnosticsPanel diagnostics={null} importJobs={[]} logs={[{ occurredAt: "2026-07-15T00:00:00Z", event: "database_unavailable", recommendation: "检查数据目录权限" }]} status={null} />);
  expect(screen.getByText("本地诊断日志")).toBeVisible();
  expect(screen.getByText(/database_unavailable/)).toBeVisible();
  expect(screen.getByText(/检查数据目录权限/)).toBeVisible();
});

it("rebuilds an inconsistent index through the supplied command", async () => {
  const rebuildSearchIndex = vi.fn().mockResolvedValue(undefined);
  render(<DiagnosticsPanel diagnostics={{ databaseAvailable: true, searchIndexConsistent: false, mcpDatabaseAvailable: true }} importJobs={[]} rebuildSearchIndex={rebuildSearchIndex} status={null} />);

  fireEvent.click(screen.getByRole("button", { name: "重建搜索索引" }));
  await waitFor(() => expect(rebuildSearchIndex).toHaveBeenCalledOnce());
});
