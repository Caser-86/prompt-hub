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
    render(<SkillLibrary collectSkillFolder={collectSkillFolder} getSkill={vi.fn()} installSkill={vi.fn()} listSkills={async () => []} reviewSkill={vi.fn()} setSkillFavorite={vi.fn()} />);

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
      getSkill={async () => ({ ...skill, reviewNotes: null, skillMarkdown: "# 本地审计\n正文", files: [], contentHash: "a".repeat(64), createdAt: "2026-07-19T00:00:00Z" })}
      listSkills={async () => [skill]}
      reviewSkill={reviewSkill}
      setSkillFavorite={vi.fn()}
      installSkill={vi.fn()}
    />);

    fireEvent.click(await screen.findByRole("button", { name: "打开 Skill：本地审计" }));
    expect(await screen.findByLabelText("Skill 正文")).toHaveTextContent("# 本地审计");
    fireEvent.click(screen.getByRole("button", { name: "审核通过" }));
    await waitFor(() => expect(reviewSkill).toHaveBeenCalledWith("skill-1", { status: "approved", notes: null }));
  });
});
