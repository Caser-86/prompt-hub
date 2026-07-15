import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AiOptimizationReview } from "./AiOptimizationReview";

describe("AiOptimizationReview", () => {
  it("creates an optimization request and safely shows the candidate text", async () => {
    const optimize = vi.fn().mockResolvedValue({ body: "优化后的正文" });
    render(<AiOptimizationReview body={"原正文"} optimize={optimize} promptId="prompt-1" />);
    fireEvent.change(screen.getByLabelText("优化指令"), { target: { value: "更清晰" } });
    fireEvent.click(screen.getByRole("button", { name: "生成优化草稿" }));
    await waitFor(() => expect(optimize).toHaveBeenCalledWith("prompt-1", "更清晰", expect.any(String)));
    expect(screen.getByLabelText("优化前后正文差异")).toHaveTextContent("优化后的正文");
  });

  it("cancels an active optimization and preserves the instruction", async () => {
    let finish: (() => void) | undefined;
    const optimize = vi.fn(() => new Promise<{ body: string }>((resolve) => { finish = () => resolve({ body: "不应显示" }); }));
    const cancel = vi.fn().mockResolvedValue(undefined);
    render(<AiOptimizationReview body="原正文" cancel={cancel} optimize={optimize} promptId="prompt-1" />);
    fireEvent.change(screen.getByLabelText("优化指令"), { target: { value: "更清晰" } });
    fireEvent.click(screen.getByRole("button", { name: "生成优化草稿" }));
    await waitFor(() => expect(optimize).toHaveBeenCalledOnce());

    fireEvent.click(screen.getByRole("button", { name: "取消优化" }));
    await waitFor(() => expect(cancel).toHaveBeenCalledWith(expect.any(String)));
    expect(screen.getByLabelText("优化指令")).toHaveValue("更清晰");
    finish?.();
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("优化已取消"));
  });
});
