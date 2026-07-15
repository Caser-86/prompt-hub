# AI Optimization Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Optimize a selected prompt into an independently reviewable inbox draft.

**Architecture:** The desktop command reads the selected prompt and delegates to the existing AI draft generator. The UI invokes that command from the detail view and renders a text-only diff before the user enters the existing inbox review flow.

**Tech Stack:** Rust, Tauri 2, React, TypeScript, Vitest and Cargo tests.

## Global Constraints

- Credentials stay in system credential storage.
- AI can create inbox drafts only; it cannot change the selected or published prompt.
- Prompt bodies are never logged or rendered as HTML.

### Task 1: Create inbox-only optimization drafts

**Files:** `apps/desktop/src-tauri/src/commands.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/tests/prompt_workflow.rs`.

- [ ] Write a failing test proving optimization leaves a published prompt unchanged and creates a new inbox draft with `AI 优化` source evidence.
- [ ] Run `cargo test -p prompt-hub-desktop --test prompt_workflow optimize` and confirm failure.
- [ ] Add `optimize_ai_prompt` with the existing provider, credential and failure behavior, registering it in the Tauri handler.
- [ ] Rerun the focused test and commit `feat(ai): create inbox drafts for prompt optimization`.

### Task 2: Review result in the detail view

**Files:** `packages/contracts/src/index.ts`, `packages/contracts/src/index.test.ts`, `apps/desktop/src/features/ai/`, `apps/desktop/src/app/AppShell.tsx`.

- [ ] Write failing contract and component tests for the optimization command, escaped before/after diff, and an inbox-review link.
- [ ] Run the focused tests and confirm failure.
- [ ] Implement the typed command and local text-only review; preserve instruction fields if the request fails.
- [ ] Rerun frontend tests and commit `feat(ai): review optimized prompt drafts safely`.

### Task 3: Verify

- [ ] Run `pnpm lint`, `pnpm typecheck`, `pnpm test`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
