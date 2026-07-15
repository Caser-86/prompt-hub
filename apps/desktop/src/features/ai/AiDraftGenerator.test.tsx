import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AiDraftGenerator } from "./AiDraftGenerator";

describe("AiDraftGenerator", () => {
  beforeEach(() => localStorage.clear());

  it("restores non-sensitive endpoint and model preferences locally", () => {
    localStorage.setItem("prompt-hub.ai.draft-settings", JSON.stringify({ endpoint: "https://api.example.com", model: "gpt-test" }));
    render(<AiDraftGenerator generateDraft={vi.fn()} testConnection={vi.fn()} />);

    expect(screen.getByLabelText("兼容 API 地址")).toHaveValue("https://api.example.com");
    expect(screen.getByLabelText("模型")).toHaveValue("gpt-test");
  });

  it("sends generation output only to the draft command", async () => {
    const generateDraft = vi.fn().mockResolvedValue({ id: "inbox-draft" });
    render(<AiDraftGenerator generateDraft={generateDraft} testConnection={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("生成指令"), { target: { value: "优化提示词" } });
    fireEvent.change(screen.getByLabelText("输入摘要"), { target: { value: "保留结构" } });
    fireEvent.click(screen.getByRole("button", { name: "生成收件箱草稿" }));
    await waitFor(() => expect(generateDraft).toHaveBeenCalledWith(expect.objectContaining({ providerId: "openai-compatible", model: "gpt-5" })));
    expect(screen.getByRole("status")).toHaveTextContent("草稿已创建到收件箱");
  });

  it("tests the configured connection without creating a draft", async () => {
    const generateDraft = vi.fn();
    const testConnection = vi.fn().mockResolvedValue({ connected: true });
    render(<AiDraftGenerator generateDraft={generateDraft} testConnection={testConnection} />);

    fireEvent.click(screen.getByRole("button", { name: "测试连接" }));

    await waitFor(() => expect(testConnection).toHaveBeenCalledWith({
      endpoint: "https://api.openai.com",
      providerId: "openai-compatible",
      model: "",
    }));
    expect(generateDraft).not.toHaveBeenCalled();
    expect(screen.getByRole("status")).toHaveTextContent("连接测试成功");
  });

  it("cancels the active generation while preserving the user's input", async () => {
    let completeGeneration: (() => void) | undefined;
    const generateDraft = vi.fn(() => new Promise<void>((resolve) => { completeGeneration = resolve; }));
    const cancelGeneration = vi.fn().mockResolvedValue(undefined);
    render(<AiDraftGenerator cancelGeneration={cancelGeneration} generateDraft={generateDraft} testConnection={vi.fn()} />);

    fireEvent.change(screen.getByLabelText("模型"), { target: { value: "gpt-5" } });
    fireEvent.change(screen.getByLabelText("生成指令"), { target: { value: "优化提示词" } });
    fireEvent.change(screen.getByLabelText("输入摘要"), { target: { value: "保留结构" } });
    fireEvent.click(screen.getByRole("button", { name: "生成收件箱草稿" }));
    await waitFor(() => expect(generateDraft).toHaveBeenCalledOnce());

    fireEvent.click(screen.getByRole("button", { name: "取消生成" }));
    await waitFor(() => expect(cancelGeneration).toHaveBeenCalledWith(expect.any(String)));
    expect(screen.getByLabelText("生成指令")).toHaveValue("优化提示词");
    completeGeneration?.();
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("生成已取消"));
  });
});
