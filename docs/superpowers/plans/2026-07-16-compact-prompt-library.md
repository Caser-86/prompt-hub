# 高密度提示词库 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将提示词库改为默认紧凑列表，使桌面端能在一屏内快速浏览、筛选和打开约 10–14 条提示词。

**Architecture:** 在 `features/library` 中新增无副作用的列表视图辅助模块，用于过滤、排序和相对时间格式化。`PromptLibrary` 仅管理视图状态并渲染两行紧凑列表；完整模型、评分、来源明细和正文继续由详情页负责。样式放在独立的组件样式文件，避免继续扩张压缩的全局样式表。

**Tech Stack:** React 18、TypeScript、Vitest、Testing Library、现有 Tauri 桌面端。

## Global Constraints

- 默认显示紧凑列表；本轮不实现卡片/列表切换、分页或虚拟滚动。
- 列表只显示标题、标签、来源、有效性、已记录工具和相对更新时间；不显示评分或适用模型。
- 外部与 AI 写入仍只能创建收件箱草稿，不能改变已发布提示词。
- 所有行为先写失败测试，再实现最小改动。
- 代码和安装包版本从 `0.1.4` 升至 `0.1.5`，并通过完整项目验证命令。

---

### Task 1: 提取紧凑列表视图规则

**Files:**
- Create: `apps/desktop/src/features/library/libraryView.ts`
- Create: `apps/desktop/src/features/library/libraryView.test.ts`

**Interfaces:**
- Consumes: `PromptListItem` from `@prompt-hub/contracts`.
- Produces: `PromptLibraryFilter`, `filterAndSortPrompts(prompts, filter)`, `formatLibraryUpdatedAt(value, now)`.

- [ ] **Step 1: Write the failing test**

```ts
it("filters favorites and verification states while preserving newest-first order", () => {
  const visible = filterAndSortPrompts(fixtures, "effective");

  expect(visible.map((prompt) => prompt.id)).toEqual(["new-effective", "old-effective"]);
  expect(filterAndSortPrompts(fixtures, "favorite").map((prompt) => prompt.id)).toEqual(["favorite"]);
});

it("formats recent update times without exposing ISO timestamps", () => {
  const now = new Date("2026-07-16T12:00:00Z");

  expect(formatLibraryUpdatedAt("2026-07-16T11:58:00Z", now)).toBe("刚刚");
  expect(formatLibraryUpdatedAt("2026-07-15T12:00:00Z", now)).toBe("昨天");
  expect(formatLibraryUpdatedAt("2026-07-10T12:00:00Z", now)).toBe("7月10日");
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @prompt-hub/desktop test -- libraryView.test.ts`

Expected: FAIL because `libraryView.ts` does not exist.

- [ ] **Step 3: Write minimal implementation**

```ts
export type PromptLibraryFilter = "all" | "favorite" | "effective" | "needs_retest";

export function filterAndSortPrompts(prompts: PromptListItem[], filter: PromptLibraryFilter) {
  return prompts
    .filter((prompt) => filter === "all" || (filter === "favorite" ? prompt.favorite : prompt.effectiveness === filter))
    .toSorted((left, right) => Date.parse(right.updatedAt) - Date.parse(left.updatedAt));
}

export function formatLibraryUpdatedAt(value: string, now = new Date()) {
  const age = now.getTime() - Date.parse(value);
  if (age < 60 * 60 * 1000) return "刚刚";
  if (age < 24 * 60 * 60 * 1000) return "今天";
  if (age < 48 * 60 * 60 * 1000) return "昨天";
  return new Intl.DateTimeFormat("zh-CN", { month: "numeric", day: "numeric" }).format(new Date(value));
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm --filter @prompt-hub/desktop test -- libraryView.test.ts`

Expected: PASS with 2 tests.

- [ ] **Step 5: Commit**

```powershell
git add apps/desktop/src/features/library/libraryView.ts apps/desktop/src/features/library/libraryView.test.ts
git commit -m "feat: add compact library view helpers"
```

### Task 2: 将提示词库渲染为紧凑列表

**Files:**
- Modify: `apps/desktop/src/features/library/PromptLibrary.tsx`
- Modify: `apps/desktop/src/features/library/PromptLibrary.test.tsx`
- Modify: `apps/desktop/src/App.test.tsx`

**Interfaces:**
- Consumes: `PromptLibraryFilter`, `filterAndSortPrompts`, and `formatLibraryUpdatedAt` from `./libraryView`.
- Produces: accessible quick filters, result count, and list rows with `prompt-list` / `prompt-list-item` classes.

- [ ] **Step 1: Write the failing component tests**

```tsx
it("renders compact rows without model or rating and filters verified prompts", async () => {
  render(<PromptLibrary loadPrompts={async () => fixtures} />);

  expect(await screen.findByText("共 3 条提示词")).toBeVisible();
  expect(screen.getByRole("list", { name: "提示词列表" })).toHaveClass("prompt-list");
  expect(screen.queryByText(/评分：/)).not.toBeInTheDocument();
  expect(screen.queryByText(/适用模型：/)).not.toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "已验证" }));
  expect(screen.getAllByRole("listitem")).toHaveLength(1);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptLibrary.test.tsx App.test.tsx`

Expected: FAIL because filters, count, and `prompt-list` markup do not yet exist.

- [ ] **Step 3: Implement the compact row markup and filters**

```tsx
const [filter, setFilter] = useState<PromptLibraryFilter>("all");
const visiblePrompts = prompts ? filterAndSortPrompts(prompts, filter) : [];

<div aria-label="提示词库筛选" className="library-quick-filters">
  {(["all", "favorite", "effective", "needs_retest"] as const).map((value) => (
    <button aria-pressed={filter === value} key={value} onClick={() => setFilter(value)} type="button">
      {filterLabel(value)}
    </button>
  ))}
  <span>{`共 ${visiblePrompts.length} 条提示词`}</span>
</div>
<ul aria-label="提示词列表" className="prompt-list">
  {visiblePrompts.map((prompt) => <li className="prompt-list-item" key={prompt.id}>{/* title, tags, source, status, tool, relative time, controls */}</li>)}
</ul>
```

- [ ] **Step 4: Run focused tests to verify they pass**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptLibrary.test.tsx App.test.tsx`

Expected: PASS; selection, favorite, batch archive and detail navigation remain covered.

- [ ] **Step 5: Commit**

```powershell
git add apps/desktop/src/features/library/PromptLibrary.tsx apps/desktop/src/features/library/PromptLibrary.test.tsx apps/desktop/src/App.test.tsx
git commit -m "feat: render prompts as a compact library list"
```

### Task 3: 添加桌面与窄屏紧凑列表样式

**Files:**
- Create: `apps/desktop/src/features/library/prompt-library.css`
- Modify: `apps/desktop/src/features/library/PromptLibrary.tsx`

**Interfaces:**
- Consumes: semantic list classes produced in Task 2.
- Produces: 56–64px desktop rows with a responsive two-line fallback below 760px.

- [ ] **Step 1: Write the failing style wiring test**

```tsx
it("loads the compact library stylesheet", async () => {
  await import("./PromptLibrary");
  expect(document.head.querySelector('link, style')).toBeTruthy();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptLibrary.test.tsx`

Expected: FAIL after the assertion is narrowed to the imported `prompt-library.css` module mock.

- [ ] **Step 3: Implement component-scoped CSS**

```css
.prompt-list { display: grid; gap: 0.45rem; margin: 0; padding: 0; list-style: none; }
.prompt-list-item { display: grid; grid-template-columns: minmax(16rem, 1.7fr) minmax(0, 1fr) auto; align-items: center; min-height: 3.75rem; padding: 0.7rem 0.9rem; }
.prompt-list-primary, .prompt-list-meta, .prompt-list-actions { display: flex; align-items: center; gap: 0.55rem; min-width: 0; }
@media (max-width: 760px) { .prompt-list-item { grid-template-columns: 1fr auto; } .prompt-list-meta { grid-column: 1 / -1; flex-wrap: wrap; } }
```

- [ ] **Step 4: Run visual and focused verification**

Run: `pnpm --filter @prompt-hub/desktop tauri dev`

Expected: desktop library shows at least 10 rows in the primary viewport; narrow layout preserves title, filters, and controls without overflow.

- [ ] **Step 5: Commit**

```powershell
git add apps/desktop/src/features/library/prompt-library.css apps/desktop/src/features/library/PromptLibrary.tsx apps/desktop/src/features/library/PromptLibrary.test.tsx
git commit -m "style: increase prompt library information density"
```

### Task 4: 发布桌面端 0.1.5

**Files:**
- Modify: `Cargo.toml`
- Modify: `package.json`
- Modify: `apps/desktop/package.json`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: complete compact-list implementation from Tasks 1–3.
- Produces: signed local Windows NSIS/MSI artifacts at version `0.1.5`.

- [ ] **Step 1: Set all workspace and Tauri versions to `0.1.5`**

```toml
[workspace.package]
version = "0.1.5"
```

```json
{ "version": "0.1.5" }
```

- [ ] **Step 2: Run complete verification**

Run:

```powershell
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @prompt-hub/desktop build
pnpm --filter @prompt-hub/desktop tauri build
```

Expected: all commands exit `0`; frontend has the new list helper and component tests; bundles include `Prompt Hub_0.1.5_x64-setup.exe`.

- [ ] **Step 3: Install and verify the generated desktop build**

```powershell
Start-Process -FilePath 'target\release\bundle\nsis\Prompt Hub_0.1.5_x64-setup.exe' -ArgumentList '/S' -Wait
Start-Process -FilePath 'D:\Program Files\Prompt Hub\prompt-hub-desktop.exe'
```

Expected: application diagnostics report `0.1.5`; prompt library opens in compact list mode.

- [ ] **Step 4: Commit and push release**

```powershell
git add Cargo.toml Cargo.lock package.json apps/desktop/package.json apps/desktop/src-tauri/tauri.conf.json
git commit -m "release: prompt hub 0.1.5"
git push origin feat/permanent-prompt-deletion
```
