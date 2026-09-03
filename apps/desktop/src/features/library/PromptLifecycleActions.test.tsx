import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PromptLifecycleActions } from "./PromptLifecycleActions";

describe("PromptLifecycleActions", () => {
  it("requires confirmation before soft deletion and exposes recovery after deletion", async () => {
    const archive = vi.fn().mockResolvedValue(undefined);
    const publish = vi.fn().mockResolvedValue(undefined);
    const softDelete = vi.fn().mockResolvedValue(undefined);
    const recover = vi.fn().mockResolvedValue(undefined);
    render(
      <PromptLifecycleActions
        archive={archive}
        initialStatus="published"
        publish={publish}
        recover={recover}
        softDelete={softDelete}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "软删除提示词" }));
    expect(softDelete).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "确认软删除" }));
    await waitFor(() => expect(softDelete).toHaveBeenCalledOnce());
    expect(screen.getByRole("button", { name: "恢复提示词" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "恢复提示词" }));
    await waitFor(() => expect(recover).toHaveBeenCalledOnce());
    expect(screen.getByRole("button", { name: "归档提示词" })).toBeVisible();
  });

  it("publishes an inbox prompt through the lifecycle boundary", async () => {
    const publish = vi.fn().mockResolvedValue(undefined);
    render(<PromptLifecycleActions archive={vi.fn()} initialStatus="inbox" publish={publish} recover={vi.fn()} softDelete={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "发布提示词" }));
    await waitFor(() => expect(publish).toHaveBeenCalledOnce());
    expect(screen.getByText("当前状态：published")).toBeVisible();
  });

  it("requires the full title before permanently clearing a deleted prompt", async () => {
    const permanentlyDelete = vi.fn().mockResolvedValue({ path: "C:/backups/permanent-delete.db" });
    render(<PromptLifecycleActions archive={vi.fn()} initialStatus="deleted" permanentlyDelete={permanentlyDelete} promptTitle="代码审查" publish={vi.fn()} recover={vi.fn()} softDelete={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "永久清除提示词" }));
    expect(screen.getByRole("button", { name: "确认永久清除" })).toBeDisabled();
    fireEvent.change(screen.getByLabelText("输入提示词标题以确认"), { target: { value: "代码审查" } });
    fireEvent.click(screen.getByRole("button", { name: "确认永久清除" }));

    await waitFor(() => expect(permanentlyDelete).toHaveBeenCalledOnce());
    expect(screen.getByRole("status")).toHaveTextContent("C:/backups/permanent-delete.db");
  });

  it("prevents duplicate lifecycle operations while one is pending", async () => {
    const archive = vi.fn(() => new Promise<void>(() => undefined));
    render(<PromptLifecycleActions archive={archive} initialStatus="published" publish={vi.fn()} recover={vi.fn()} softDelete={vi.fn()} />);

    const archiveButton = screen.getByRole("button", { name: "归档提示词" });
    fireEvent.click(archiveButton);
    fireEvent.click(archiveButton);

    await waitFor(() => expect(archive).toHaveBeenCalledOnce());
    expect(archiveButton).toBeDisabled();
  });
});
