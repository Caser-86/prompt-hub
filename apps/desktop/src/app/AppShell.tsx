import { useEffect, useRef, useState } from "react";
import {
  ArrowLeftIcon,
  CalendarDaysIcon,
  CircleStackIcon,
  CubeIcon,
  SparklesIcon,
  StarIcon,
  TagIcon,
  WrenchScrewdriverIcon,
} from "@heroicons/react/24/outline";
import "./prompt-detail.css";

import { CommandPalette } from "../components/CommandPalette";
import { NotificationRegion } from "../components/NotificationRegion";
import { PromptEditor } from "../features/editor/PromptEditor";
import { PromptMetadataEditor } from "../features/editor/PromptMetadataEditor";
import { PromptHistory } from "../features/history/PromptHistory";
import { AiOptimizationReview } from "../features/ai/AiOptimizationReview";
import { PromptLibrary } from "../features/library/PromptLibrary";
import { PromptContentActions } from "../features/library/PromptContentActions";
import { markPromptUsageMigrated, readPromptUsage, shouldMigratePromptUsage } from "../features/library/promptUsage";
import { separatePromptProvenance } from "../features/library/promptContent";
import { PromptLifecycleActions } from "../features/library/PromptLifecycleActions";
import { PromptSearch } from "../features/search/PromptSearch";
import { InboxImport } from "../features/inbox/InboxImport";
import { SettingsPage } from "../features/settings/SettingsPage";
import { SkillLibrary } from "../features/skills/SkillLibrary";
import { RecoveryScreen } from "../features/recovery/RecoveryScreen";
import { desktopCommands } from "../services/desktop";
import { navigationItems, type AppRoute } from "./navigation";
import type { BootstrapStatus, PromptHistoryItem, PromptListItem } from "@prompt-hub/contracts";

const effectivenessLabels = {
  effective: "已验证",
  ineffective: "已失效",
  needs_retest: "待复测",
  unverified: "未验证",
} as const;

function DetailInfoRow({ children, icon: Icon, label }: {
  children: React.ReactNode;
  icon: typeof CircleStackIcon;
  label: string;
}) {
  return <div className="detail-info-row">
    <Icon aria-hidden="true" />
    <span>{label}</span>
    <div>{children}</div>
  </div>;
}

export function AppShell() {
  const [activeRoute, setActiveRoute] = useState<AppRoute>("library");
  const [isCommandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [isEditorOpen, setEditorOpen] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<PromptListItem | null>(null);
  const [history, setHistory] = useState<PromptHistoryItem[] | null>(null);
  const [historyLoadError, setHistoryLoadError] = useState(false);
  const [historyReloadKey, setHistoryReloadKey] = useState(0);
  const [libraryKey, setLibraryKey] = useState(0);
  const [bootstrapStatus, setBootstrapStatus] = useState<BootstrapStatus>({ state: "ready", code: null, safeMessage: null, backupName: null });
  const commandTriggerRef = useRef<HTMLButtonElement>(null);
  const promptContent = separatePromptProvenance(history?.at(-1)?.body ?? "");
  const displayedSources = selectedPrompt
    ? [
      ...(selectedPrompt.sources ?? []),
      ...(promptContent.provenance && !(selectedPrompt.sources ?? []).some((source) => source.location === promptContent.provenance?.location)
        ? [{ kind: "reference", ...promptContent.provenance }]
        : []),
    ]
    : [];
  const metadataExport = displayedSources
    .map((source) => `- ${source.name}（${source.location ?? "无位置记录"}；采集于 ${source.collectedAt}）`)
    .join("\n");

  const closeCommandPalette = () => {
    setCommandPaletteOpen(false);
    commandTriggerRef.current?.focus();
  };

  useEffect(() => {
    const handleKeyboardShortcut = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandPaletteOpen(true);
      }
      if (event.key === "Escape") {
        closeCommandPalette();
      }
    };

    window.addEventListener("keydown", handleKeyboardShortcut);
    return () => window.removeEventListener("keydown", handleKeyboardShortcut);
  }, []);

  useEffect(() => {
    if (!shouldMigratePromptUsage()) return;
    const legacyUsage = readPromptUsage();
    const entries = Object.entries(legacyUsage).map(([id, useCount]) => ({ id, useCount }));
    if (!entries.length) {
      markPromptUsageMigrated();
      return;
    }
    void desktopCommands.migrateLegacyPromptUsage(entries).then(() => markPromptUsageMigrated()).catch(() => undefined);
  }, []);

  useEffect(() => {
    const promptId = selectedPrompt?.id;
    if (!promptId) {
      setHistory(null);
      setHistoryLoadError(false);
      return;
    }
    setHistory(null);
    setHistoryLoadError(false);
    let active = true;
    void desktopCommands.promptHistory(promptId)
      .then((nextHistory) => {
        if (!active) return;
        // A prompt without any version is not a usable detail record. Treat
        // this as a recoverable load failure instead of offering copy/edit
        // actions that would operate on an empty body.
        if (nextHistory.length === 0) {
          setHistory([]);
          setHistoryLoadError(true);
          return;
        }
        setHistory(nextHistory);
      })
      .catch(() => {
        if (active) {
          setHistory([]);
          setHistoryLoadError(true);
        }
      });
    return () => { active = false; };
  }, [selectedPrompt?.id, historyReloadKey]);

  useEffect(() => {
    void desktopCommands.getBootstrapStatus()
      .then(setBootstrapStatus)
      .catch(() => setBootstrapStatus({ state: "recovery", code: "bootstrap_status_unavailable", safeMessage: "应用启动状态不可用，请重试。", backupName: null }));
  }, []);

  if (bootstrapStatus.state === "recovery") {
    return <RecoveryScreen
      exportDiagnostics={desktopCommands.exportBootstrapDiagnostics}
      listRecoveryBackups={desktopCommands.listRecoveryBackups}
      onRecovered={() => { setBootstrapStatus({ state: "ready", code: null, safeMessage: null, backupName: null }); }}
      previewRecoveryBackup={desktopCommands.previewRecoveryBackup}
      retry={desktopCommands.retryDatabaseBootstrap}
      restoreRecoveryBackup={desktopCommands.restoreRecoveryBackup}
      status={bootstrapStatus}
    />;
  }

  return (
    <div className="app-shell">
      <header className="app-header workspace-header">
        <a className="brand" href="#library" onClick={() => setActiveRoute("library")}>
          <span aria-hidden="true" className="brand-mark">P</span>
          <span>Prompt Hub</span>
        </a>
        <button
          aria-label="打开命令面板"
          className="command-trigger"
          onClick={() => setCommandPaletteOpen(true)}
          ref={commandTriggerRef}
          type="button"
        >
          <span>搜索提示词或执行命令</span>
          <kbd>Ctrl K</kbd>
        </button>
        <button
          className="button-primary header-create"
          onClick={() => {
            setActiveRoute("library");
            setSelectedPrompt(null);
            setEditorOpen(true);
          }}
          type="button"
        >
          <span aria-hidden="true">＋</span>
          新建提示词
        </button>
      </header>

      <div className="app-frame">
        <nav aria-label="主导航" className="primary-navigation">
          {navigationItems.map((item) => (
            <a
              aria-current={activeRoute === item.id ? "page" : undefined}
              href={`#${item.id}`}
              key={item.id}
              className="navigation-link"
              onClick={() => setActiveRoute(item.id)}
            >
              <span aria-hidden="true" className="navigation-marker">{item.label.slice(0, 1)}</span>
              <span>{item.label}</span>
            </a>
          ))}
        </nav>

        <main className="app-content content-frame" id="main-content">
          {activeRoute === "library" ? (
            selectedPrompt ? (
              <section aria-labelledby="prompt-details-title" className="prompt-details-layout">
                <header className="prompt-detail-header">
                  <button className="prompt-detail-back" onClick={() => setSelectedPrompt(null)} type="button"><ArrowLeftIcon aria-hidden="true" /> 返回</button>
                  <div>
                    <p className="eyebrow">PROMPT DETAIL</p>
                    <div className="prompt-detail-title-row">
                      <h2 id="prompt-details-title">{selectedPrompt.title}</h2>
                      <span className={`status-pill status-${selectedPrompt.effectiveness}`}>{effectivenessLabels[selectedPrompt.effectiveness as keyof typeof effectivenessLabels] ?? "未验证"}</span>
                    </div>
                  </div>
                  <section aria-label="提示词主操作" className="prompt-detail-actions">
                    {historyLoadError ? <p role="alert">无法加载提示词正文。<button onClick={() => setHistoryReloadKey((key) => key + 1)} type="button">重试加载提示词</button></p> : history ? <PromptContentActions
                      body={promptContent.body}
                      metadataExport={metadataExport}
                      onUsed={async () => {
                        const usage = await desktopCommands.recordPromptUse(selectedPrompt.id);
                        setSelectedPrompt((current) => current ? { ...current, useCount: usage.useCount, lastUsedAt: usage.lastUsedAt } : current);
                        setLibraryKey((key) => key + 1);
                      }}
                      title={selectedPrompt.title}
                    /> : <p>正在准备提示词操作…</p>}
                  </section>
                </header>
                <div className="prompt-detail-main">
                  <article aria-label="提示词正文" className="prompt-detail-body surface-card">
                    <h3>提示词正文</h3>
                    {historyLoadError ? <p>无法加载提示词正文，请使用上方“重试加载提示词”。</p> : history ? <pre>{promptContent.body}</pre> : <p>正在加载提示词正文…</p>}
                  </article>
                  <div className="prompt-detail-aside">
                  <aside aria-label="提示词信息" className="prompt-detail-info surface-card">
                    <h3>提示词信息</h3>
                    <DetailInfoRow icon={CircleStackIcon} label="来源">{selectedPrompt.sourceNames.join("、") || "未记录"}</DetailInfoRow>
                    <DetailInfoRow icon={WrenchScrewdriverIcon} label="适用工具">{selectedPrompt.applicableTools?.join("、") || "未记录"}</DetailInfoRow>
                    <DetailInfoRow icon={CubeIcon} label="推荐模型">{selectedPrompt.applicableModels?.join("、") || "未记录"}</DetailInfoRow>
                    <DetailInfoRow icon={StarIcon} label="效果评级"><span className="detail-rating" aria-label={`评分：${selectedPrompt.rating ?? "未评分"}`}>{selectedPrompt.rating ? "★".repeat(selectedPrompt.rating) : "未评分"}</span></DetailInfoRow>
                    <DetailInfoRow icon={CalendarDaysIcon} label="创建时间"><time dateTime={selectedPrompt.createdAt}>{selectedPrompt.createdAt}</time></DetailInfoRow>
                    <DetailInfoRow icon={CalendarDaysIcon} label="更新时间"><time dateTime={selectedPrompt.updatedAt}>{selectedPrompt.updatedAt}</time></DetailInfoRow>
                    <DetailInfoRow icon={TagIcon} label="标签"><span className="detail-tags">{selectedPrompt.tags.length ? selectedPrompt.tags.map((tag) => <span className="detail-tag" key={tag}>{tag}</span>) : "未记录"}</span></DetailInfoRow>
                    {displayedSources.length ? <details className="prompt-sources"><summary>完整来源</summary><ul>{displayedSources.map((source) => <li key={`${source.kind}-${source.name}-${source.collectedAt}`}>{source.name} · {source.location ?? "无位置记录"} · <time dateTime={source.collectedAt}>{source.collectedAt}</time></li>)}</ul></details> : null}
                    <PromptMetadataEditor
                      initial={{
                        tool: selectedPrompt.applicableTools?.[0],
                        model: selectedPrompt.applicableModels?.[0],
                        compatibilityStatus: selectedPrompt.compatibilityStatuses?.[0] ?? "unknown",
                        effectiveness: selectedPrompt.effectiveness as "unverified" | "effective" | "ineffective" | "needs_retest",
                        rating: selectedPrompt.rating ?? null,
                      }}
                      promptId={selectedPrompt.id}
                      saveMetadata={async (id, metadata) => {
                        await desktopCommands.recordPromptMetadata(id, metadata);
                        try {
                          const updated = await desktopCommands.getPrompt(id);
                          if (updated) setSelectedPrompt(updated);
                        } catch {
                          // The metadata write already succeeded; avoid asking the user
                          // to submit it again just because the detail refresh failed.
                        }
                        try {
                          const refreshedHistory = await desktopCommands.promptHistory(id);
                          if (refreshedHistory.length === 0) {
                            setHistory([]);
                            setHistoryLoadError(true);
                          } else {
                            setHistory(refreshedHistory);
                            setHistoryLoadError(false);
                          }
                        } catch {
                          // Keep the current history visible if the refresh is temporarily unavailable.
                        }
                        setLibraryKey((key) => key + 1);
                      }}
                    />
                  </aside>
                  <section aria-label="更多操作" className="prompt-detail-more surface-card">
                  {history && !historyLoadError ? <PromptHistory
                    history={history}
                    restoreVersion={async (versionNumber) => {
                      const promptId = selectedPrompt.id;
                      await desktopCommands.restorePromptVersion(promptId, versionNumber);
                      // A successful restore must not be reported as failed merely because
                      // refreshing the detail view is temporarily unavailable. The next
                      // library reload will still pick up the new current version.
                      try {
                        const refreshedHistory = await desktopCommands.promptHistory(promptId);
                        if (refreshedHistory.length === 0) {
                          setHistory([]);
                          setHistoryLoadError(true);
                        } else {
                          setHistory(refreshedHistory);
                          setHistoryLoadError(false);
                        }
                      } catch {
                        // Keep the detail open; the list refresh below will make the
                        // persisted version visible when the user returns.
                      }
                      setLibraryKey((key) => key + 1);
                    }}
                  /> : null}
                  <details className="prompt-editor-disclosure">
                    <summary>编辑提示词正文</summary>
                    {history && !historyLoadError ? (
                      <PromptEditor
                        initial={{
                          title: selectedPrompt.title,
                          body: promptContent.body,
                          description: selectedPrompt.description ?? null,
                          category: selectedPrompt.category,
                          tags: selectedPrompt.tags,
                          variables: selectedPrompt.variables ?? [],
                        }}
                        onSaved={async () => {
                          const updated = await desktopCommands.getPrompt(selectedPrompt.id);
                          if (updated) setSelectedPrompt(updated);
                          try {
                            const refreshedHistory = await desktopCommands.promptHistory(selectedPrompt.id);
                            if (refreshedHistory.length === 0) {
                              setHistory([]);
                              setHistoryLoadError(true);
                            } else {
                              setHistory(refreshedHistory);
                              setHistoryLoadError(false);
                            }
                          } catch {
                            // The content write already succeeded; the next detail refresh can retry history.
                          }
                          setLibraryKey((key) => key + 1);
                        }}
                        saveDraft={(draft) => desktopCommands.revisePrompt(selectedPrompt.id, draft)}
                        showPreviewActions={false}
                        submitLabel="保存修订"
                      />
                    ) : historyLoadError ? <p>无法加载提示词编辑器，请先使用上方“重试加载提示词”。</p> : <p>正在加载提示词正文…</p>}
                  </details>
                  <details className="prompt-ai-disclosure">
                    <summary><SparklesIcon aria-hidden="true" /> AI 优化</summary>
                    {history && !historyLoadError ? <AiOptimizationReview
                      body={promptContent.body}
                      cancel={desktopCommands.cancelAiGeneration}
                      promptId={selectedPrompt.id}
                      optimize={async (id, instruction, taskId) => {
                        const stored = JSON.parse(localStorage.getItem("prompt-hub.ai.draft-settings") ?? "{}") as { endpoint?: string; model?: string };
                        const result = await desktopCommands.optimizeAiPrompt(id, {
                          taskId,
                          endpoint: stored.endpoint ?? "https://api.openai.com", providerId: "openai-compatible",
                          instruction, inputSummary: "", model: stored.model ?? "",
                        }) as { current_version?: { content?: { body?: string } } };
                        return { body: result.current_version?.content?.body };
                      }}
                    /> : historyLoadError ? <p>无法加载提示词正文，暂时不能进行 AI 优化。</p> : <p>正在加载 AI 优化…</p>}
                  </details>
                  <PromptLifecycleActions
                    archive={() => desktopCommands.archivePrompt(selectedPrompt.id)}
                    initialStatus={selectedPrompt.status}
                    permanentlyDelete={() => desktopCommands.permanentlyDeletePrompt(selectedPrompt.id)}
                    onPermanentlyDeleted={() => { setSelectedPrompt(null); setLibraryKey((key) => key + 1); }}
                    publish={() => desktopCommands.publishPrompt(selectedPrompt.id)}
                    promptTitle={selectedPrompt.title}
                    recover={() => desktopCommands.recoverPrompt(selectedPrompt.id)}
                    softDelete={() => desktopCommands.softDeletePrompt(selectedPrompt.id)}
                  />
                  </section>
                  </div>
                </div>
              </section>
            ) : isEditorOpen ? (
              <PromptEditor
                onSaved={() => {
                  setEditorOpen(false);
                  setLibraryKey((key) => key + 1);
                }}
                saveDraft={desktopCommands.createManualPromptDraft}
              />
            ) : (
              <PromptLibrary
                key={libraryKey}
                batchArchive={async (ids) => { await desktopCommands.batchArchivePrompts(ids); }}
                loadPrompts={desktopCommands.listPrompts}
                onAdvancedSearch={() => setActiveRoute("search")}
                onCreate={() => setEditorOpen(true)}
                onFavorite={async (prompt, favorite) => { await desktopCommands.setPromptFavorite(prompt.id, favorite); }}
                onSelect={setSelectedPrompt}
              />
            )
          ) : activeRoute === "search" ? (
            <PromptSearch
              onSelectPrompt={async (id) => {
                const prompt = await desktopCommands.getPrompt(id);
                if (prompt) {
                  setActiveRoute("library");
                  setEditorOpen(false);
                  setSelectedPrompt(prompt);
                }
              }}
              searchPrompts={desktopCommands.searchPrompts}
            />
          ) : activeRoute === "skills" ? (
            <SkillLibrary
              collectSkillFolder={desktopCommands.collectSkillFolder}
              collectGitSkill={desktopCommands.collectGitSkill}
              getSkill={desktopCommands.getSkill}
              listSkills={desktopCommands.listSkills}
              reviewSkill={desktopCommands.reviewSkill}
              setSkillFavorite={desktopCommands.setSkillFavorite}
              installSkill={desktopCommands.installSkill}
              verifySkillInstallation={desktopCommands.verifySkillInstallation}
            />
          ) : activeRoute === "inbox" ? (
            <InboxImport importFile={desktopCommands.importFileToInbox} importFolder={desktopCommands.importFolderToInbox} importUrl={desktopCommands.importUrlToInbox} loadPrompts={desktopCommands.listPrompts} onReview={(prompt) => { setSelectedPrompt(prompt); setActiveRoute("library"); }} />
          ) : activeRoute === "settings" ? (
            <SettingsPage
              createBackup={desktopCommands.createManualBackup}
              getAiCredentialStatus={desktopCommands.getAiCredentialStatus}
              getApplicationStatus={desktopCommands.getApplicationStatus}
              getDiagnosticsStatus={desktopCommands.getDiagnosticsStatus}
              getRedactedDiagnosticEvents={desktopCommands.getRedactedDiagnosticEvents}
              rebuildSearchIndex={desktopCommands.rebuildSearchIndex}
              getMcpSetup={desktopCommands.getMcpSetup}
              generateAiDraft={desktopCommands.generateAiDraft}
              cancelAiGeneration={desktopCommands.cancelAiGeneration}
              testAiConnection={desktopCommands.testAiConnection}
              previewRestore={desktopCommands.previewBackupRestore}
              pruneLocalBackups={desktopCommands.pruneLocalBackups}
              recentImportJobs={desktopCommands.recentImportJobs}
              restoreBackup={desktopCommands.restoreBackup}
              saveAiCredential={desktopCommands.saveAiCredential}
            />
          ) : (
            <section aria-label={`${navigationItems.find((item) => item.id === activeRoute)?.label}内容`} className="empty-state">
              <h2>{navigationItems.find((item) => item.id === activeRoute)?.label}</h2>
              <p>当前没有可显示的内容。</p>
            </section>
          )}
        </main>
      </div>

      <NotificationRegion />
      {isCommandPaletteOpen ? <CommandPalette
        loadPrompts={desktopCommands.listPrompts}
        onClose={closeCommandPalette}
        onCreate={() => {
          setActiveRoute("library");
          setSelectedPrompt(null);
          setEditorOpen(true);
        }}
        onNavigate={(route) => {
          setActiveRoute(route);
          setSelectedPrompt(null);
          setEditorOpen(false);
        }}
        onSelectPrompt={(prompt) => {
          setActiveRoute("library");
          setSelectedPrompt(prompt);
          setEditorOpen(false);
        }}
        searchPrompts={desktopCommands.searchPrompts}
      /> : null}
    </div>
  );
}
