import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptLibrary } from "./PromptLibrary";

describe("PromptLibrary", () => {
  it("shows an accessible empty library with a creation entry point", async () => {
    render(<PromptLibrary loadPrompts={async () => []} />);

    expect(await screen.findByRole("heading", { name: "提示词库" })).toBeVisible();
    expect(screen.getByRole("button", { name: "创建提示词" })).toBeVisible();
    expect(screen.getByText("还没有提示词资产")).toBeVisible();
  });

  it("opens the editor from the creation entry point", async () => {
    const onCreate = vi.fn();
    render(<PromptLibrary loadPrompts={async () => []} onCreate={onCreate} />);

    fireEvent.click(await screen.findByRole("button", { name: "创建提示词" }));
    expect(onCreate).toHaveBeenCalledOnce();
  });
});
