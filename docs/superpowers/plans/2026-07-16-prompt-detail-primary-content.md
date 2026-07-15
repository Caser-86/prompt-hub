# Prompt Detail Primary Content Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the current prompt body the first, largest and immediately usable element in the prompt-detail view.

**Architecture:** `AppShell` remains the route owner and obtains the current body from the final history item. It will arrange the detail into a primary reading column and a compact sidebar, while `PromptHistory` becomes a closed native disclosure panel so historical prompt bodies do not precede the current body. CSS supplies the desktop two-column layout and a single-column responsive fallback; no desktop command, contract, persistence or lifecycle behavior changes.

**Tech Stack:** React 19, TypeScript, Vitest, Testing Library, CSS, Tauri 2.

## Global Constraints

- Keep prompt management offline-capable and independent of model credentials.
- Preserve prompt contents, sources, metadata, version recovery, lifecycle actions and all existing desktop commands.
- Do not log unredacted prompt bodies in diagnostics or test output.
- Preserve accessible names and keyboard operation; native `<details>` remains keyboard-operable.
- Add no UI library, data field, network call or backend command.

---

### Task 1: Expose the current prompt body before secondary detail content

**Files:**
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/App.test.tsx`

**Interfaces:**
- Consumes: `PromptHistoryItem[]` from `desktopCommands.promptHistory`.
- Produces: a `section[aria-label="提示词正文"]` containing the latest history body and a `section[aria-label="提示词主操作"]` containing `PromptContentActions`.

- [ ] **Step 1: Write the failing test** — after opening `代码审查`, assert `screen.findByLabelText("提示词正文")` contains `请审查这段代码` and the `提示词主操作` region contains `复制提示词正文`.

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`

Expected: the test fails because the page only renders the body inside version history and has no `提示词正文` region.

- [ ] **Step 3: Implement the minimum view structure** — in `AppShell.tsx`, derive `const currentBody = history?.at(-1)?.body ?? ""` in the selected-prompt branch. Render title, primary actions and a labeled `提示词正文` card before metadata, lifecycle, history and AI controls. Move `PromptContentActions` beside the title/body rather than after `PromptHistory`.

- [ ] **Step 4: Run test to verify it passes**

Run: `pnpm --filter @prompt-hub/desktop test -- App.test.tsx`

Expected: PASS with the current body and copy control present in their labeled primary regions.

### Task 2: Put metadata and secondary operations in a sidebar and collapse history

**Files:**
- Modify: `apps/desktop/src/app/AppShell.tsx`
- Modify: `apps/desktop/src/features/history/PromptHistory.tsx`
- Modify: `apps/desktop/src/features/history/PromptHistory.test.tsx`
- Modify: `apps/desktop/src/styles.css`

**Interfaces:**
- Consumes: existing prompt metadata and `PromptHistory` props.
- Produces: `prompt-detail-main`, `prompt-detail-sidebar`, `prompt-detail-body`, `prompt-detail-secondary` classes, and a closed `details` disclosure named `版本历史`.

- [ ] **Step 1: Write the failing history disclosure test** — render `PromptHistory` with one version, find the closest `details` ancestor of `版本历史`, and assert that it has no `open` attribute.

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptHistory.test.tsx`

Expected: FAIL because `版本历史` is currently rendered in an always-open `section`.

- [ ] **Step 3: Implement the minimum secondary hierarchy** — wrap `PromptHistory` in a closed `<details className="prompt-history-disclosure">` with a `<summary>版本历史</summary>` and preserve all restore/diff behavior inside it. In `AppShell.tsx`, place source/time/compatibility/effectiveness/metadata editor in `aside.prompt-detail-sidebar`; place lifecycle, history and AI review in `section.prompt-detail-secondary` below the primary grid. In CSS, use `minmax(0, 1fr) minmax(18rem, 22rem)` at desktop widths and a single column below 900px. Style the prompt body with `white-space: pre-wrap`, a bounded height and internal scroll.

- [ ] **Step 4: Run focused tests to verify they pass**

Run: `pnpm --filter @prompt-hub/desktop test -- PromptHistory.test.tsx App.test.tsx`

Expected: PASS; history remains recoverable after expansion, while the current prompt body is already visible in AppShell.

### Task 3: Verify the redesigned screen without changing behavior

**Files:**
- Modify only if a test or rendered screen identifies a reproducible regression.

**Interfaces:**
- Consumes: the production desktop application and existing Tauri commands.
- Produces: verification evidence for the body-first hierarchy at desktop and narrow widths.

- [ ] **Step 1: Run frontend quality checks**

Run: `pnpm lint`, `pnpm typecheck`, `pnpm test`, and `pnpm --filter @prompt-hub/desktop build`.

Expected: all commands exit with status 0.

- [ ] **Step 2: Inspect the rendered application**

Run: `pnpm --filter @prompt-hub/desktop tauri dev`

Expected: opening a prompt shows title, copy action and current body without scrolling; metadata is in the adjacent sidebar; history and AI tools do not occupy the first screen.

- [ ] **Step 3: Commit verified changes**

Run: `git add apps/desktop/src/app/AppShell.tsx apps/desktop/src/App.test.tsx apps/desktop/src/features/history/PromptHistory.tsx apps/desktop/src/features/history/PromptHistory.test.tsx apps/desktop/src/styles.css docs/superpowers/plans/2026-07-16-prompt-detail-primary-content.md` followed by `git commit -m "feat(desktop): prioritize prompt content in details"`.

Expected: the commit contains only detail-layout work and its plan; unrelated existing changes remain unstaged.

## Self-Review

- The first task makes the current body and copy action visible before secondary information.
- The second task moves metadata to a sidebar and turns history into an on-demand disclosure without removing recovery behavior.
- The third task requires focused, full and rendered verification before any completion claim.
- No new storage, credential, network, source or lifecycle behavior is introduced.
