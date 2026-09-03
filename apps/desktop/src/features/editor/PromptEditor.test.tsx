import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptEditor } from "./PromptEditor";

describe("PromptEditor", () => {
  it("submits a manual prompt as an inbox draft", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    render(<PromptEditor saveDraft={saveDraft} />);
    expect(screen.getByRole("form", { name: "提示词编辑器" })).toHaveClass("prompt-editor");
    expect(screen.getByLabelText("标题").closest("label")).toHaveClass("editor-title-field");
    expect(screen.getByLabelText("正文").closest("label")).toHaveClass("editor-body-field");

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

  it("loads existing content, preserves tags, and removes variables before saving a revision", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptEditor
        initial={{
          title: "会议纪要",
          body: "整理 {{language}}",
          description: "说明",
          category: "工作",
          tags: ["会议", "效率"],
          variables: [{ name: "language", kind: "text", description: null, defaultValue: "中文", required: false }],
        }}
        onSaved={() => undefined}
        saveDraft={saveDraft}
        submitLabel="保存修订"
      />,
    );

    expect(screen.getByLabelText("标题")).toHaveValue("会议纪要");
    expect(screen.getByLabelText("正文")).toHaveValue("整理 {{language}}");
    expect(screen.getByLabelText("标签")).toHaveValue("会议, 效率");
    fireEvent.click(screen.getByRole("button", { name: "删除变量 language" }));
    fireEvent.click(screen.getByRole("button", { name: "保存修订" }));

    expect(saveDraft).toHaveBeenCalledWith({
      title: "会议纪要",
      body: "整理 {{language}}",
      description: "说明",
      category: "工作",
      tags: ["会议", "效率"],
      variables: [],
    });
  });

  it("rejects duplicate variable names before calling the backend", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    render(<PromptEditor saveDraft={saveDraft} />);
    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "测试" } });
    fireEvent.change(screen.getByLabelText("正文"), { target: { value: "{{one}}" } });
    fireEvent.click(screen.getByRole("button", { name: "添加变量" }));
    fireEvent.change(screen.getAllByLabelText("变量名称")[0], { target: { value: "one" } });
    fireEvent.click(screen.getByRole("button", { name: "添加变量" }));
    fireEvent.change(screen.getAllByLabelText("变量名称")[1], { target: { value: "one" } });
    fireEvent.click(screen.getByRole("button", { name: "保存到收件箱" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("变量名称不能重复");
    expect(saveDraft).not.toHaveBeenCalled();
  });

  it("does not report a successful save as failed when the post-save refresh fails", async () => {
    const saveDraft = vi.fn().mockResolvedValue(undefined);
    const onSaved = vi.fn().mockRejectedValue(new Error("refresh failed"));
    render(<PromptEditor onSaved={onSaved} saveDraft={saveDraft} />);

    fireEvent.change(screen.getByLabelText("标题"), { target: { value: "测试" } });
    fireEvent.change(screen.getByLabelText("正文"), { target: { value: "正文" } });
    fireEvent.click(screen.getByRole("button", { name: "保存到收件箱" }));

    await waitFor(() => expect(onSaved).toHaveBeenCalledOnce());
    expect(saveDraft).toHaveBeenCalledOnce();
    expect(await screen.findByRole("alert")).toHaveTextContent("提示词已保存，但界面刷新失败");
  });
});
