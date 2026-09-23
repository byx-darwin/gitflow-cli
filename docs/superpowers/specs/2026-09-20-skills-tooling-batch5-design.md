# 批次5：Skill/Tooling 机械性修复设计（#353 #363 #355 #338）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-20-002

## 背景

五个 skill/tooling 基础设施缺陷。逐个核实现状后发现范围比原始 issue 描述更小：

## 现状核实

| Issue | 状态 | 说明 |
|---|---|---|
| #353 | **真 bug，需修** | `skills/gf-workflow/references.md`（及 `SKILL.md` 内嵌同款命令）的软链深度公式对 `.claude` 多退一级，现场复现确认悬空 |
| #363 | **真 bug，需修** | `make check-skills-drift` 只比目录存在性，不比内容 |
| #355 | **真 bug，需修** | `make check-walkthrough-skill` 的 awk `getline` 吞行漏检 |
| #350 | **范围超出本批次，移出** | 命令语法子问题（"问题1"）已修；但"问题2"（散文与命令口径矛盾：行内代码算不算入限额）与 AC3（26/30 skill 用正确公式一跑就超限，需要豁免/压缩/调整限额的政策决策）仍未解决，且比 issue 原文描述更严重。这是需要用户拍板的政策问题，不是机械修复，留到下一轮单独处理（详见 #350 评论区的更正说明） |
| #338 | **一半已修** | 链接断链部分已由 `8d5c42a fix(skills): correct ../references paths to ../../docs/references` 修复并验证（`docs/references/pr-review-checklist.md` 真实存在，`skills/gf-pr-review/SKILL.md` 三处引用已正确指向）；`skills/gf-pr/SKILL.md` frontmatter 非法键 `Full params:` **仍未修** |

## 修复方案

### #353：软链深度公式拆分

`skills/gf-workflow/references.md`（Symlink shared directories 代码块）与
`skills/gf-workflow/SKILL.md`（Phase 3 Step 1 表格里内嵌的同款单行命令）都要改。

根因：`.cache/workflows` 软链文件本身位于 `$WORKTREE_PATH/.cache/`（比
`$WORKTREE_PATH` 深一级），需要 `segs + 1` 个 `../`；`.claude` 软链文件直接位于
`$WORKTREE_PATH/`，只需要 `segs` 个 `../`。两者不能共用一个 `ups`。

```bash
segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
ups_cache=$((segs + 1))   # .cache/workflows 软链文件在 .cache/ 下，多一级
ups_claude=$segs          # .claude 软链文件直接在 $WORKTREE_PATH 下
rel_cache=$(printf '../%.0s' $(seq 1 "$ups_cache"))
rel_claude=$(printf '../%.0s' $(seq 1 "$ups_claude"))
mkdir -p "$WORKTREE_PATH/.cache"
ln -s "${rel_cache}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
ln -s "${rel_claude}.claude" "$WORKTREE_PATH/.claude"
```

自检块同步补上 `.claude` 的验证（此前只验证了 `.cache/workflows`）：

```bash
test -d "$WORKTREE_PATH/.cache/workflows" || {
  echo "ABORT: .cache/workflows symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups_cache=$ups_cache"
  exit 1
}
test -d "$WORKTREE_PATH/.claude" || {
  echo "ABORT: .claude symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups_claude=$ups_claude"
  exit 1
}
```

"Why the Symlink Depth Is Computed" 说明段落与表格同步更新为两个公式并存的表述。

`SKILL.md` 里的内嵌单行命令按同样逻辑改写（保持单行风格，与现有格式一致）。

### #363：drift 检查补内容比对

`Makefile` 的 `check-skills-drift` target，在"已安装且目录存在"分支追加一行
`diff -rq` 内容比对：

```make
	for p in skills/*/; do \
		n=`basename "$$p"`; \
		if [ ! -d "$$D/$$n" ]; then echo "✗ 缺失  $$n —— 仓库中存在，尚未安装"; DRIFT=1; \
		elif ! diff -rq "$$p" "$$D/$$n" >/dev/null 2>&1; then echo "✗ 内容不一致  $$n —— 已安装但内容与 skills/ 不同"; DRIFT=1; fi; \
	done; \
```

### #355：`getline` 改成数组索引

`Makefile` 的 `FCNT=` 那一行 awk 脚本，改为先把整份文件读进数组，再按下标往后看
3 行，不再用会消耗主输入流的 `getline`：

```awk
{lines[NR]=$0} END{
  total=NR; c=0;
  for(i=1;i<=total;i++){
    if(lines[i] ~ /^- \[Measured\]/){
      f=0;
      for(j=i+1;j<=i+3 && j<=total;j++){ if(lines[j] ~ /^ *```/) f=1 }
      if(!f) c++
    }
  }
  print c+0
}
```

### #338：frontmatter 非法键搬家

`skills/gf-pr/SKILL.md`：删掉 frontmatter 里的 `Full params: docs/references/gf-pr-params.md` 一行，在已有的 `## See Also` 段落里追加一条：

```markdown
- Full params reference: `docs/references/gf-pr-params.md`
```

## 测试策略

Makefile target 与 awk/shell 脚本，无 Rust 单测。验证方式：
- `#363`：实测触发一次真实漂移（改动 `~/.claude/skills` 里某个文件内容）验证能报出"内容不一致"，恢复后验证报"一致"
- `#355`：用一份 `[Measured]` 条目紧邻排布的走查包片段实测 `FCNT`，验证不再漏检
- `#353`：本次修复后创建的下一个 gf-workflow worktree 顺带验证 `.claude` 软链能正确解析（本会话后续批次即是活验证）
- `#338`：`make check-skills-drift`（若已跑过）或人工核对 frontmatter 只剩合法键

## 范围外

- `#350`：范围超出本批次（政策决策，非机械修复），已重新打开留待下一轮单独处理
- `#338` 链接部分：已确认无需改动
