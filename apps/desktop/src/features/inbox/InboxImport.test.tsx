import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { InboxImport } from "./InboxImport";

describe("InboxImport", () => {
  it("shows a retry action when a local file import fails", async () => {
    const importFile = vi.fn().mockRejectedValue(new Error("invalid file"));
    render(
      <InboxImport
        importFile={importFile}
        loadPrompts={vi.fn().mockResolvedValue([])}
        onReview={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByLabelText("文件路径"), { target: { value: "C:\\导入\\损坏.json" } });
    fireEvent.click(screen.getByRole("button", { name: "导入到收件箱" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("无法导入文件");
    fireEvent.click(screen.getByRole("button", { name: "重试导入" }));
    await waitFor(() => expect(importFile).toHaveBeenCalledTimes(2));
  });
});
