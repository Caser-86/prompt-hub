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
