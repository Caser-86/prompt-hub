# Professional Desktop UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a professional, accessible Prompt Hub desktop interface while preserving all offline prompt-management behavior.

**Architecture:** Import the existing global stylesheet from the React entry point, then apply reusable semantic CSS classes to the existing shell and feature components. No Tauri commands, contracts, storage behavior, or dependencies change.

**Tech Stack:** React 19, TypeScript, Vitest, Testing Library, Vite, Tauri 2, CSS.

## Global Constraints

- Keep prompt management offline-capable and independent of model credentials.
- Do not store or log credentials, raw authorization headers, or unredacted prompt bodies.
- Preserve existing ARIA labels, command-palette keyboard behavior, and focus return.
- Add no third-party UI framework or fabricated prompt data.
- Target normal-text contrast of at least 4.5:1; status labels retain visible text.

---

### Task 1: Restore the global visual system

**Files:** Modify `apps/desktop/src/main.tsx`, `apps/desktop/src/App.test.tsx`, and `apps/desktop/src/styles.css`.

**Interfaces:** Produces a CSS-loaded entry point and `button-primary`, `button-secondary`, `surface-card`, `field-stack`, and `status-pill` classes.

- [ ] **Step 1: Write the failing test** — add `readFileSync(resolve(import.meta.dirname, "main.tsx"), "utf8")` assertion in `App.test.tsx` requiring `import "./styles.css"`.
- [ ] **Step 2: Verify red** — run `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`; expected failure because `main.tsx` has no stylesheet import.
- [ ] **Step 3: Implement minimum change** — put `import "./styles.css";` before the App import in `main.tsx`. Replace CSS root colors with `--canvas: #f7f6f2`, `--surface: #fff`, `--sidebar: #151922`, `--ink: #202432`, `--muted: #687083`, `--line: #e4e5ea`, `--accent: #635bff`, `--accent-strong: #4d45e7`, and `--success: #16825d`. Define indigo primary and outlined secondary buttons, white surface cards, field stacks, and text-labelled status pills.
- [ ] **Step 4: Verify green** — rerun `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`; expected PASS.
- [ ] **Step 5: Commit** — `git add apps/desktop/src/main.tsx apps/desktop/src/App.test.tsx apps/desktop/src/styles.css` then `git commit -m "fix(desktop): load the global visual system"`.

### Task 2: Rebuild the dense workspace shell

**Files:** Modify `apps/desktop/src/app/AppShell.tsx`, `apps/desktop/src/App.test.tsx`, and `apps/desktop/src/styles.css`.

**Interfaces:** Consumes `navigationItems` and existing route/editor state. Produces `workspace-header`, `navigation-link`, `navigation-marker`, `content-frame`, and an existing-state-driven global create action.

- [ ] **Step 1: Write the failing test** — render `App` and assert `screen.getByRole("banner")` has `workspace-header`; assert the `新建提示词` button has `button-primary`.
- [ ] **Step 2: Verify red** — run `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`; expected failure because no class/global create button exists.
- [ ] **Step 3: Implement minimum change** — set header class to `app-header workspace-header`; add a `button-primary header-create` button which sets route `library`, clears selected prompt, and opens the existing editor. Give anchors `navigation-link`, add `<span aria-hidden="true" className="navigation-marker">{item.label.slice(0, 1)}</span>`, apply `content-frame` to main, and remove the route-agnostic hero. CSS: 15.5rem dark sidebar, 4.75rem header, indigo active nav, 96rem content width, horizontal nav at 700px.
- [ ] **Step 4: Verify green** — rerun App tests; existing command palette and focus tests pass.
- [ ] **Step 5: Commit** — add the three files and commit `feat(desktop): redesign the application workspace shell`.

### Task 3: Make library and detail scanning efficient

**Files:** Modify `apps/desktop/src/features/library/PromptLibrary.tsx`, `apps/desktop/src/features/library/PromptLibrary.test.tsx`, `apps/desktop/src/app/AppShell.tsx`, and `apps/desktop/src/styles.css`.

**Interfaces:** Retains `PromptListItem` callbacks. Produces `library-toolbar`, `prompt-grid`, `prompt-card`, `prompt-card-meta`, `empty-library-state`, and `prompt-details-layout`.

- [ ] **Step 1: Write failing tests** — loaded list has `prompt-grid`, its item has `prompt-card`; empty library has a `创建第一条提示词` button and clicking calls `onCreate` once.
- [ ] **Step 2: Verify red** — run `pnpm --filter @prompt-hub/desktop test -- PromptLibrary.test.tsx`; expected failure for missing classes/action.
- [ ] **Step 3: Implement minimum change** — use `library-toolbar`; make creation button primary; apply `prompt-grid` to list and `prompt-card surface-card` to items; group source/tool/model/effectiveness/rating/time in `prompt-card-meta`. Replace empty text with `empty-library-state surface-card`, heading `从第一条提示词开始`, explanatory copy, and existing `onCreate` callback. Wrap selected detail's already-rendered metadata/lifecycle/history/content in `prompt-details-layout`. CSS grid: `repeat(auto-fit,minmax(21rem,1fr))`, 1rem gap, one column at 760px.
- [ ] **Step 4: Verify green** — run `pnpm --filter @prompt-hub/desktop test -- PromptLibrary.test.tsx App.test.tsx`; expected PASS.
- [ ] **Step 5: Commit** — add changed library, tests, shell and CSS; commit `feat(library): present prompts as workspace cards`.

### Task 4: Unify search, inbox, and settings

**Files:** Modify `apps/desktop/src/features/search/PromptSearch.tsx`, `apps/desktop/src/features/inbox/InboxImport.tsx`, `apps/desktop/src/features/settings/SettingsPage.tsx`, `apps/desktop/src/App.test.tsx`, and `apps/desktop/src/styles.css`.

**Interfaces:** Retains existing props and produces `page-header`, `filter-panel`, `search-results`, `import-panel`, `inbox-draft-list`, and `settings-stack`.

- [ ] **Step 1: Write failing test** — after opening search, `搜索提示词` heading's closest header has `page-header`; after opening inbox, `screen.getByLabelText("导入操作")` has `import-panel`.
- [ ] **Step 2: Verify red** — run `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`; expected failure because content is ungrouped.
- [ ] **Step 3: Implement minimum change** — search title/help goes in `header.page-header`, filters in non-submitting `form.filter-panel.surface-card`, results in `search-results`. Inbox title/help goes in `page-header`, three import controls in `section[aria-label="导入操作"].import-panel.surface-card`, imports become primary, drafts gain `inbox-draft-list`. Settings children keep order/props inside `main.settings-stack`. Add responsive filter grid and native control focus/border rules.
- [ ] **Step 4: Verify green** — run `pnpm --filter @prompt-hub/desktop test -- App.test.tsx PromptSearch.test.tsx InboxImport.test.tsx`; expected PASS.
- [ ] **Step 5: Commit** — add all changed files; commit `feat(desktop): unify secondary workspace screens`.

### Task 5: Verify rendered desktop UI

**Files:** Change a component/test only if an actual screenshot reveals a reproducible regression.

- [ ] **Step 1: Full frontend validation** — run `pnpm lint`, `pnpm typecheck`, `pnpm --filter @prompt-hub/desktop test`, and `pnpm --filter @prompt-hub/desktop build`; expected all exit 0.
- [ ] **Step 2: Visual acceptance** — run `pnpm --filter @prompt-hub/desktop tauri dev`; capture empty library, populated library, detail, search, inbox, settings. Inspect visible CSS, shell hierarchy, controls, empty action and focus indication. Screenshots do not prove full accessibility compliance.
- [ ] **Step 3: Correct only evidence-backed defects** — first add the nearest focused test and observe it fail; make minimum CSS/TSX correction; rerun focused test and affected full check.
- [ ] **Step 4: Commit** — add verified UI changes and commit `test(desktop): verify professional workspace presentation`.

## Self-review

- Tasks 1–4 cover every confirmed screen and keep data interfaces unchanged.
- Task 5 requires build and screenshot evidence before completion.
- No unresolved placeholder, unspecified dependency, or unbounded product feature is included.
