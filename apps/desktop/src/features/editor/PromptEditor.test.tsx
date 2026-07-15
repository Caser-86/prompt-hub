import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptEditor } from "./PromptEditor";

describe("PromptEditor", () => {
  it("submits a manual prompt as an inbox draft", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    render(<PromptEditor saveDraft={saveDraft} />);

    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "代码审查" } });
    fireEvent.change(screen.getByLabelText("正文"), { target: { value: "审查当前变更" } });
    fireEvent.change(screen.getByLabelText("分类"), { target: { value: "开发" } });
    fireEvent.click(screen.getByRole("button", { name: "保存到收件箱" }));

    expect(saveDraft).toHaveBeenCalledWith({
      title: "代码审查",
      body: "审查当前变更",
      description: null,
      category: "开发",
      tags: [],
      variables: [],
    });
  });

  it("includes typed variables in a saved manual draft", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    render(<PromptEditor saveDraft={saveDraft} />);

    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "代码审查" } });
    fireEvent.change(screen.getByLabelText("正文"), { target: { value: "审查 {{language}}" } });
    fireEvent.click(screen.getByRole("button", { name: "添加变量" }));
    fireEvent.change(screen.getByLabelText("变量名称"), { target: { value: "language" } });
    fireEvent.change(screen.getByLabelText("变量默认值"), { target: { value: "Rust" } });
    fireEvent.click(screen.getByRole("button", { name: "保存到收件箱" }));

    expect(saveDraft).toHaveBeenCalledWith(expect.objectContaining({
      variables: [
        {
          name: "language",
          kind: "text",
          description: null,
          defaultValue: "Rust",
          required: false,
        },
      ],
    }));
  });

  it("renders a safe preview and identifies required variables without a value", () => {
    render(<PromptEditor saveDraft={vi.fn().mockResolvedValue(undefined)} />);

    fireEvent.change(screen.getByLabelText("正文"), { target: { value: "审查 {{language}} 的变更" } });
    fireEvent.click(screen.getByRole("button", { name: "添加变量" }));
    fireEvent.change(screen.getByLabelText("变量名称"), { target: { value: "language" } });
    fireEvent.click(screen.getByLabelText("变量必填"));

    expect(screen.getByText("缺少必填变量：language")).toBeVisible();
    expect(within(screen.getByLabelText("渲染预览")).getByText("审查 {{language}} 的变更")).toBeVisible();
    expect(screen.getByRole("button", { name: "复制提示词正文" })).toBeVisible();
  });
});
