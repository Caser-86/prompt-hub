import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptMetadataEditor } from "./PromptMetadataEditor";

describe("PromptMetadataEditor", () => {
  it("saves tool/model compatibility and an effectiveness rating", async () => {
    const saveMetadata = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptMetadataEditor
        promptId="prompt-1"
        saveMetadata={saveMetadata}
      />,
    );

    fireEvent.change(screen.getByLabelText("适用工具"), { target: { value: "Codex" } });
    fireEvent.change(screen.getByLabelText("适用模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("兼容性状态"), { target: { value: "confirmed" } });
    fireEvent.change(screen.getByLabelText("有效性"), { target: { value: "effective" } });
    fireEvent.change(screen.getByLabelText("评分"), { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "保存元数据" }));

    await waitFor(() => expect(saveMetadata).toHaveBeenCalledWith("prompt-1", {
      tool: "Codex",
      model: "gpt-5",
      compatibilityStatus: "confirmed",
      effectiveness: "effective",
      rating: 5,
      notes: null,
    }));
  });

  it("hydrates existing metadata and does not overwrite it with blank defaults", async () => {
    const saveMetadata = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptMetadataEditor
        initial={{ tool: "Codex", model: "gpt-5", compatibilityStatus: "confirmed", effectiveness: "effective", rating: 4 }}
        promptId="prompt-1"
        saveMetadata={saveMetadata}
      />,
    );

    expect(screen.getByLabelText("适用工具")).toHaveValue("Codex");
    expect(screen.getByLabelText("适用模型")).toHaveValue("gpt-5");
    expect(screen.getByLabelText("兼容性状态")).toHaveValue("confirmed");
    expect(screen.getByLabelText("有效性")).toHaveValue("effective");
    expect(screen.getByLabelText("评分")).toHaveValue(4);
    fireEvent.click(screen.getByRole("button", { name: "保存元数据" }));

    await waitFor(() => expect(saveMetadata).toHaveBeenCalledWith("prompt-1", expect.objectContaining({
      tool: "Codex", model: "gpt-5", compatibilityStatus: "confirmed", effectiveness: "effective", rating: 4,
    })));
  });

  it("shows a retryable error when the atomic metadata save fails", async () => {
    const saveMetadata = vi.fn().mockRejectedValue(new Error("offline"));
    render(<PromptMetadataEditor promptId="prompt-1" saveMetadata={saveMetadata} />);

    fireEvent.click(screen.getByRole("button", { name: "保存元数据" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/无法保存元数据.*重试/);
  });
});
