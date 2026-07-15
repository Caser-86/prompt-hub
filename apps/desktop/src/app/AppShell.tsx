import { useEffect, useRef, useState } from "react";

import { CommandPalette } from "../components/CommandPalette";
import { NotificationRegion } from "../components/NotificationRegion";
import { PromptEditor } from "../features/editor/PromptEditor";
import { PromptMetadataEditor } from "../features/editor/PromptMetadataEditor";
import { PromptHistory } from "../features/history/PromptHistory";
import { PromptLibrary } from "../features/library/PromptLibrary";
import { PromptLifecycleActions } from "../features/library/PromptLifecycleActions";
import { PromptSearch } from "../features/search/PromptSearch";
import { InboxImport } from "../features/inbox/InboxImport";
import { SettingsPage } from "../features/settings/SettingsPage";
import { desktopCommands } from "../services/desktop";
import { navigationItems, type AppRoute } from "./navigation";
import type { PromptHistoryItem, PromptListItem } from "@prompt-hub/contracts";

export function AppShell() {
  const [activeRoute, setActiveRoute] = useState<AppRoute>("library");
  const [isCommandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [isEditorOpen, setEditorOpen] = useState(false);
  const [selectedPrompt, setSelectedPrompt] = useState<PromptListItem | null>(null);
  const [history, setHistory] = useState<PromptHistoryItem[] | null>(null);
  const [libraryKey, setLibraryKey] = useState(0);
  const commandTriggerRef = useRef<HTMLButtonElement>(null);

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
    if (!selectedPrompt) {
      setHistory(null);
      return;
    }
    void desktopCommands.promptHistory(selectedPrompt.id).then(setHistory).catch(() => setHistory([]));
  }, [selectedPrompt]);

  return (
    <div className="app-shell">
      <header className="app-header">
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
      </header>

      <div className="app-frame">
        <nav aria-label="主导航" className="primary-navigation">
          {navigationItems.map((item) => (
            <a
              aria-current={activeRoute === item.id ? "page" : undefined}
              href={`#${item.id}`}
              key={item.id}
              onClick={() => setActiveRoute(item.id)}
            >
              {item.label}
            </a>
          ))}
        </nav>

        <main className="app-content" id="main-content">
          <p className="eyebrow">PROMPT ASSET WORKSPACE</p>
          <h1>Prompt Hub</h1>
          <p>本地优先的提示词资产管理工具</p>
          {activeRoute === "library" ? (
            selectedPrompt ? (
              <section aria-labelledby="prompt-details-title">
                <button onClick={() => setSelectedPrompt(null)} type="button">返回提示词库</button>
                <h2 id="prompt-details-title">{selectedPrompt.title}</h2>
                <p>来源：{selectedPrompt.sourceNames.join("、") || "未记录"}</p>
                <time dateTime={selectedPrompt.createdAt}>创建于 {selectedPrompt.createdAt}</time>
                <time dateTime={selectedPrompt.updatedAt}>最后更新于 {selectedPrompt.updatedAt}</time>
                <p>适用工具：{selectedPrompt.applicableTools?.join("、") || "未记录"}</p>
                <p>适用模型：{selectedPrompt.applicableModels?.join("、") || "未记录"}</p>
                <p>有效性：{selectedPrompt.effectiveness}；评分：{selectedPrompt.rating ?? "未评分"}</p>
                <PromptMetadataEditor
                  promptId={selectedPrompt.id}
                  saveCompatibility={desktopCommands.recordPromptCompatibility}
                  saveValidation={desktopCommands.recordPromptValidation}
                />
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
                {history ? (
                  <PromptHistory
                    history={history}
                    restoreVersion={(versionNumber) => desktopCommands.restorePromptVersion(
                      selectedPrompt.id,
                      versionNumber,
                    )}
                  />
                ) : <p>正在加载版本历史…</p>}
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
                onCreate={() => setEditorOpen(true)}
                onFavorite={async (prompt, favorite) => { await desktopCommands.setPromptFavorite(prompt.id, favorite); }}
                onSelect={setSelectedPrompt}
              />
            )
          ) : activeRoute === "search" ? (
            <PromptSearch searchPrompts={desktopCommands.searchPrompts} />
          ) : activeRoute === "inbox" ? (
            <InboxImport importFile={desktopCommands.importFileToInbox} importFolder={desktopCommands.importFolderToInbox} importUrl={desktopCommands.importUrlToInbox} loadPrompts={desktopCommands.listPrompts} onReview={(prompt) => { setSelectedPrompt(prompt); setActiveRoute("library"); }} />
          ) : activeRoute === "settings" ? (
            <SettingsPage
              createBackup={desktopCommands.createManualBackup}
              getAiCredentialStatus={desktopCommands.getAiCredentialStatus}
              getApplicationStatus={desktopCommands.getApplicationStatus}
              getDiagnosticsStatus={desktopCommands.getDiagnosticsStatus}
              rebuildSearchIndex={desktopCommands.rebuildSearchIndex}
              getMcpSetup={desktopCommands.getMcpSetup}
              generateAiDraft={desktopCommands.generateAiDraft}
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
      {isCommandPaletteOpen ? <CommandPalette onClose={closeCommandPalette} /> : null}
    </div>
  );
}
