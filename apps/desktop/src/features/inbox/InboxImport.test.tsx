import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { InboxImport } from "./InboxImport";

describe("InboxImport", () => {
  it("shows a retry action when a local file import fails", async () => {
    const importFile = vi.fn().mockRejectedValue(new Error("invalid file"));
    render(
      <InboxImport
        importFile={importFile}
        importFolder={vi.fn().mockResolvedValue({ imported: 0, skippedDuplicates: 0, failed: 0 })}
        loadPrompts={vi.fn().mockResolvedValue([])}
        onReview={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByLabelText("文件路径"), { target: { value: "C:\\导入\\损坏.json" } });
    fireEvent.click(screen.getByRole("button", { name: "导入到收件箱" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("无法导入内容");
    fireEvent.click(screen.getByRole("button", { name: "重试导入" }));
    await waitFor(() => expect(importFile).toHaveBeenCalledTimes(2));
  });

  it("scans a selected local folder into the inbox", async () => {
    const importFolder = vi.fn().mockResolvedValue({ imported: 2, skippedDuplicates: 1, failed: 0 });
    render(
      <InboxImport
        importFile={vi.fn()}
        importFolder={importFolder}
        loadPrompts={vi.fn().mockResolvedValue([])}
        onReview={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByLabelText("文件夹路径"), { target: { value: "C:\\提示词" } });
    fireEvent.click(screen.getByRole("button", { name: "扫描文件夹到收件箱" }));

    await waitFor(() => expect(importFolder).toHaveBeenCalledWith("C:\\提示词"));
    expect(screen.getByRole("status")).toHaveTextContent("已创建 2 条待审核草稿");
  });
});
