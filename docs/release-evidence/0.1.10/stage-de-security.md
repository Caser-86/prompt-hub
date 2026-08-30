# Stage D/E and MCP security verification evidence

Date: 2026-08-29
Branch: `codex/legacy-v5-schema-recovery`
Scope: MCP published/inbox authority, controlled Skill installation, and immutable Git Skill snapshots.

## Implemented boundaries

### MCP

- The STDIO server exposes only `search_prompts`, `get_prompt`, `render_prompt`, and `save_prompt_draft`.
- Runtime arguments are validated against the same versioned JSON Schemas returned by `tools/list`; unknown fields and lifecycle overrides are rejected.
- Search, get, and render return published prompts only. Inbox, archived, and soft-deleted prompts are denied.
- `save_prompt_draft` can create only a new inbox draft and cannot select or overwrite another lifecycle state.
- One JSON-RPC input line is limited to 1 MiB. Oversized lines are drained, rejected without echoing their content, and the next request can still be processed.

### Skill installation

- Collection and installation never execute Skill files. Extensionless/shebang scripts, files under `scripts/`, hidden scripts, binary files, and invalid UTF-8 files are classified independently.
- Installation requires an approved asset and a matching reviewed content hash. Existing destinations fail closed unless backup replacement is explicitly selected and confirmed again in the UI.
- If the files were installed but the installation record cannot be persisted, the new tree is removed and the old backup is restored. Automatic rollback refuses to delete a tree that changed after installation.

### Immutable Git snapshots

- Sources are restricted to strict public `https://github.com/<owner>/<repo>` URLs and complete 40-character commit SHAs.
- Git commands ignore ambient system/global config and command config injection, disable credential helpers, terminal prompts, hooks, submodule recursion, and `file`/`ext` protocols.
- The tree is listed without checkout. File count (512), single file size (2 MiB), total size (16 MiB), depth (12), tree-listing output, modes, object IDs, and paths are checked before blob materialization.
- Blob output is bounded and must match the size declared by the immutable tree. Symlinks and submodules are rejected.

## Commits

- `30ef85c` — MCP runtime schema, published-only reads, and bounded STDIO.
- `bd62113` — Skill scan classification, install rollback, and Git process/resource hardening.
- `e589bf0` — opt-in real public immutable Git snapshot verification.
- `e26fbed` — Skill end-to-end review, conflict, confirmation, replacement, and text-only preview coverage.
- `18f6520` — invalid Git candidates are rejected before Skill persistence.

## Commands run

| Command | Result |
| --- | --- |
| `pnpm install --frozen-lockfile` | PASS |
| `pnpm lint` | PASS |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS — contracts 22, desktop 61 |
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — only the explicit performance/network tests remain ignored by default; Skill service suite now 6 tests |
| `pnpm --filter @prompt-hub/desktop build` | PASS |
| `cargo test -p prompt-skill --test git snapshots_a_real_public_skill_at_an_immutable_commit_without_checkout -- --ignored --exact` | PASS against `openai/skills` commit `49f948faa9258a0c61caceaf225e179651397431` |
| `pnpm exec playwright test --reporter=line` | PASS — 4 tests, including the Skill workflow |
| `git ls-remote --heads origin` | PASS — public HTTPS Git connectivity verified |

The desktop Vitest suite still emits the pre-existing React `act(...)` warnings from `App.test.tsx`; assertions pass. No credential, authorization header, or raw prompt body was written to this evidence.

## Residual boundaries

- Prompt Hub trusts the installed `git` executable itself; it constrains Git configuration and protocols but does not sandbox a compromised local Git binary.
- A Skill is inert while collected, reviewed, and copied by Prompt Hub. A separate tool may execute it after installation, so approval remains a user security decision.
- Automatic subscription/update, archive import, marketplace trust, signed Windows packaging, and a real external MCP-client acceptance test are outside this stage.
