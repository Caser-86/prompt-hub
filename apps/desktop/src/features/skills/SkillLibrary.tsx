import { useEffect, useMemo, useState } from "react";
import { ShieldCheckIcon, StarIcon } from "@heroicons/react/24/outline";
import { StarIcon as StarSolidIcon } from "@heroicons/react/24/solid";

import type { GitSkillCollectionDraft, SkillDetail, SkillInstallDraft, SkillInstallation, SkillInstallationVerification, SkillListItem, SkillReviewDraft, SkillReviewStatus } from "@prompt-hub/contracts";

import "./skill-library.css";

type SkillLibraryProps = {
  collectSkillFolder: (path: string) => Promise<SkillListItem>;
  collectGitSkill: (source: GitSkillCollectionDraft) => Promise<SkillListItem>;
  getSkill: (id: string) => Promise<SkillDetail | null>;
  listSkills: () => Promise<SkillListItem[]>;
  reviewSkill: (id: string, review: SkillReviewDraft) => Promise<void>;
  setSkillFavorite: (id: string, favorite: boolean) => Promise<void>;
  installSkill: (id: string, installation: SkillInstallDraft) => Promise<SkillInstallation>;
  verifySkillInstallation: (id: string) => Promise<SkillInstallationVerification>;
};

type SkillFilter = "all" | "favorite" | "pending" | "approved" | "risk";

const statusLabels: Record<SkillReviewStatus, string> = {
  pending_review: "待审核",
  approved: "已审核",
  rejected: "已拒绝",
  risk_pending_confirmation: "风险待确认",
};

const riskLabels: Record<string, string> = {
  contains_script: "含脚本",
  contains_binary: "含二进制",
  contains_hidden_file: "含隐藏文件",
};

export function SkillLibrary({ collectSkillFolder, collectGitSkill, getSkill, listSkills, reviewSkill, setSkillFavorite, installSkill, verifySkillInstallation }: SkillLibraryProps) {
  const [skills, setSkills] = useState<SkillListItem[] | null>(null);
  const [selected, setSelected] = useState<SkillDetail | null>(null);
  const [folderPath, setFolderPath] = useState("");
  const [filter, setFilter] = useState<SkillFilter>("all");
  const [isCollecting, setCollecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installTarget, setInstallTarget] = useState("");
  const [destinationName, setDestinationName] = useState("");
  const [replaceAfterBackup, setReplaceAfterBackup] = useState(false);
  const [installation, setInstallation] = useState<SkillInstallation | null>(null);
  const [gitSource, setGitSource] = useState<GitSkillCollectionDraft>({ repositoryUrl: "", commit: "", subdirectory: "" });
  const [verification, setVerification] = useState<SkillInstallationVerification | null>(null);

  useEffect(() => {
    void listSkills().then(setSkills).catch(() => setError("无法读取本地 Skill 库，请重试。"));
  }, [listSkills]);

  const visibleSkills = useMemo(() => (skills ?? []).filter((skill) => {
    if (filter === "favorite") return skill.favorite;
    if (filter === "pending") return skill.reviewStatus === "pending_review";
    if (filter === "approved") return skill.reviewStatus === "approved";
    if (filter === "risk") return skill.risks.length > 0 || skill.reviewStatus === "risk_pending_confirmation";
    return true;
  }), [filter, skills]);

  const collect = async () => {
    const path = folderPath.trim();
    if (!path) return;
    setCollecting(true);
    setError(null);
    try {
      const skill = await collectSkillFolder(path);
      setSkills((current) => [skill, ...(current ?? []).filter((item) => item.id !== skill.id)]);
      setFolderPath("");
    } catch {
      setError("无法扫描此目录。请确认它是绝对路径、包含 SKILL.md，且目录没有超出扫描限制。");
    } finally {
      setCollecting(false);
    }
  };

  const collectGit = async () => {
    if (!gitSource.repositoryUrl.trim() || !gitSource.commit.trim()) return;
    setError(null);
    try {
      const skill = await collectGitSkill({ ...gitSource, repositoryUrl: gitSource.repositoryUrl.trim(), commit: gitSource.commit.trim(), subdirectory: gitSource.subdirectory.trim() });
      setSkills((current) => [skill, ...(current ?? []).filter((item) => item.id !== skill.id)]);
      setGitSource({ repositoryUrl: "", commit: "", subdirectory: "" });
    } catch { setError("无法读取 Git Skill。仅支持公开 GitHub HTTPS 地址和固定 40 位提交 SHA。"); }
  };

  const openSkill = async (id: string) => {
    setError(null);
    try {
      const detail = await getSkill(id);
      if (!detail) {
        setError("这个 Skill 已不在本地库中。");
        return;
      }
      setSelected(detail);
      setDestinationName(detail.name);
      setInstallation(detail.installation);
      setVerification(null);
      setReplaceAfterBackup(false);
    } catch {
      setError("无法读取 Skill 详情，请重试。");
    }
  };

  const updateReview = async (status: SkillReviewStatus) => {
    if (!selected) return;
    await reviewSkill(selected.id, { status, notes: null });
    setSelected({ ...selected, reviewStatus: status });
    setSkills((current) => current?.map((skill) => skill.id === selected.id ? { ...skill, reviewStatus: status } : skill) ?? null);
  };

  const toggleFavorite = async (skill: SkillListItem) => {
    const favorite = !skill.favorite;
    await setSkillFavorite(skill.id, favorite);
    setSkills((current) => current?.map((item) => item.id === skill.id ? { ...item, favorite } : item) ?? null);
    setSelected((current) => current?.id === skill.id ? { ...current, favorite } : current);
  };

  const install = async () => {
    if (!selected || !installTarget.trim() || !destinationName.trim()) return;
    if (replaceAfterBackup && !window.confirm("同名 Skill 将先创建备份，再替换为当前已审核版本。是否继续？")) return;
    setError(null);
    try {
      const result = await installSkill(selected.id, { targetRoot: installTarget.trim(), destinationName: destinationName.trim(), replaceAfterBackup });
      setInstallation(result);
    } catch (reason) {
      setError(`无法安装 Skill：${installErrorMessage(reason, "请检查目标路径、同名冲突和原始文件是否发生变化。")}`);
    }
  };
  const verifyInstallation = async () => { if (!selected) return; try { setVerification(await verifySkillInstallation(selected.id)); } catch { setError("无法验证已安装 Skill。请确认安装记录和目标目录仍可访问。"); } };

  if (selected) {
    return <section aria-labelledby="skill-detail-title" className="skill-library skill-detail-layout">
      <header className="skill-detail-header">
        <button className="button-secondary" onClick={() => setSelected(null)} type="button">返回 Skill 列表</button>
        <div><p className="eyebrow">SKILL REVIEW</p><h1 id="skill-detail-title">{selected.name}</h1></div>
        <button aria-label={selected.favorite ? `取消收藏 Skill：${selected.name}` : `收藏 Skill：${selected.name}`} className={`favorite-toggle${selected.favorite ? " is-favorite" : ""}`} onClick={() => void toggleFavorite(selected)} type="button">{selected.favorite ? <StarSolidIcon aria-hidden="true" /> : <StarIcon aria-hidden="true" />}</button>
      </header>
      {error ? <p role="alert" className="skill-error">{error}</p> : null}
      <div className="skill-detail-grid">
        <article className="surface-card skill-markdown-card">
          <div className="skill-card-heading"><div><h2>Skill 正文</h2><p>仅供审核与复制；本页面不会执行脚本。</p></div><span className={`skill-status skill-status-${selected.reviewStatus}`}>{statusLabels[selected.reviewStatus]}</span></div>
          <pre aria-label="Skill 正文">{selected.skillMarkdown}</pre>
        </article>
        <aside className="skill-detail-aside">
          <section className="surface-card skill-review-card">
            <h2>审核与风险</h2>
            {selected.risks.length ? <div className="skill-risk-notice"><ShieldCheckIcon aria-hidden="true" /><div><strong>需人工确认</strong><p>{selected.risks.map(riskLabel).join("、")}。收集和预览均未执行这些文件。</p></div></div> : <p className="skill-safe-note">未检测到脚本、二进制或隐藏文件。</p>}
            <div className="skill-review-actions"><button className="button-primary" onClick={() => void updateReview("approved")} type="button">审核通过</button><button className="button-secondary" onClick={() => void updateReview("risk_pending_confirmation")} type="button">标记风险待确认</button><button className="skill-reject-button" onClick={() => void updateReview("rejected")} type="button">拒绝</button></div>
          </section>
          <section className="surface-card skill-facts-card"><h2>收集记录</h2><dl><dt>来源</dt><dd>{selected.source.location}</dd><dt>内容校验</dt><dd><code>{selected.contentHash.slice(0, 12)}…</code></dd><dt>文件数</dt><dd>{selected.files.length} 个</dd><dt>更新时间</dt><dd><time dateTime={selected.updatedAt}>{formatDate(selected.updatedAt)}</time></dd></dl></section>
          <section className="surface-card skill-files-card"><h2>已收集文件</h2><ul>{selected.files.map((file) => <li key={file.relativePath}><span>{file.relativePath}</span><small>{file.kind} · {formatBytes(file.bytes)}</small></li>)}</ul></section>
          {selected.reviewStatus === "approved" ? <section aria-label="安装 Skill" className="surface-card skill-install-card"><h2>安装到 Codex</h2><p>安装只复制已审核文件，并再次核对内容；不会执行任何脚本。</p><label htmlFor="skill-install-target">目标目录</label><input id="skill-install-target" onChange={(event) => setInstallTarget(event.target.value)} placeholder="例如 C:\\Users\\you\\.codex\\skills" value={installTarget} /><label htmlFor="skill-install-name">安装目录名称</label><input id="skill-install-name" onChange={(event) => setDestinationName(event.target.value)} placeholder="例如 review-copy" value={destinationName} /><p>仅允许一个文件夹名称；如目标中已存在同名目录，默认会停止，不会覆盖。</p><label className="skill-replace-choice"><input checked={replaceAfterBackup} onChange={(event) => setReplaceAfterBackup(event.target.checked)} type="checkbox" />同名时先备份再替换</label><button className="button-primary" disabled={!installTarget.trim() || !destinationName.trim()} onClick={() => void install()} type="button">安装 Skill</button>{installation ? <><p className="skill-install-success">已安装到 {installation.installPath}{installation.backupPath ? "；原版本已备份。" : "。"}</p><button className="button-secondary" onClick={() => void verifyInstallation()} type="button">检查本地漂移</button>{verification ? <p className={`skill-verification skill-verification-${verification.state}`}>{verification.state === "matching" ? "安装内容一致" : verification.state === "drifted" ? "发现本地内容变化" : "安装目录不可用"}</p> : null}</> : null}</section> : <section className="surface-card skill-install-card skill-install-locked"><h2>安装</h2><p>审核通过后才能安装。安装始终是显式操作，不会自动执行或覆盖文件。</p></section>}
        </aside>
      </div>
    </section>;
  }

  return <section aria-labelledby="skills-title" className="skill-library">
    <header className="feature-heading skill-heading"><div><p className="eyebrow">LOCAL SKILLS</p><h1 id="skills-title">Skill 库</h1><p>收集、审核与安装 Codex Skill；扫描不会执行任何脚本。</p></div></header>
    <section aria-label="收集本地 Skill" className="surface-card skill-collect-panel"><div><label htmlFor="skill-folder-path">Skill 文件夹路径</label><input id="skill-folder-path" onChange={(event) => setFolderPath(event.target.value)} placeholder="例如 C:\\Users\\you\\.codex\\skills\\my-skill" value={folderPath} /><p>目录必须包含 <code>SKILL.md</code>。扫描只读取文件与风险标记。</p></div><button className="button-primary" disabled={!folderPath.trim() || isCollecting} onClick={() => void collect()} type="button">{isCollecting ? "扫描中…" : "扫描本地 Skill"}</button></section>
    <details className="surface-card skill-git-collect"><summary>从固定 Git 提交收集</summary><p>只支持公开 GitHub HTTPS 仓库和 40 位提交 SHA；不会检出或执行仓库脚本。</p><label htmlFor="git-skill-url">仓库地址</label><input id="git-skill-url" onChange={(event) => setGitSource((current) => ({ ...current, repositoryUrl: event.target.value }))} placeholder="https://github.com/org/repo.git" value={gitSource.repositoryUrl} /><label htmlFor="git-skill-commit">提交 SHA</label><input id="git-skill-commit" onChange={(event) => setGitSource((current) => ({ ...current, commit: event.target.value }))} placeholder="40 位提交 SHA" value={gitSource.commit} /><label htmlFor="git-skill-subdirectory">Skill 子目录（可选）</label><input id="git-skill-subdirectory" onChange={(event) => setGitSource((current) => ({ ...current, subdirectory: event.target.value }))} placeholder="例如 skills/reviewer" value={gitSource.subdirectory} /><button className="button-secondary" disabled={!gitSource.repositoryUrl.trim() || !gitSource.commit.trim()} onClick={() => void collectGit()} type="button">收集 Git Skill</button></details>
    <div className="skill-toolbar"><div aria-label="筛选 Skill" className="skill-filter-options" role="group">{(["all", "pending", "approved", "favorite", "risk"] as const).map((value) => <button aria-pressed={filter === value} className="library-filter-button" key={value} onClick={() => setFilter(value)} type="button">{filterLabel(value)}</button>)}</div><p className="library-result-count">共 {visibleSkills.length} 个 Skill</p></div>
    {skills?.length === 0 ? <section className="surface-card skill-empty-state"><h2>还没有收集到 Skill</h2><p>输入本地目录后开始扫描。每一个 Skill 都会先进入待审核状态。</p></section> : null}
    {visibleSkills.length ? <ul aria-label="Skill 列表" className="skill-list">{visibleSkills.map((skill) => <li className="surface-card skill-list-item" key={skill.id}><div className="skill-list-primary"><button aria-label={`打开 Skill：${skill.name}`} className="skill-list-title" onClick={() => void openSkill(skill.id)} type="button"><strong>{skill.name}</strong><span>{skill.description || "无描述"}</span></button></div><div className="skill-list-meta"><span className={`skill-status skill-status-${skill.reviewStatus}`}>{statusLabels[skill.reviewStatus]}</span>{skill.risks.map((risk) => <span className="skill-risk-chip" key={risk}>{riskLabel(risk)}</span>)}<span>{skill.source.kind === "git_repository" ? "Git 仓库" : "本地目录"}</span></div><button aria-label={skill.favorite ? `取消收藏 Skill：${skill.name}` : `收藏 Skill：${skill.name}`} className={`favorite-toggle${skill.favorite ? " is-favorite" : ""}`} onClick={() => void toggleFavorite(skill)} type="button">{skill.favorite ? <StarSolidIcon aria-hidden="true" /> : <StarIcon aria-hidden="true" />}</button></li>)}</ul> : skills?.length ? <section className="surface-card skill-empty-state"><p>当前筛选下没有 Skill。</p></section> : null}
    {error ? <p role="alert" className="skill-error">{error}</p> : null}
  </section>;
}

function filterLabel(filter: SkillFilter) { return { all: "全部", pending: "待审核", approved: "已审核", favorite: "收藏", risk: "有风险" }[filter]; }
function riskLabel(risk: string) { return riskLabels[risk] ?? "需审核"; }
function formatDate(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString("zh-CN", { hour12: false }); }
function formatBytes(bytes: number) { return bytes < 1024 ? `${bytes} B` : `${Math.ceil(bytes / 1024)} KB`; }
function errorMessage(reason: unknown, fallback: string) {
  if (typeof reason === "string" && reason.trim()) return reason;
  if (reason instanceof Error && reason.message.trim()) return reason.message;
  return fallback;
}
function installErrorMessage(reason: unknown, fallback: string) {
  const message = errorMessage(reason, "");
  const normalized = message.toLowerCase();
  if ((normalized.includes("destination") && normalized.includes("exist")) || message.includes("同名")) {
    return "同名冲突：目标目录中已有同名 Skill。请勾选“同名时先备份再替换”后重试。";
  }
  if (normalized.includes("source") && normalized.includes("chang")) {
    return "原始 Skill 内容已变化，请重新收集并审核后再安装。";
  }
  return message || fallback;
}
