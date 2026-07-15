import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AiDraftGenerator } from "./AiDraftGenerator";

describe("AiDraftGenerator", () => {
  it("restores non-sensitive endpoint and model preferences locally", () => {
    localStorage.setItem("prompt-hub.ai.draft-settings", JSON.stringify({ endpoint: "https://api.example.com", model: "gpt-test" }));
    render(<AiDraftGenerator generateDraft={vi.fn()} />);

    expect(screen.getByLabelText("兼容 API 地址")).toHaveValue("https://api.example.com");
    expect(screen.getByLabelText("模型")).toHaveValue("gpt-test");
  });

  it("sends generation output only to the draft command", async () => {
    const generateDraft = vi.fn().mockResolvedValue({ id: "inbox-draft" });
    render(<AiDraftGenerator generateDraft={generateDraft} />);
    fireEvent.change(screen.getByLabelText("模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("生成指令"), { target: { value: "优化提示词" } });
    fireEvent.change(screen.getByLabelText("输入摘要"), { target: { value: "保留结构" } });
    fireEvent.click(screen.getByRole("button", { name: "生成收件箱草稿" }));
    await waitFor(() => expect(generateDraft).toHaveBeenCalledWith(expect.objectContaining({ providerId: "openai-compatible", model: "gpt-5" })));
    expect(screen.getByRole("status")).toHaveTextContent("草稿已创建到收件箱");
  });
});
