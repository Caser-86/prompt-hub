import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptLifecycleActions } from "./PromptLifecycleActions";

describe("PromptLifecycleActions", () => {
  it("requires confirmation before soft deletion and exposes recovery after deletion", async () => {
    const archive = vi.fn().mockResolvedValue(undefined);
    const softDelete = vi.fn().mockResolvedValue(undefined);
    const recover = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptLifecycleActions
        archive={archive}
        initialStatus="published"
        recover={recover}
        softDelete={softDelete}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "软删除提示词" }));
    expect(softDelete).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "确认软删除" }));
    await waitFor(() => expect(softDelete).toHaveBeenCalledOnce());
    expect(screen.getByRole("button", { name: "恢复提示词" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "恢复提示词" }));
    await waitFor(() => expect(recover).toHaveBeenCalledOnce());
    expect(screen.getByRole("button", { name: "归档提示词" })).toBeVisible();
  });
});
