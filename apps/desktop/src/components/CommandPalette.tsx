import { useState } from "react";

import type { AppRoute } from "../app/navigation";

type CommandPaletteProps = {
  onClose: () => void;
  onCreate: () => void;
  onNavigate: (route: AppRoute) => void;
};

const commands: Array<{ label: string; route?: AppRoute; type: "create" | "navigate" }> = [
  { label: "新建提示词", type: "create" },
  { label: "打开提示词库", route: "library", type: "navigate" },
  { label: "打开收件箱", route: "inbox", type: "navigate" },
  { label: "打开搜索", route: "search", type: "navigate" },
  { label: "打开设置", route: "settings", type: "navigate" },
];

export function CommandPalette({ onClose, onCreate, onNavigate }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const visibleCommands = commands.filter((command) => command.label.includes(query.trim()));

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
        {visibleCommands.length ? <ul aria-label="可用命令">{visibleCommands.map((command) => <li key={command.label}><button onClick={() => runCommand(command)} type="button">{command.label}</button></li>)}</ul> : <p>没有匹配的命令。</p>}
        <button onClick={onClose} type="button">关闭命令面板</button>
      </section>
    </div>
  );
}
