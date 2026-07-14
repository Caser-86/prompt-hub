import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { App } from "./App";

describe("App", () => {
  it("identifies the production application", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Prompt Hub" })).toBeVisible();
    expect(screen.getByText("本地优先的提示词资产管理工具")).toBeVisible();
  });
});
