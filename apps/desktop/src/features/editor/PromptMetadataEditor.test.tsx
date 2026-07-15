import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptMetadataEditor } from "./PromptMetadataEditor";

describe("PromptMetadataEditor", () => {
  it("saves tool/model compatibility and an effectiveness rating", async () => {
    const saveCompatibility = vi.fn().mockResolvedValue(undefined);
    const saveValidation = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptMetadataEditor
        promptId="prompt-1"
        saveCompatibility={saveCompatibility}
        saveValidation={saveValidation}
      />,
    );

    fireEvent.change(screen.getByLabelText("适用工具"), { target: { value: "Codex" } });
    fireEvent.change(screen.getByLabelText("适用模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("有效性"), { target: { value: "effective" } });
    fireEvent.change(screen.getByLabelText("评分"), { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "保存元数据" }));

    await waitFor(() => expect(saveCompatibility).toHaveBeenCalledWith("prompt-1", {
      tool: "Codex",
      model: "gpt-5",
      status: "confirmed",
      notes: null,
    }));
    expect(saveValidation).toHaveBeenCalledWith("prompt-1", {
      status: "effective",
      rating: 5,
      notes: null,
    });
  });
});
