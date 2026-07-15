import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AiSettings } from "./AiSettings";

describe("AiSettings", () => {
  it("shows only configuration state and clears the submitted secret", async () => {
    const saveCredential = vi.fn().mockResolvedValue({ configured: true });
    render(<AiSettings getStatus={vi.fn().mockResolvedValue({ configured: false })} saveCredential={saveCredential} />);
    expect(await screen.findByText("尚未配置 AI 凭据")).toBeVisible();
    const input = screen.getByLabelText("OpenAI 兼容 API 密钥");
    fireEvent.change(input, { target: { value: "do-not-display" } });
    fireEvent.click(screen.getByRole("button", { name: "安全保存密钥" }));
    await waitFor(() => expect(saveCredential).toHaveBeenCalledWith("openai-compatible", "do-not-display"));
    expect(input).toHaveValue("");
    expect(screen.getByText("已配置 AI 凭据")).toBeVisible();
  });
});
