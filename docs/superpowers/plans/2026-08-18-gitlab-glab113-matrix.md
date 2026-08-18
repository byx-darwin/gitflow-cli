# GitLab glab 1.113.0 Compatibility Matrix Update — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add glab 1.113.0 to the GitLab compatibility matrix `tested_versions` and regenerate the published matrix Markdown.

**Architecture:** A bounded data+docs change. The single source of truth is `crates/core/resources/compatibility-matrix.json`, embedded at compile time via `include_str!` in `crates/core/src/compatibility.rs`. `docs/compatibility-matrix.md` is generated from that JSON by `cargo run -p gf-core --example gen_compat_matrix` (Makefile target `compatibility-matrix`). No Rust code changes; `min_version` unchanged at 1.30.0.

**Tech Stack:** Rust workspace (gitflow-core), serde_json, Makefile.

**Spec:** `docs/superpowers/specs/2026-08-18-gitlab-glab113-matrix-design.md`

## Global Constraints

- `min_version` for GitLab stays `"1.30.0"` — glab 1.113.0's breaking changes were already fixed in gf by Issue #199 / PR #201.
- Do NOT modify GitHub or GitCode platform entries.
- `docs/compatibility-matrix.md` must be regenerated via `make compatibility-matrix` — never hand-edited (file header states this).
- `updated_at` in the matrix JSON must be `2026-08-18`.
- No contract fixture changes (GitLab fixtures are generic `v1`, no per-version fixtures exist).
- Rust source is unchanged, so full gate set (`cargo build`/`clippy`/`fmt`) is still run per repo policy since `include_str!` embeds the JSON; scoped verification is `cargo test -p gf-core`.

---

### Task 1: Update the compatibility matrix JSON

**Files:**
- Modify: `crates/core/resources/compatibility-matrix.json` (GitLab platform block)

**Interfaces:**
- Consumes: existing JSON schema (`schema_version`, `updated_at`, `gitflow_cli_version`, `platforms[]`).
- Produces: embedded data consumed by `platform_compatibility()` / `platform_requirement()` in `crates/core/src/compatibility.rs`.

- [ ] **Step 1: Edit `crates/core/resources/compatibility-matrix.json`**

In the `GitLab` platform block, change `"tested_versions": ["1.111.0", "1.112.0"]` to `"tested_versions": ["1.111.0", "1.112.0", "1.113.0"]`, and change top-level `"updated_at"` from `"2026-08-09"` to `"2026-08-18"`. Keep `"min_version": "1.30.0"` and all `features` unchanged. Resulting GitLab block:

```json
{
  "name": "GitLab",
  "identifier": "gitlab",
  "cli_binary": "glab",
  "min_version": "1.30.0",
  "tested_versions": ["1.111.0", "1.112.0", "1.113.0"],
  "install_url": "https://gitlab.com/gitlab-org/cli#installation",
  "doc_link": "https://gitlab.com/gitlab-org/cli/-/blob/main/docs/",
  "features": { "issue": true, "pr": true, "label": true, "milestone": true, "release": true, "pipeline": true, "review": true, "auth": true }
}
```

- [ ] **Step 2: Validate JSON parses**

Run: `jq empty crates/core/resources/compatibility-matrix.json`
Expected: no output, exit code 0.

- [ ] **Step 3: Run core tests (embedded JSON is compile-time validated)**

Run: `cargo test -p gf-core`
Expected: all pass, including `test_should_load_all_three_platforms` (3 platforms).

- [ ] **Step 4: Commit**

```bash
git add crates/core/resources/compatibility-matrix.json
git commit -m "chore(deps): add glab 1.113.0 to compatibility matrix"
```

---

### Task 2: Regenerate the compatibility matrix Markdown

**Files:**
- Modify: `docs/compatibility-matrix.md` (auto-generated)

**Interfaces:**
- Consumes: the updated `crates/core/resources/compatibility-matrix.json` from Task 1.
- Produces: regenerated `docs/compatibility-matrix.md` with GitLab row showing `1.111.0, 1.112.0, 1.113.0`.

- [ ] **Step 1: Regenerate via Makefile target**

Run: `make compatibility-matrix`
Expected: output `Generated docs/compatibility-matrix.md`; GitLab row now reads `≥ 1.30.0 | 1.111.0, 1.112.0, 1.113.0`.

- [ ] **Step 2: Verify the GitLab row**

Run: `grep -n "GitLab" docs/compatibility-matrix.md`
Expected: `| GitLab | \`glab\` | ≥ 1.30.0 | 1.111.0, 1.112.0, 1.113.0 | ... |`

- [ ] **Step 3: Format and lint**

Run: `cargo fmt --check && cargo clippy -- -D warnings`
Expected: clean (no Rust source changed; confirms nothing regressed).

- [ ] **Step 4: Commit**

```bash
git add docs/compatibility-matrix.md
git commit -m "docs: regenerate compatibility matrix with glab 1.113.0"
```

---

### Task 3: Open PR and verify

**Files:** none (remote operation).

- [ ] **Step 1: Create PR with `Closes #198`**

```bash
git push -u origin HEAD
gf pr create --title "chore(deps): add glab 1.113.0 to compatibility matrix" --body "Closes #198"
```

- [ ] **Step 2: Run smoke test to confirm no regression**

Run: `make smoke-test-gitlab`
Expected: `54 passed, 0 failed, 5 skipped` (matches the pre-change baseline for glab 1.113.0).

- [ ] **Step 3: Record `pr_url` and `tests_passed` in the workflow contract**

Expected: contract `wf-2026-08-18-003` Phase 3 evidence updated with `pr_url` and `tests_passed: true`.
