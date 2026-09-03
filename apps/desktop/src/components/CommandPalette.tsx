import { useEffect, useRef, useState } from "react";

import type { PromptListItem, PromptSearchPage } from "@prompt-hub/contracts";

import type { AppRoute } from "../app/navigation";

type CommandPaletteProps = {
  onClose: () => void;
  onCreate: () => void;
  onNavigate: (route: AppRoute) => void;
  loadPrompts: () => Promise<PromptListItem[]>;
  searchPrompts?: (text: string, limit?: number, offset?: number, filters?: undefined, sort?: "relevance") => Promise<PromptSearchPage>;
  onSelectPrompt: (prompt: PromptListItem) => void;
};

const commands: Array<{ label: string; route?: AppRoute; type: "create" | "navigate" }> = [
  { label: "新建提示词", type: "create" },
  { label: "打开提示词库", route: "library", type: "navigate" },
  { label: "打开 Skill 库", route: "skills", type: "navigate" },
  { label: "打开收件箱", route: "inbox", type: "navigate" },
  { label: "打开设置", route: "settings", type: "navigate" },
];

export function CommandPalette({ loadPrompts, onClose, onCreate, onNavigate, onSelectPrompt, searchPrompts }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [prompts, setPrompts] = useState<PromptListItem[]>([]);
  const [loadError, setLoadError] = useState(false);
  const [searchPromptIds, setSearchPromptIds] = useState<string[] | null>(null);
  const searchGeneration = useRef(0);
  const visibleCommands = commands.filter((command) => command.label.includes(query.trim()));
  const promptQuery = query.trim().toLocaleLowerCase();
  const visiblePrompts = promptQuery
    ? (searchPromptIds
      ? prompts.filter((prompt) => searchPromptIds.includes(prompt.id))
      : prompts.filter((prompt) => [prompt.title, ...prompt.tags, ...prompt.sourceNames].join(" ").toLocaleLowerCase().includes(promptQuery)))
      .slice(0, 8)
    : [];

  useEffect(() => {
    setLoadError(false);
    void loadPrompts()
      .then(setPrompts)
      .catch(() => {
        setPrompts([]);
        setLoadError(true);
      });
  }, [loadPrompts]);

  useEffect(() => {
    const trimmedQuery = query.trim();
    const generation = ++searchGeneration.current;
    if (!trimmedQuery || !searchPrompts) {
      setSearchPromptIds(null);
      return;
    }
    setSearchPromptIds(null);
    const timeout = window.setTimeout(() => {
      void searchPrompts(trimmedQuery, 8, 0, undefined, "relevance")
        .then((result) => {
          if (searchGeneration.current === generation) setSearchPromptIds(result.hits.map((hit) => hit.id));
        })
        .catch(() => {
          if (searchGeneration.current === generation) setSearchPromptIds(null);
        });
    }, 120);
    return () => window.clearTimeout(timeout);
  }, [query, searchPrompts]);

  const runCommand = (command: (typeof commands)[number]) => {
    if (command.type === "create") onCreate();
    else if (command.route) onNavigate(command.route);
    onClose();
  };

  return (
    <div
      aria-label="命令面板"
      aria-modal="true"
      className="command-palette-backdrop"
      onMouseDown={onClose}
      role="dialog"
    >
      <section className="command-palette" onMouseDown={(event) => event.stopPropagation()}>
        <label htmlFor="command-search">快速操作</label>
        <input autoFocus id="command-search" onChange={(event) => setQuery(event.target.value)} placeholder="搜索命令或提示词" value={query} />
        {loadError ? <p role="alert">提示词索引暂时不可用，仍可执行其他命令。</p> : null}
        {visibleCommands.length ? <ul aria-label="可用命令">{visibleCommands.map((command) => <li key={command.label}><button onClick={() => runCommand(command)} type="button">{command.label}</button></li>)}</ul> : null}
        {visiblePrompts.length ? <section aria-label="匹配提示词"><p>提示词</p><ul>{visiblePrompts.map((prompt) => <li key={prompt.id}><button aria-label={`打开提示词：${prompt.title}`} onClick={() => { onSelectPrompt(prompt); onClose(); }} type="button">{prompt.title}</button></li>)}</ul></section> : null}
        {!visibleCommands.length && !visiblePrompts.length ? <p>没有匹配的命令或提示词。</p> : null}
        <button onClick={onClose} type="button">关闭命令面板</button>
      </section>
    </div>
  );
}
