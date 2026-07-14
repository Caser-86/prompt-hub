import { useEffect, useRef, useState } from "react";

import { CommandPalette } from "../components/CommandPalette";
import { NotificationRegion } from "../components/NotificationRegion";
import { navigationItems, type AppRoute } from "./navigation";

export function AppShell() {
  const [activeRoute, setActiveRoute] = useState<AppRoute>("library");
  const [isCommandPaletteOpen, setCommandPaletteOpen] = useState(false);
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
          <section aria-label={`${navigationItems.find((item) => item.id === activeRoute)?.label}内容`} className="empty-state">
            <h2>{navigationItems.find((item) => item.id === activeRoute)?.label}</h2>
            <p>当前没有可显示的内容。</p>
          </section>
        </main>
      </div>

      <NotificationRegion />
      {isCommandPaletteOpen ? <CommandPalette onClose={closeCommandPalette} /> : null}
    </div>
  );
}
