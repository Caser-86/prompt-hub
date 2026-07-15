import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptHistory } from "./PromptHistory";

describe("PromptHistory", () => {
  it("compares two immutable versions and restores only after confirmation", async () => {
    const restoreVersion = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptHistory
        history={[
          { number: 1, body: "第一版正文", createdAt: "2026-07-15T00:00:00Z" },
          { number: 2, body: "第二版正文", createdAt: "2026-07-15T00:01:00Z" },
        ]}
        restoreVersion={restoreVersion}
      />,
    );

    expect(screen.getByText("第一版正文")).toBeVisible();
    expect(screen.getByText("第二版正文")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "恢复版本 1" }));
    expect(restoreVersion).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "确认恢复版本 1" }));
    await waitFor(() => expect(restoreVersion).toHaveBeenCalledWith(1));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("shows an escaped line diff against the current version", () => {
    render(
      <PromptHistory
        history={[
          { number: 1, body: "保留行\n删除行\n<script>unsafe</script>", createdAt: "2026-07-15T00:00:00Z" },
          { number: 2, body: "保留行\n新增行\n<script>unsafe</script>", createdAt: "2026-07-15T00:01:00Z" },
        ]}
        restoreVersion={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    const diff = screen.getByLabelText("版本正文差异");
    expect(diff).toHaveTextContent("- 删除行");
    expect(diff).toHaveTextContent("+ 新增行");
    expect(diff).toHaveTextContent("<script>unsafe</script>");
    expect(diff.querySelector("script")).toBeNull();
  });
});
