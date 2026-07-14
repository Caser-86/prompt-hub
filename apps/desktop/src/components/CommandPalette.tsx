type CommandPaletteProps = {
  onClose: () => void;
};

export function CommandPalette({ onClose }: CommandPaletteProps) {
  return (
    <div
      aria-label="命令面板"
      aria-modal="true"
      className="command-palette-backdrop"
      onMouseDown={onClose}
      role="dialog"
    >
      <section
        className="command-palette"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <label htmlFor="command-search">快速操作</label>
        <input autoFocus id="command-search" placeholder="搜索命令或提示词" />
        <button onClick={onClose} type="button">
          关闭命令面板
        </button>
      </section>
    </div>
  );
}
