import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SkillLibrary } from "./SkillLibrary";

const skill = {
  id: "skill-1", name: "本地审计", description: "安全检查工作流",
  source: { kind: "local_directory", location: "C:/Skills/audit", revision: null },
  risks: ["contains_script"], reviewStatus: "pending_review" as const,
  favorite: false, updatedAt: "2026-07-19T00:00:00Z",
};

describe("SkillLibrary", () => {
  it("collects a local folder into pending review and makes its risk visible", async () => {
    const collectSkillFolder = vi.fn().mockResolvedValue(skill);
    render(<SkillLibrary collectGitSkill={vi.fn()} collectSkillFolder={collectSkillFolder} getSkill={vi.fn()} installSkill={vi.fn()} listSkills={async () => []} reviewSkill={vi.fn()} setSkillFavorite={vi.fn()} verifySkillInstallation={vi.fn()} />);

    fireEvent.change(screen.getByLabelText("Skill 文件夹路径"), { target: { value: "C:/Skills/audit" } });
    fireEvent.click(screen.getByRole("button", { name: "扫描本地 Skill" }));

    await waitFor(() => expect(collectSkillFolder).toHaveBeenCalledWith("C:/Skills/audit"));
    expect(await screen.findByRole("button", { name: "打开 Skill：本地审计" })).toBeVisible();
    expect(screen.getByText("含脚本")).toBeVisible();
    expect(screen.getByLabelText("Skill 列表")).toHaveTextContent("待审核");
  });

  it("shows raw Skill markdown only after opening details and reviews explicitly", async () => {
    const reviewSkill = vi.fn().mockResolvedValue(undefined);
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={async () => ({ ...skill, reviewNotes: null, skillMarkdown: "# 本地审计\n正文", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z", installation: null })}
      listSkills={async () => [skill]}
      reviewSkill={reviewSkill}
      setSkillFavorite={vi.fn()}
      installSkill={vi.fn()}
      verifySkillInstallation={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    expect(await screen.findByLabelText("Skill 正文")).toHaveTextContent("# 本地审计");
    fireEvent.click(screen.getByRole("button", { name: "审核通过" }));
    await waitFor(() => expect(reviewSkill).toHaveBeenCalledWith("skill-1", { status: "approved", notes: null }));
  });

  it("lets an approved Skill use a safe destination name chosen by the user", async () => {
    const installSkill = vi.fn().mockResolvedValue({ installPath: "C:/Codex/skills/review-copy", backupPath: null, installedHash: "b".repeat(64) });
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={async () => ({ ...skill, reviewStatus: "approved" as const, reviewNotes: null, skillMarkdown: "# 本地审计", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z", installation: null })}
      listSkills={async () => [{ ...skill, reviewStatus: "approved" as const }]}
      reviewSkill={vi.fn()}
      setSkillFavorite={vi.fn()}
      installSkill={installSkill}
      verifySkillInstallation={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    fireEvent.change(await screen.findByLabelText("安装目录名称"), { target: { value: "review-copy" } });
    fireEvent.change(screen.getByLabelText("目标目录"), { target: { value: "C:/Codex/skills" } });
    fireEvent.click(screen.getByRole("button", { name: "安装 Skill" }));

    await waitFor(() => expect(installSkill).toHaveBeenCalledWith("skill-1", {
      targetRoot: "C:/Codex/skills", destinationName: "review-copy", replaceAfterBackup: false,
    }));
  });

  it("explains a same-name installation conflict in the user's language", async () => {
    const installSkill = vi.fn().mockRejectedValue(new Error("Skill destination already exists"));
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={async () => ({ ...skill, reviewStatus: "approved" as const, reviewNotes: null, skillMarkdown: "# 本地审计", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z", installation: null })}
      listSkills={async () => [{ ...skill, reviewStatus: "approved" as const }]}
      reviewSkill={vi.fn()}
      setSkillFavorite={vi.fn()}
      installSkill={installSkill}
      verifySkillInstallation={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    fireEvent.change(await screen.findByLabelText("安装目录名称"), { target: { value: "review-copy" } });
    fireEvent.change(screen.getByLabelText("目标目录"), { target: { value: "C:/Codex/skills" } });
    fireEvent.click(screen.getByRole("button", { name: "安装 Skill" }));

    await expect(screen.findByRole("alert")).resolves.toHaveTextContent("同名冲突");
  });

  it("prevents duplicate Skill installation submissions while the first is pending", async () => {
    let resolveInstall: (() => void) | undefined;
    const installSkill = vi.fn().mockImplementation(() => new Promise<void>((resolve) => {
      resolveInstall = () => resolve();
    }));
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={async () => ({ ...skill, reviewStatus: "approved" as const, reviewNotes: null, skillMarkdown: "# 本地审计", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z", installation: null })}
      listSkills={async () => [{ ...skill, reviewStatus: "approved" as const }]}
      reviewSkill={vi.fn()}
      setSkillFavorite={vi.fn()}
      installSkill={installSkill}
      verifySkillInstallation={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    fireEvent.change(await screen.findByLabelText("安装目录名称"), { target: { value: "review-copy" } });
    fireEvent.change(screen.getByLabelText("目标目录"), { target: { value: "C:/Codex/skills" } });
    const installButton = screen.getByRole("button", { name: "安装 Skill" });
    fireEvent.click(installButton);
    fireEvent.click(installButton);

    expect(installSkill).toHaveBeenCalledOnce();
    expect(installButton).toBeDisabled();
    resolveInstall?.();
    await waitFor(() => expect(installButton).not.toBeDisabled());
  });

  it("prevents duplicate list favorite submissions while the first is pending", async () => {
    let resolveFavorite: (() => void) | undefined;
    const setSkillFavorite = vi.fn().mockImplementation(() => new Promise<void>((resolve) => {
      resolveFavorite = () => resolve();
    }));
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={vi.fn()}
      installSkill={vi.fn()}
      listSkills={async () => [skill]}
      reviewSkill={vi.fn()}
      setSkillFavorite={setSkillFavorite}
      verifySkillInstallation={vi.fn()}
    />);

    const favoriteButton = await screen.findByRole("button", { name: "收藏 Skill：本地审计" });
    fireEvent.click(favoriteButton);
    fireEvent.click(favoriteButton);

    expect(setSkillFavorite).toHaveBeenCalledOnce();
    expect(favoriteButton).toBeDisabled();
    resolveFavorite?.();
    await waitFor(() => expect(favoriteButton).not.toBeDisabled());
  });

  it("reports review and favorite failures instead of leaving rejected promises unhandled", async () => {
    const reviewSkill = vi.fn().mockRejectedValue(new Error("offline"));
    const setSkillFavorite = vi.fn().mockRejectedValue(new Error("offline"));
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={async () => ({ ...skill, reviewNotes: null, skillMarkdown: "# 本地审计", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z", installation: null })}
      listSkills={async () => [skill]}
      reviewSkill={reviewSkill}
      setSkillFavorite={setSkillFavorite}
      installSkill={vi.fn()}
      verifySkillInstallation={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    expect(await screen.findByRole("heading", { name: "本地审计" })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "审核通过" }));
    await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("无法更新 Skill，请重试"));
    fireEvent.click(screen.getByRole("button", { name: "收藏 Skill：本地审计" }));
    await waitFor(() => expect(setSkillFavorite).toHaveBeenCalledWith("skill-1", true));
    expect(screen.getByRole("alert")).toHaveTextContent("无法更新 Skill，请重试");
  });

  it("offers a retry when the local Skill list cannot be loaded", async () => {
    const listSkills = vi.fn()
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValueOnce([skill]);
    render(<SkillLibrary
      collectSkillFolder={vi.fn()}
      collectGitSkill={vi.fn()}
      getSkill={vi.fn()}
      installSkill={vi.fn()}
      listSkills={listSkills}
      reviewSkill={vi.fn()}
      setSkillFavorite={vi.fn()}
      verifySkillInstallation={vi.fn()}
    />);

    expect(await screen.findByRole("alert")).toHaveTextContent("无法读取本地 Skill 库");
    fireEvent.click(screen.getByRole("button", { name: "重试读取 Skill 库" }));
    await waitFor(() => expect(listSkills).toHaveBeenCalledTimes(2));
    expect(await screen.findByRole("button", { name: "打开 Skill：本地审计" })).toBeVisible();
  });
});
