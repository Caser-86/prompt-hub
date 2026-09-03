import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PromptContentActions } from "./PromptContentActions";

describe("PromptContentActions", () => {
  beforeEach(() => {
    Object.assign(navigator, { clipboard: { writeText: vi.fn().mockResolvedValue(undefined) } });
  });

  it("copies the current prompt body without modifying the library", async () => {
    render(<PromptContentActions body="请审查这段代码" title="代码审查" />);

    fireEvent.click(screen.getByRole("button", { name: "复制提示词正文" }));

    await waitFor(() => expect(navigator.clipboard.writeText).toHaveBeenCalledWith("请审查这段代码"));
    expect(screen.getByRole("status")).toHaveTextContent("已复制提示词正文");
  });

  it("keeps a successful copy when usage recording is temporarily unavailable", async () => {
    const onUsed = vi.fn().mockRejectedValue(new Error("database unavailable"));
    render(<PromptContentActions body="请审查这段代码" onUsed={onUsed} title="代码审查" />);

    fireEvent.click(screen.getByRole("button", { name: "复制提示词正文" }));

    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("已复制提示词正文"));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});
