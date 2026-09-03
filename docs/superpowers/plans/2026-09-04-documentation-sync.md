# Documentation and GitHub Synchronization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the repository documentation easy to navigate and synchronize the current, verified product branch to GitHub without publishing unverified artifacts.

**Architecture:** Keep `README.md` as the five-minute entry point and add `docs/README.md` as the documentation map. Preserve the product specification as the authority, while grouping user, operations, security, design, and release-evidence documents by purpose rather than moving historical files. Push the completed feature branch after a fresh repository verification; do not merge into `main` or create a release unless explicitly requested.

**Tech Stack:** Markdown, Git, GitHub CLI, pnpm/Vitest/Playwright, Rust/Cargo, Tauri.

**Spec:** `docs/prompt-hub-product-spec.md`

## Global Constraints

- `docs/prompt-hub-product-spec.md` is the sole product-requirement baseline.
- Core prompt management remains offline-capable and independent of model credentials.
- External and AI writes may create inbox drafts only; never overwrite published prompts.
- Documentation and release evidence must not contain credentials, authorization headers, databases, backups, or unredacted prompt bodies.
- A GitHub synchronization must preserve the current feature branch and must not silently merge into `main`.

---

### Task 1: Establish the documentation map

**Files:**
- Create: `docs/README.md`
- Modify: `README.md`

**Interfaces:**
- `README.md` links to `docs/README.md` for the complete map.
- `docs/README.md` links only to tracked, user-facing or maintainable project documents.

- [x] **Step 1: Write the failing documentation check**

  Verify that the current README has no single documentation index:

  ```powershell
  rg -n "docs/README|文档索引" README.md docs
  ```

  Expected: no matching documentation index entry.

- [x] **Step 2: Run it to verify the gap**

  Run the command above and record the absence of an index before editing.

- [x] **Step 3: Add the documentation index and entry-point link**

  Create a Chinese index with these sections and links:

  - 开始使用：`user-guide.md`, `import-formats.md`, `mcp-setup.md`
  - 产品与设计：`prompt-hub-product-spec.md`, `skill-library-design.md`
  - 数据、安全与恢复：`privacy.md`, `security-review.md`, `recovery-runbook.md`
  - 发布与证据：`release-checklist.md`, `release-evidence/README.md`, `search-baseline.md`
  - 计划与规格：`superpowers/plans/`, `superpowers/specs/`
  - 可复用导入种子：`import-seeds/`

  Update the README documentation section to lead with the index and state that historical plans/specs are retained for traceability.

- [x] **Step 4: Run the documentation check to verify the index exists**

  ```powershell
  rg -n "docs/README|文档索引" README.md docs/README.md
  git diff --check
  ```

  Expected: both files contain the index link and `git diff --check` exits 0.

- [x] **Step 5: Commit the documentation map**

  ```powershell
  git add README.md docs/README.md
  git commit -m "docs: add project documentation index"
  ```

### Task 2: Align release and change documentation

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/release-checklist.md`
- Modify: `docs/release-evidence/README.md`
- Modify: `docs/release-evidence/0.1.10/packaging.md`

**Interfaces:**
- Release documents point to the same 0.1.10 candidate evidence and distinguish local candidate use from a public release.
- The changelog documents the documentation index and current release limitations without claiming signing, updates, or clean-profile acceptance.

- [x] **Step 1: Check for stale release claims and broken links**

  ```powershell
  rg -n "公开正式|已完成|CHANGELOG|release-evidence|workingTree|NotSigned" CHANGELOG.md docs/release-checklist.md docs/release-evidence
  ```

- [x] **Step 2: Update the minimal release wording**

  Add a `Documentation` subsection under `Unreleased`, link the documentation index, and keep all 0.1.10 limitations explicit. Do not mark unchecked release gates as complete.

- [x] **Step 3: Validate the release links and sensitive-content guard**

  ```powershell
  rg -n "docs/README|release-evidence/0.1.10|NotSigned|代码签名|自动更新" README.md CHANGELOG.md docs
  rg -n "(credential|authorization header|private key|unredacted prompt|database backup)" README.md CHANGELOG.md docs -g '*.md'
  ```

  Expected: release links and limitations are present; the sensitive-content search returns no credentials or private database paths.

- [x] **Step 4: Commit aligned release documentation**

  ```powershell
  git add CHANGELOG.md docs/release-checklist.md docs/release-evidence/README.md docs/release-evidence/0.1.10/packaging.md
  git commit -m "docs: align release evidence and limitations"
  ```

### Task 3: Verify and synchronize the feature branch

**Files:**
- Verify: all tracked files in the worktree
- Update: no generated database, backup, installer, or `build-info.json` files

**Interfaces:**
- The branch remains `feat/permanent-prompt-deletion` and tracks `origin/feat/permanent-prompt-deletion`.
- GitHub receives the verified commits and the working tree is clean after commit.

- [x] **Step 1: Run the required verification suite**

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

- [x] **Step 2: Check repository hygiene**

  ```powershell
  git diff --check
  git status --short --branch
  git ls-files | rg "(^|/)(node_modules|target|dist|test-results|.*\\.(db|db-shm|db-wal))($|/)"; if ($LASTEXITCODE -eq 1) { exit 0 }
  ```

  Expected: no whitespace errors, no generated local data tracked, and only intentional source/docs changes remain before commit.

- [x] **Step 3: Commit the synchronized product changes**

  ```powershell
  git add -A
  git status --short
  git commit -m "chore: synchronize formal product documentation and fixes"
  ```

- [x] **Step 4: Push the feature branch to GitHub**

  ```powershell
  git push -u origin feat/permanent-prompt-deletion
  git status --short --branch
  git ls-remote --heads origin feat/permanent-prompt-deletion
  ```

  Expected: push exits 0, the branch is clean and up to date, and the remote hash equals local `HEAD`.

- [x] **Step 5: Verify GitHub repository metadata and branch visibility**

  ```powershell
  gh repo view Caser-86/prompt-hub --json url,visibility,defaultBranchRef,description,repositoryTopics
  gh api repos/Caser-86/prompt-hub/branches/feat/permanent-prompt-deletion --jq '{name:.name,sha:.commit.sha,protected:.protected}'
  ```

  Expected: repository remains private, description/topics are present, and the pushed branch points to the committed local `HEAD`.
