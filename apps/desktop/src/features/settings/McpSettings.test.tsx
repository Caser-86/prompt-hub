import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { McpSettings } from "./McpSettings";

describe("McpSettings", () => {
  it("shows generated local Codex configuration without prompt content", async () => {
    render(<McpSettings getSetup={vi.fn().mockResolvedValue({ databasePath: "C:/data/prompt-hub.db", databaseAvailable: true, configuration: "{\"mcpServers\":{}}" })} />);
    expect(await screen.findByRole("status")).toHaveTextContent("可用");
    expect(screen.getByLabelText("Codex MCP 配置")).toHaveTextContent("mcpServers");
  });
});
