# Skills/Tooling Batch5 Mechanical Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 4 个独立的 skill/tooling 缺陷（#353 #363 #355 #338），均为纯 Makefile/awk/markdown 改动，无 Rust 编译代码。

**Architecture:** 4 个任务分别落在 4 组不相关文件（`skills/gf-workflow/*` 两个文件、`Makefile` 两处独立 target、`skills/gf-pr/SKILL.md` 一个文件），互不重叠，可合并成一个 PR 交付。

**Tech Stack:** Bash / GNU Make / AWK / Markdown

**Spec:** `docs/superpowers/specs/2026-09-20-skills-tooling-batch5-design.md`

## Global Constraints

- `#350` 已移出本批次范围（政策决策，不在此计划内处理）
- 不改动 `.pre-commit-config.yaml`（本批次未涉及，但提醒：该文件受 CLAUDE.md 保护）
- 每个任务改完后跑对应的人工验证命令，不假设"看起来对"

---

### Task 1: gf-workflow 软链深度公式拆分（#353）

**Files:**
- Modify: `skills/gf-workflow/references.md`（Symlink shared directories 代码块 + Why 说明段落）
- Modify: `skills/gf-workflow/SKILL.md`（Phase 3 Step 1 表格内嵌的同款单行命令）

**Interfaces:**
- 无跨任务接口——独立文件改动

- [ ] **Step 1: 改 `references.md` 的软链创建代码块**

把这段：

```bash
segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
ups=$((segs + 1))
rel=$(printf '../%.0s' $(seq 1 "$ups"))
mkdir -p "$WORKTREE_PATH/.cache"
ln -s "${rel}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
ln -s "${rel}.claude" "$WORKTREE_PATH/.claude"
```

替换为：

```bash
segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
ups_cache=$((segs + 1))   # .cache/workflows 软链文件位于 .cache/ 下，多一级
ups_claude=$segs          # .claude 软链文件直接位于 $WORKTREE_PATH 下
rel_cache=$(printf '../%.0s' $(seq 1 "$ups_cache"))
rel_claude=$(printf '../%.0s' $(seq 1 "$ups_claude"))
mkdir -p "$WORKTREE_PATH/.cache"
ln -s "${rel_cache}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
ln -s "${rel_claude}.claude" "$WORKTREE_PATH/.claude"
```

- [ ] **Step 2: 改 `references.md` 的自检代码块**

紧随其后的自检块，把：

```bash
test -d "$WORKTREE_PATH/.cache/workflows" || {
  echo "ABORT: symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups=$ups"
  echo "Expected to resolve to repo-root .cache/workflows but did not."
  exit 1
}
```

替换为：

```bash
test -d "$WORKTREE_PATH/.cache/workflows" || {
  echo "ABORT: .cache/workflows symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups_cache=$ups_cache"
  echo "Expected to resolve to repo-root .cache/workflows but did not."
  exit 1
}
test -d "$WORKTREE_PATH/.claude" || {
  echo "ABORT: .claude symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups_claude=$ups_claude"
  echo "Expected to resolve to repo-root .claude but did not."
  exit 1
}
```

- [ ] **Step 3: 改 `references.md` 的"Why the Symlink Depth Is Computed"说明段落**

把这句：

```markdown
A relative symlink resolves starting from the directory that *contains* the
symlink file, not from `worktree_path` itself. The symlinks above live at
`$WORKTREE_PATH/.cache/workflows` and `$WORKTREE_PATH/.claude`, so their
containing directory (`$WORKTREE_PATH/.cache/`) is **one segment deeper**
than `$WORKTREE_PATH`. The number of `../` needed to reach the repo root is
therefore:

```
ups = (number of "/"-separated segments in worktree_path) + 1
```
```

替换为：

```markdown
A relative symlink resolves starting from the directory that *contains* the
symlink file, not from `worktree_path` itself. `$WORKTREE_PATH/.cache/workflows`
lives inside `$WORKTREE_PATH/.cache/`, **one segment deeper** than
`$WORKTREE_PATH` — but `$WORKTREE_PATH/.claude` lives directly at
`$WORKTREE_PATH`, with no extra segment. The two symlinks therefore need two
different `../` counts:

```
ups_cache  = (number of "/"-separated segments in worktree_path) + 1
ups_claude = (number of "/"-separated segments in worktree_path)
```

An earlier version of this formula used `ups_cache`'s value for both
symlinks, which left `.claude` dangling (it resolved one level above the
repo root). Verified empirically with real `mkdir` + `ln -s` — see Issue #353.
```

- [ ] **Step 4: 改 `references.md` 的对照表格**

把：

```markdown
| `worktree_path` | segments | `ups` | Hardcoded `../../` resolves to |
|---|---|---|---|
| `.worktree/foo` (single-segment — the case the old hardcoded value was written for) | 2 | 3 | `.worktree/` — **not the repo root** |
| `.worktree/feat/89-desc` (branch name contains `/`, per the `feat/<issue-number>-<short-description>` convention) | 3 | 4 | `.worktree/feat/` — **not the repo root** |
```

替换为：

```markdown
| `worktree_path` | segments | `ups_cache` | `ups_claude` |
|---|---|---|---|
| `.worktree/foo` (single-segment) | 2 | 3 | 2 |
| `.worktree/feat/89-desc` (branch name contains `/`, per the `feat/<issue-number>-<short-description>` convention) | 3 | 4 | 3 |
```

（保留该段后面"The old hardcoded `../../`……"那一整段说明文字不变，只替换表格本身；那段文字讲的是历史上硬编码 `../../` 的问题，与本次 `ups_cache`/`ups_claude` 拆分无关，不用改。）

- [ ] **Step 5: 改 `SKILL.md` 内嵌的同款单行命令**

`skills/gf-workflow/SKILL.md` Phase 3 Step 1 表格行里，找到这段内嵌命令（是一整行很长的表格单元格内容的一部分）：

```
segs=$(awk -F/ '{print NF}' <<< "<worktree-path>"); ups=$((segs + 1)); rel=$(printf '../%.0s' $(seq 1 "$ups")); mkdir -p <worktree-path>/.cache && ln -s "${rel}.cache/workflows" <worktree-path>/.cache/workflows && ln -s "${rel}.claude" <worktree-path>/.claude; test -d <worktree-path>/.cache/workflows || { echo "ABORT: symlink depth miscalculated — worktree_path=<worktree-path> segs=$segs ups=$ups, expected to resolve to repo-root .cache/workflows but did not. Check worktree_path follows the .worktree/<branch-name> convention."; exit 1; }
```

替换为：

```
segs=$(awk -F/ '{print NF}' <<< "<worktree-path>"); ups_cache=$((segs + 1)); ups_claude=$segs; rel_cache=$(printf '../%.0s' $(seq 1 "$ups_cache")); rel_claude=$(printf '../%.0s' $(seq 1 "$ups_claude")); mkdir -p <worktree-path>/.cache && ln -s "${rel_cache}.cache/workflows" <worktree-path>/.cache/workflows && ln -s "${rel_claude}.claude" <worktree-path>/.claude; test -d <worktree-path>/.cache/workflows || { echo "ABORT: .cache/workflows symlink depth miscalculated — worktree_path=<worktree-path> segs=$segs ups_cache=$ups_cache, expected to resolve to repo-root .cache/workflows but did not."; exit 1; }; test -d <worktree-path>/.claude || { echo "ABORT: .claude symlink depth miscalculated — worktree_path=<worktree-path> segs=$segs ups_claude=$ups_claude, expected to resolve to repo-root .claude but did not."; exit 1; }
```

（该命令后面紧跟的 `**Immediately exclude them**……` 一段不变，只替换这一段命令本身。用精确字符串匹配定位，因为这行命令在文件里只出现一次。）

- [ ] **Step 6: 人工验证**

Run:
```bash
mkdir -p /tmp/symlink-verify/.worktree/feat/xyz-test /tmp/symlink-verify/.claude /tmp/symlink-verify/.cache/workflows
cd /tmp/symlink-verify
WORKTREE_PATH=".worktree/feat/xyz-test"
segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
ups_cache=$((segs + 1))
ups_claude=$segs
rel_cache=$(printf '../%.0s' $(seq 1 "$ups_cache"))
rel_claude=$(printf '../%.0s' $(seq 1 "$ups_claude"))
mkdir -p "$WORKTREE_PATH/.cache"
ln -s "${rel_cache}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
ln -s "${rel_claude}.claude" "$WORKTREE_PATH/.claude"
test -d "$WORKTREE_PATH/.cache/workflows" && echo "cache OK"
test -d "$WORKTREE_PATH/.claude" && echo "claude OK"
cd / && rm -rf /tmp/symlink-verify
```
Expected: 两行都打印 `OK`

- [ ] **Step 7: 提交**

```bash
git add skills/gf-workflow/references.md skills/gf-workflow/SKILL.md
git commit -m "fix(gf-workflow): split worktree symlink depth formula for .cache/workflows vs .claude (#353)"
```

---

### Task 2: check-skills-drift 补内容比对（#363）

**Files:**
- Modify: `Makefile:184-203`（`check-skills-drift` target）

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 改 Makefile target**

把：

```make
	for p in skills/*/; do \
		n=`basename "$$p"`; \
		if [ ! -d "$$D/$$n" ]; then echo "✗ 缺失  $$n —— 仓库中存在，尚未安装"; DRIFT=1; fi; \
	done; \
```

替换为：

```make
	for p in skills/*/; do \
		n=`basename "$$p"`; \
		if [ ! -d "$$D/$$n" ]; then echo "✗ 缺失  $$n —— 仓库中存在，尚未安装"; DRIFT=1; \
		elif ! diff -rq "$$p" "$$D/$$n" >/dev/null 2>&1; then echo "✗ 内容不一致  $$n —— 已安装但内容与 skills/ 不同"; DRIFT=1; fi; \
	done; \
```

- [ ] **Step 2: 人工验证——先制造一处真实漂移**

Run:
```bash
echo "# drift probe" >> ~/.claude/skills/gf-pr/SKILL.md
make check-skills-drift 2>&1 | grep "gf-pr"
```
Expected: 输出包含 `✗ 内容不一致  gf-pr`

- [ ] **Step 3: 恢复现场**

Run:
```bash
git -C ~/.claude/skills/../.. diff --stat 2>/dev/null || true
sed -i '' '/# drift probe/d' ~/.claude/skills/gf-pr/SKILL.md
make check-skills-drift 2>&1 | tail -3
```
Expected: 探针行被删除后，`gf-pr` 不再出现在漂移报告里（若其余 skill 本来就有真实漂移，那些行不受影响，属于真实信号不是本步骤引入的噪音）

- [ ] **Step 4: 提交**

```bash
git add Makefile
git commit -m "fix(makefile): check-skills-drift now diffs content, not just directory presence (#363)"
```

---

### Task 3: check-walkthrough-skill 的 `getline` 改数组下标（#355）

**Files:**
- Modify: `Makefile:464`（`check-walkthrough-skill` target 的 `FCNT=` 行）

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 改 awk 脚本**

把这一行（注意原文里反引号用 `\x60` 转义，因为整行被 Makefile 的反引号命令替换语法包裹）：

```make
		FCNT=`awk '/^- \[Measured\]/{f=0; for(i=1;i<=3;i++){if((getline line)>0){if(line ~ /^ *\x60\x60\x60/) f=1}}; if(!f) c++} END{print c+0}' "$$REP"`; \
```

替换为：

```make
		FCNT=`awk '{lines[NR]=$$0} END{total=NR; c=0; for(i=1;i<=total;i++){if(lines[i] ~ /^- \[Measured\]/){f=0; for(j=i+1;j<=i+3 && j<=total;j++){if(lines[j] ~ /^ *\x60\x60\x60/) f=1}; if(!f) c++}} print c+0}' "$$REP"`; \
```

（Makefile 里 `$0`/`$NR` 这类 awk 内建变量在反引号命令替换里要写成 `$$0`/`$$NR`（`$$` 转义 Make 自己的变量展开），与原脚本 `$$REP` 的转义方式保持一致；`i`/`j`/`c`/`total`/`f`/`lines` 是普通 awk 变量，不需要转义。）

- [ ] **Step 2: 人工验证——构造相邻 `[Measured]` 条目验证不再漏检**

Run:
```bash
cat > /tmp/walkthrough-probe.md << 'PROBE'
- [Measured] 第一条，紧跟着第二条，中间没有代码围栏
- [Measured] 第二条，同样没有代码围栏
```
PROBE
```
```bash
awk '{lines[NR]=$0} END{total=NR; c=0; for(i=1;i<=total;i++){if(lines[i] ~ /^- \[Measured\]/){f=0; for(j=i+1;j<=i+3 && j<=total;j++){if(lines[j] ~ /^ *```/) f=1}; if(!f) c++}} print c+0}' /tmp/walkthrough-probe.md
rm -f /tmp/walkthrough-probe.md
```
Expected: 输出 `2`（两条都应被判定为缺输出块，旧的 `getline` 版本在这种紧邻排布下只会数出 `1`，因为第一条的 `getline` 循环会把第二条那一行吞掉）

- [ ] **Step 3: 回归——确认既有走查包仍判定正确**

Run:
```bash
make check-walkthrough-skill 2>&1 | grep "#3\|Measured"
```
Expected: 与改动前观察到的结果一致（`docs/walkthrough-feat-329-gf-walkthrough-2026-09-16.md` 现有条目之间都有代码围栏分隔，`FCNT` 应仍为 `0`）

- [ ] **Step 4: 提交**

```bash
git add Makefile
git commit -m "fix(makefile): check-walkthrough-skill no longer drops adjacent [Measured] entries via getline (#355)"
```

---

### Task 4: gf-pr frontmatter 非法键搬家（#338 剩余部分）

**Files:**
- Modify: `skills/gf-pr/SKILL.md:1-9`（frontmatter）
- Modify: `skills/gf-pr/SKILL.md:161-166`（`## See Also` 段落）

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 从 frontmatter 删除非法键**

把：

```yaml
---
name: gf-pr
description: >
  Use when the user manages PRs via gf: create/list/view/close/merge/
  checkout/comment/sync/ready/wip/reopen or toggles draft/ready state.
  当用户通过 gf 创建、查看、合并、关闭、评论、检出、同步、
  标记PR时使用。
Full params: docs/references/gf-pr-params.md
---
```

替换为：

```yaml
---
name: gf-pr
description: >
  Use when the user manages PRs via gf: create/list/view/close/merge/
  checkout/comment/sync/ready/wip/reopen or toggles draft/ready state.
  当用户通过 gf 创建、查看、合并、关闭、评论、检出、同步、
  标记PR时使用。
---
```

- [ ] **Step 2: 在 `## See Also` 补一条**

把：

```markdown
## See Also

- `/gf-pr-create` — PR creation workflow
- `/gf-pr-review` — full review
- `/gf-pr-inline-review` — line-level review
- `/gf-pr-apply-feedback` — post-review code changes
```

替换为：

```markdown
## See Also

- `/gf-pr-create` — PR creation workflow
- `/gf-pr-review` — full review
- `/gf-pr-inline-review` — line-level review
- `/gf-pr-apply-feedback` — post-review code changes
- Full params reference: `docs/references/gf-pr-params.md`
```

- [ ] **Step 3: 人工验证**

Run:
```bash
sed -n '1,9p' skills/gf-pr/SKILL.md
```
Expected: frontmatter 只剩 `name`、`description` 两个键，无 `Full params:`

Run:
```bash
grep -n "gf-pr-params.md" skills/gf-pr/SKILL.md
```
Expected: 唯一命中在 `## See Also` 段落里

- [ ] **Step 4: 提交**

```bash
git add skills/gf-pr/SKILL.md
git commit -m "fix(skills): move gf-pr's illegal frontmatter key into See Also (#338)"
```

---

## Final Verification

- [ ] **Step 1: pre-commit 全量**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed`（本次未改 Rust 代码，`cargo fmt`/`cargo check` 应为 `Skipped`）

- [ ] **Step 2: `make lint` 中涉及的 skill 相关检查**

Run: `make check-skills-drift`
Expected: 命令能正常运行（结果取决于当前 `~/.claude/skills` 与仓库是否一致，不强制要求 0 漂移——本步骤只验证命令本身不报错）
