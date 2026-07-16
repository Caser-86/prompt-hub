# Prompt Hub engineering guide

## Product authority

- Treat `docs/prompt-hub-product-spec.md` as the sole product-requirement baseline.
- Follow `docs/superpowers/plans/2026-07-15-prompt-hub-model-batched-execution.md` in phase order.
- External and AI writes may create inbox drafts only; never overwrite published prompts.
- Keep core prompt management offline-capable and independent of model credentials.

## Required workflow

- Use test-driven development for behavior: write a failing test, observe the expected failure, implement the minimum change, then rerun the focused and related suites.
- Add a forward migration and migration test for every database schema change.
- Never store or log credentials, raw authorization headers, or unredacted prompt bodies.
- Do not claim completion without fresh command output.

## Skill library extension

- Treat the Skill library as a first-class asset type alongside prompts, not as prompt text or an automatic installer.
- Follow `docs/skill-library-design.md` before adding Skill collection, scanning, import, installation, update, or removal behavior.
- Skills from local folders, Git repositories, archives, and URLs are untrusted: preview and audit them before installation; never execute bundled scripts during collection or preview.
- Installation requires explicit user confirmation, a clear target directory, collision handling, and a rollback-capable backup. Never overwrite an installed Skill by default.
- Keep the Skill library local-first and usable without model credentials. AI may classify or summarize only after the user enables it; it must never silently install, execute, publish, or delete a Skill.

## Model allocation

- Use **GPT-5.6 Sol, xhigh** for product architecture, database/security boundaries, untrusted Skill handling, destructive or installation flows, cross-crate changes, and final release review.
- Use **GPT-5.6 Terra, high** for bounded implementation slices, UI integration, migrations already specified by the design, test repairs, and code review follow-up.
- Use **GPT-5.6 Terra, medium** for documentation, test fixtures, metadata cleanup, small styling changes, and mechanical refactors with a clear acceptance test.
- Use **GPT-5.6 Luna, medium** only for bulk non-authoritative work such as deduplicating candidate metadata or preparing import manifests; a Sol or Terra review is required before persistent writes.
- If the active model cannot establish a fact from code, tests, or authoritative sources, it must state that it cannot confirm it and stop rather than inventing an answer or performing a risky action.

## Verification commands

```powershell
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
pnpm test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --filter @prompt-hub/desktop build
```

On Windows shells where MSVC tools are not already active, run Rust commands through Visual Studio Build Tools:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:VSLANG = "1033"
$vs = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat'
$cmd = 'call "' + $vs + '" -arch=x64 -host_arch=x64 && cargo test --workspace'
cmd.exe /d /c $cmd
```
