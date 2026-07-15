import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AiOptimizationReview } from "./AiOptimizationReview";

describe("AiOptimizationReview", () => {
  it("creates an optimization request and safely shows the candidate text", async () => {
    const optimize = vi.fn().mockResolvedValue({ body: "优化后的正文" });
    render(<AiOptimizationReview body={"原正文"} optimize={optimize} promptId="prompt-1" />);
    fireEvent.change(screen.getByLabelText("优化指令"), { target: { value: "更清晰" } });
    fireEvent.click(screen.getByRole("button", { name: "生成优化草稿" }));
    await waitFor(() => expect(optimize).toHaveBeenCalledWith("prompt-1", "更清晰"));
    expect(screen.getByLabelText("优化前后正文差异")).toHaveTextContent("优化后的正文");
  });
});
