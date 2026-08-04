# Skill 目录重命名 `gitflow-*` → `gf-*` 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将全部 26 个 skill 目录及其引用从 `gitflow-*` 重命名为 `gf-*`，使 skill 命名与二进制 `gf` 一致。

**Architecture:** 采用批量重命名（方案 A）。分四层执行：① Rust 源码过滤逻辑与测试断言；② Makefile/install.sh/hooks 脚本；③ `skills/` + `.claude/skills/` 目录与文档文件重命名；④ 全局内容白名单替换。最后统一验证。

**Tech Stack:** Rust 2024 workspace (`gf` crate)、git、shell、Markdown 文档。

## Global Constraints

- **`gitflow-cli`（GitHub 仓库 URL）绝不改动** —— 如 `byx-darwin/gitflow-cli` 在 README、Rust 源码、CHANGELOG 中的出现均保留。
- **CLI 配置标识符保留** —— `skills.rs` 中 `matcher: "gitflow"`、`gitflow.co_contribution` 是 settings.json/hook 配置字段，**不属于 skill 名，不改**。
- 替换采用 **26 个 skill 名白名单**，按**名称长度降序**执行（最长先替换，避免 `gf-issue` 误伤 `gf-issue-create`）。
- 排除目录：`.cache/`、`target/`、`.git/`。
- `.claude/skills/` 不被 git 跟踪（本地安装副本），仅用普通 `mv` 同步，不进 commit。
- 禁止 `cargo clean`；禁止写 TODO/占位代码。

**Skill 名白名单（26 个，替换目标均为 `gf-<name>`）：**
```
auth, autoreport-bug, commit, issue, issue-create, issue-review, issue-triage,
label-milestone, label-stats, pipeline-analyzer, pr, pr-apply-feedback,
pr-create, pr-inline-review, pr-review, precommit, quality, regression,
release, release-helper, repo, repo-onboarding, review, security-check,
weekly-report, workflow
```

---

### Task 1: 更新 Rust 源码 `skills.rs` 的 skill 前缀过滤逻辑

**Files:**
- Modify: `apps/cli/src/commands/skills.rs:296`（install 过滤）
- Modify: `apps/cli/src/commands/skills.rs:371`（bundled install 分组）
- Modify: `apps/cli/src/commands/skills.rs:593`（list 过滤）
- Modify: `apps/cli/src/commands/skills.rs:630`（uninstall 过滤）
- Modify: `apps/cli/src/commands/skills.rs:600,645`（提示文案 `gitflow skills` → `gf skills`）

**Interfaces:**
- Consumes: 无（纯内部逻辑改动）
- Produces: 4 处过滤条件 `starts_with("gitflow-")` → `starts_with("gf-")`；2 处提示文案同步

- [ ] **Step 1: 确认基线测试通过**

Run: `cargo test -p gf`
Expected: PASS（改前基线）

- [ ] **Step 2: 更新 4 处过滤条件 + 2 处提示文案**

在 `apps/cli/src/commands/skills.rs` 中执行 6 处精确替换：
- 行 296、371、593、630：`starts_with("gitflow-")` → `starts_with("gf-")`
- 行 600、645：`(未安装任何 gitflow skills)` → `(未安装任何 gf skills)`

**保留不改**：行 523 `"matcher": "gitflow"`、行 539/542/700 matcher 比较、行 785-808 `gitflow.co_contribution`/`joined_at`、行 1130/1135-1141 测试断言 `Some("gitflow")` —— 这些是 CLI 配置标识符，不属于 skill 名。

- [ ] **Step 3: 运行测试验证**

Run: `cargo test -p gf`
Expected: PASS（含 `test_merge_stop_hook_*` 断言 `matcher == "gitflow"` 仍通过，证明配置标识符未误改）

- [ ] **Step 4: 提交**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "refactor(skills): update skill prefix filter from gitflow- to gf- (#126)"
```

---

### Task 2: 更新 CLI 测试文件的 skill 名断言

**Files:**
- Modify: `apps/cli/tests/common/mod.rs:17`（`skills/gf-workflow/SKILL.md` → `skills/gf-workflow/SKILL.md`）
- Modify: `apps/cli/tests/workflow_modes_test.rs`（`gf-issue-create` 等断言）
- Modify: `apps/cli/tests/workflow_phase1_test.rs`（`gf-issue-create`/`gf-issue-review` 断言）
- Modify: `apps/cli/tests/workflow_phase2_test.rs`（`gf-quality` 断言）
- Modify: `apps/cli/tests/workflow_phase3_phase4_test.rs`（`gf-pipeline-analyzer`/`gf-issue-triage`/`gf-review` 断言）

**Interfaces:**
- Consumes: Task 1 的过滤逻辑（`gf-` 前缀）
- Produces: 测试断言与 SKILL.md 内容一致（断言 skill 名存在于 `skills/gf-*/SKILL.md`）

- [ ] **Step 1: 更新断言字符串**

对上述 5 个测试文件，将其中所有 skill 名断言从 `gitflow-<name>` 改为 `gf-<name>`（保持 `content.contains(...)` 断言结构不变）。示例：
- `common/mod.rs:17`：`format!("{manifest_dir}/../../skills/gf-workflow/SKILL.md")` → `.../skills/gf-workflow/SKILL.md`
- `workflow_modes_test.rs`：`md.contains("gf-issue-create")` → `md.contains("gf-issue-create")`（含失败消息字符串）

**注意**：`workflow_modes_test.rs` 中 `gitflow-cli` 出现的字符串（若有）保留不动。

- [ ] **Step 2: 运行测试验证（预期失败）**

Run: `cargo test -p gf --test workflow_modes_test workflow_phase1_test workflow_phase2_test workflow_phase3_phase4_test`
Expected: FAIL —— 因 SKILL.md 尚未重命名，`skills/gf-workflow/SKILL.md` 路径不存在（RED）

- [ ] **Step 3: 提交**

```bash
git add apps/cli/tests/common/mod.rs apps/cli/tests/workflow_modes_test.rs apps/cli/tests/workflow_phase1_test.rs apps/cli/tests/workflow_phase2_test.rs apps/cli/tests/workflow_phase3_phase4_test.rs
git commit -m "refactor(skills): update test assertions to gf-* skill names (#126)"
```

---

### Task 3: 更新 Makefile 与 install.sh 的 skill 过滤

**Files:**
- Modify: `Makefile:76`（`rm -rf ~/.claude/skills/gitflow-*` → `gf-*`）
- Modify: `scripts/install.sh:514`（`find ... -name "gitflow-*"` → `"gf-*"`）

**Interfaces:**
- Consumes: 无
- Produces: 安装/卸载逻辑匹配 `gf-*` 前缀目录

- [ ] **Step 1: 替换 2 处过滤**

- `Makefile:76`：`gitflow-*` → `gf-*`
- `scripts/install.sh:514-515`：`-name "gitflow-*"` → `-name "gf-*"`（同时更新 515 行注释）

- [ ] **Step 2: 验证**

Run: `grep -nF 'gitflow' Makefile scripts/install.sh`
Expected: 仅剩 `gitflow-cli` 仓库 URL 相关行（若有），无 skill 过滤残留

- [ ] **Step 3: 提交**

```bash
git add Makefile scripts/install.sh
git commit -m "refactor(skills): update Makefile and install.sh skill filters to gf-* (#126)"
```

---

### Task 4: 更新 hooks 脚本的 skill 引用

**Files:**
- Modify: `hooks/auto-report-bug.sh:11,123,124`
- Modify: `apps/cli/hooks/auto-report-bug.sh:11,123,124`

**Interfaces:**
- Consumes: 无
- Produces: hook 提示加载 `gf-autoreport-bug` skill

- [ ] **Step 1: 替换 3 处 skill 引用（每个文件）**

- 行 11：注释 `gf-autoreport-bug skill` → `gf-autoreport-bug skill`
- 行 123：`请加载 gf-autoreport-bug Skill` → `请加载 gf-autoreport-bug Skill`
- 行 124：`skills/gf-autoreport-bug/SKILL.md` → `skills/gf-autoreport-bug/SKILL.md`

**保留不改**：行 93 的 `gitflow-cli` GitHub URL。

- [ ] **Step 2: 验证**

Run: `grep -nF 'gitflow-' hooks/auto-report-bug.sh apps/cli/hooks/auto-report-bug.sh`
Expected: 仅剩 `gitflow-cli` URL（行 93），无 skill 名残留

- [ ] **Step 3: 提交**

```bash
git add hooks/auto-report-bug.sh apps/cli/hooks/auto-report-bug.sh
git commit -m "refactor(skills): update auto-report-bug hook to gf-autoreport-bug (#126)"
```

---

### Task 5: 重命名 `skills/` 目录与 `.claude/skills/` 本地副本

**Files:**
- Rename: `skills/gitflow-*` → `skills/gf-*`（26 个目录，`git mv`）
- Rename: `.claude/skills/gitflow-*` → `.claude/skills/gf-*`（26 个目录，普通 `mv`，不进 git）

**Interfaces:**
- Consumes: Task 1-4 的过滤逻辑已指向 `gf-*`
- Produces: `skills/gf-*/SKILL.md` 路径；build.rs manifest 重新扫描生成 `gf-*` 条目

- [ ] **Step 1: 批量重命名 `skills/`（git mv）**

```bash
cd /Volumes/SSD/workspace/github.com/byx-darwin/gitflow-cli
for d in skills/gitflow-*; do
  new="${d/gitflow-/gf-}"
  git mv "$d" "$new"
done
```

- [ ] **Step 2: 同步重命名 `.claude/skills/`（普通 mv）**

```bash
for d in .claude/skills/gitflow-*; do
  new="${d/gitflow-/gf-}"
  mv "$d" "$new"
done
```

- [ ] **Step 3: 验证目录重命名**

Run: `ls skills/ | grep -c '^gitflow'`；`ls skills/ | grep -c '^gf-'`
Expected: `0` 与 `26`

Run: `git status --short | head -30`
Expected: 52 个 rename 记录（26 `.claude/` 不计入，因未被跟踪）

- [ ] **Step 4: 提交**

```bash
git add skills/
git commit -m "refactor(skills): rename skill directories from gitflow-* to gf-* (#126)"
```

---

### Task 6: 重命名 docs 文档文件

**Files:**
- Rename: `docs/references/gitflow-*.md` → `docs/references/gf-*.md`（9 个）
- Rename: `docs/research/skill-analysis-gitflow-*.md` → `docs/research/skill-analysis-gf-*.md`（26 个）
- Rename: `docs/superpowers/tests/skills/gitflow-*-test.md` → `docs/superpowers/tests/skills/gf-*-test.md`（24 个）
- Rename: `docs/gf-workflow-guide.md` → `docs/gf-workflow-guide.md`
- Rename: `docs/superpowers/plans/2026-07-06-gf-workflow-refactor.md` → `2026-07-06-gf-workflow-refactor.md`
- Rename: `docs/superpowers/plans/2026-07-09-gf-workflow-auto-trigger.md` → `2026-07-09-gf-workflow-auto-trigger.md`
- Rename: `docs/superpowers/specs/2026-07-09-gf-workflow-auto-trigger-design.md` → `2026-07-09-gf-workflow-auto-trigger-design.md`

**Interfaces:**
- Consumes: 无
- Produces: 文档文件名与 skill 名一致；`docs/index.md:13` 对 guide 的链接需同步更新（由 Task 7 内容替换处理 `./gf-workflow-guide.md` → `./gf-workflow-guide.md`）

- [ ] **Step 1: 批量重命名全部文件（git mv）**

```bash
cd /Volumes/SSD/workspace/github.com/byx-darwin/gitflow-cli
for f in docs/references/gitflow-*.md; do
  git mv "$f" "${f/gitflow-/gf-}"
done
for f in docs/research/skill-analysis-gitflow-*.md; do
  git mv "$f" "${f/gitflow-/gf-}"
done
for f in docs/superpowers/tests/skills/gitflow-*-test.md; do
  git mv "$f" "${f/gitflow-/gf-}"
done
git mv docs/gf-workflow-guide.md docs/gf-workflow-guide.md
git mv docs/superpowers/plans/2026-07-06-gf-workflow-refactor.md docs/superpowers/plans/2026-07-06-gf-workflow-refactor.md
git mv docs/superpowers/plans/2026-07-09-gf-workflow-auto-trigger.md docs/superpowers/plans/2026-07-09-gf-workflow-auto-trigger.md
git mv docs/superpowers/specs/2026-07-09-gf-workflow-auto-trigger-design.md docs/superpowers/specs/2026-07-09-gf-workflow-auto-trigger-design.md
```

- [ ] **Step 2: 验证**

Run: `find . -type f -not -path './.cache/*' -not -path './target/*' -not -path './.git/*' -not -path './.claude/skills/*' \( -name '*.md' -o -name '*.sh' \) | grep -E 'gitflow-(auth|autoreport-bug|commit|issue|issue-create|issue-review|issue-triage|label-milestone|label-stats|pipeline-analyzer|pr|pr-apply-feedback|pr-create|pr-inline-review|pr-review|precommit|quality|regression|release|release-helper|repo|repo-onboarding|review|security-check|weekly-report|workflow)'`
Expected: 无输出（`.claude/skills/` 为本地副本，已在 Task 5 处理）

- [ ] **Step 3: 提交**

```bash
git add docs/references/ docs/research/ docs/superpowers/tests/skills/ docs/gf-workflow-guide.md docs/superpowers/plans/ docs/superpowers/specs/
git commit -m "refactor(skills): rename skill reference docs to gf-* (#126)"
```

---

### Task 7: 全局内容替换（26 个 skill 名，~2,176 处）

**Files:**
- Modify: 全部含 `gitflow-<skill名>` 引用的 `.md/.rs/.sh/.toml/.yaml/.yml/.json` 文件（164 个，排除 `.cache/`、`target/`、`.git/`）

**Interfaces:**
- Consumes: Task 5-6 完成（目录/文件名已为 `gf-*`）
- Produces: 所有文档/配置中 skill 引用为 `gf-*`；CHANGELOG 中 `gitflow-cli` 保留

- [ ] **Step 1: 生成白名单替换脚本（按长度降序）**

```bash
cd /Volumes/SSD/workspace/github.com/byx-darwin/gitflow-cli
SKILLS="pipeline-analyzer pr-apply-feedback pr-inline-review label-milestone autoreport-bug release-helper repo-onboarding security-check weekly-report issue-create issue-review issue-triage label-stats regression pr-review pr-create precommit workflow release quality commit review issue repo auth pr"
SED_EXPR=""
for s in $SKILLS; do
  SED_EXPR="${SED_EXPR}s/gitflow-${s}/gf-${s}/g; "
done
printf '%s\n' "$SED_EXPR" > /tmp/skill_rename.sed
```

- [ ] **Step 2: 对目标文件应用替换**

```bash
cd /Volumes/SSD/workspace/github.com/byx-darwin/gitflow-cli
grep -rlE 'gitflow-(auth|autoreport-bug|commit|issue|issue-create|issue-review|issue-triage|label-milestone|label-stats|pipeline-analyzer|pr|pr-apply-feedback|pr-create|pr-inline-review|pr-review|precommit|quality|regression|release|release-helper|repo|repo-onboarding|review|security-check|weekly-report|workflow)' \
  --include='*.md' --include='*.rs' --include='*.sh' --include='*.toml' --include='*.yaml' --include='*.yml' --include='*.json' . \
  | grep -v '^./.cache/' | grep -v '^./target/' | grep -v '^./.git/' \
  | xargs -I{} sed -i -f /tmp/skill_rename.sed '{}'
```

- [ ] **Step 3: 验证无残留 skill 名**

Run: `grep -rnE 'gitflow-(auth|autoreport-bug|commit|issue|issue-create|issue-review|issue-triage|label-milestone|label-stats|pipeline-analyzer|pr|pr-apply-feedback|pr-create|pr-inline-review|pr-review|precommit|quality|regression|release|release-helper|repo|repo-onboarding|review|security-check|weekly-report|workflow)' --include='*.md' --include='*.rs' --include='*.sh' --include='*.toml' --include='*.yaml' --include='*.yml' --include='*.json' . | grep -v '^./.cache/' | grep -v '^./target/'`
Expected: 无输出

- [ ] **Step 4: 验证 `gitflow-cli` URL 未被误伤**

Run: `grep -rnF 'gitflow-cli' README.md apps/cli/README.md crates/github/src/pipeline.rs`
Expected: 仍存在（仓库 URL 保留）

- [ ] **Step 5: 提交**

```bash
git add -A
git commit -m "refactor(skills): replace gitflow-* skill references with gf-* across repo (#126)"
```

---

### Task 8: 最终验证与回归

**Files:**
- 验证产物：无文件修改

**Interfaces:**
- Consumes: Task 1-7 全部完成
- Produces: 退出标准证据

- [ ] **Step 1: 全量构建 + 测试**

Run: `make build`
Expected: PASS（build.rs 重新扫描 `skills/` 生成 `gf-*` manifest）

Run: `make test`
Expected: 全部 PASS（含 Task 2 更新的断言，断言 `skills/gf-workflow/SKILL.md` 现存在）

- [ ] **Step 2: `gf skills list` 验证**

Run: `cargo run -p gf -- skills list`
Expected: 列出 26 个 `gf-*` skill（含 `gf-workflow`）

- [ ] **Step 3: 残留检查（排除历史文档中的 `gitflow-cli`）**

Run: `grep -rn 'gitflow-' --include='*.md' --include='*.rs' --include='*.sh' --include='*.toml' --include='*.yaml' --include='*.yml' --include='*.json' . | grep -v '^./.cache/' | grep -v '^./target/' | grep -v 'gitflow-cli'`
Expected: 无输出（历史 CHANGELOG 等若含 skill 名，需人工确认是否为"历史记录除外"范畴）

- [ ] **Step 4: 文档同步验证**

Run: `make check-agent-sync`
Expected: PASS（CLAUDE.md 与 skills 一致）

- [ ] **Step 5: 提交验证产物（如有修复）**

```bash
git status --short
# 如有文件待提交，git add -A && git commit -m "refactor(skills): regression fixes (#126)"
```

---

## Self-Review Notes

**Spec 覆盖检查：**
- ✅ 26 skill 目录（Task 5）· 9+ docs/references（Task 6）· research/tests 文档（Task 6）· 全局引用（Task 7）· Rust 源码（Task 1-2）· hooks/scripts/Makefile（Task 3-4）· 退出标准（Task 8）
- ⚠️ 设计文档中的 `gitflow-cli` 保留规则已落实为 Global Constraints + Task 7 Step 4 验证
- ⚠️ `.claude/skills/` 不跟踪的事实已在 Task 5 Step 2 处理（普通 mv）

**类型一致性：** 所有替换目标统一为 `gf-<skill名>`，与 skill 目录命名一致；build.rs 通过目录扫描自动生成 manifest，无需改代码。
