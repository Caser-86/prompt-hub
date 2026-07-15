import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { DiagnosticsPanel } from "./DiagnosticsPanel";

it("shows redacted database, search index, and MCP health", () => {
  render(<DiagnosticsPanel diagnostics={{ databaseAvailable: true, searchIndexConsistent: true, mcpDatabaseAvailable: true }} importJobs={[]} status={null} />);

  expect(screen.getByText("数据库")).toBeVisible();
  expect(screen.getAllByText("可用")).toHaveLength(2);
  expect(screen.getByText("搜索索引")).toBeVisible();
  expect(screen.getByText("一致")).toBeVisible();
  expect(screen.getByText("MCP 数据库")).toBeVisible();
});
