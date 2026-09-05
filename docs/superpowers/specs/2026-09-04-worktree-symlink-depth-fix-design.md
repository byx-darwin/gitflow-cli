# Worktree Preflight 符号链接相对深度修复

- Issue: #322
- Workflow: wf-2026-09-04-005
- Classification: Bounded（既有 Worktree Preflight 流程内的定点修复）

## Context

`skills/gf-workflow/SKILL.md` Phase 3 Step 1 与 `references.md` 的 Worktree
Preflight 示例，创建 `.cache/workflows` / `.claude` 共享目录符号链接时，硬编码
相对深度 `../../`：

```bash
mkdir -p <worktree-path>/.cache
ln -s ../../.cache/workflows <worktree-path>/.cache/workflows
ln -s ../../.claude <worktree-path>/.claude
```

Issue #322 报告：当分支名（`feat/<issue-number>-<short-description>`）含 `/`
时，`worktree_path` 实际是两级（`.worktree/feat/89-xxx/`），固定的 `../../`
只跳 2 层，落在 `.worktree/feat/` 而非仓库根，导致符号链接指向不存在路径。
下游项目 `iproost/proxy/api-src` Issue #89 已实测踩坑。

## 实测发现：既有"单段"场景本身也是错的

在设计前用真实 `mkdir` + `ln -s` 复现符号链接解析（而非从文档推断）：

```bash
mkdir -p repo-root/.cache/workflows
mkdir -p repo-root/.worktree/foo/.cache
cd repo-root
ln -s ../../.cache/workflows .worktree/foo/.cache/workflows
readlink -f .worktree/foo/.cache/workflows
# => /repo-root/.worktree/.cache   （不是仓库根！）
```

结论：符号链接文件本身位于 `<worktree_path>/.cache/`，比 `worktree_path`
本身多一级。硬编码 `../../`（2 层）对单段 `worktree_path`（如
`.worktree/foo`）也少算一层，只到 `.worktree/` 就停了，从未真正到达仓库根。
Issue #322 描述的"含 `/` 场景"只是让这个既有 bug 更容易被踩中，根因是深度公式
本身就错，而不是"只在含斜杠时才错"。

## 正确公式

```
ups = worktree_path 按 "/" 切分后的段数 + 1
```

（+1 是因为符号链接实际所在目录 `<worktree_path>/.cache/` 比
`worktree_path` 本身深一级）

验证：

| worktree_path | 段数 | 应有 ups | 实测结果 |
|---|---|---|---|
| `.worktree/foo` | 2 | 3 | `../../../.cache/workflows` → 仓库根 ✅ |
| `.worktree/feat/89-desc` | 3 | 4 | `../../../../.cache/workflows` → 仓库根 ✅ |

## 方案

采纳 Issue 建议方向 1（动态计算深度）+ 方向 3（存在性自检），不采纳方向 2
（绝对路径）——`references.md` 已有专门章节说明为何必须用相对路径 +
`info/exclude`（见 "Why These Symlinks Must Never Reach the Main Branch"），
绝对路径会破坏这个既定设计权衡（换 clone 位置即失效，且与仓库自身的可移植性
目标冲突）。

### 1. 动态深度计算

```bash
segs=$(awk -F/ '{print NF}' <<< "$worktree_path")
ups=$((segs + 1))
rel=$(printf '../%.0s' $(seq 1 "$ups"))
mkdir -p "$worktree_path/.cache"
ln -s "${rel}.cache/workflows" "$worktree_path/.cache/workflows"
ln -s "${rel}.claude" "$worktree_path/.claude"
```

### 2. 符号链接存在性自检

创建后立即验证符号链接真实可达（而不是仅仅"文件存在"——断链的符号链接本身
也会通过 `test -e`/`ls -l` 显示存在，需要验证其**指向的目标目录**）：

```bash
if [ ! -d "$worktree_path/.cache/workflows" ]; then
  echo "ABORT: 符号链接深度算错 — worktree_path=$worktree_path 段数=$segs ups=$ups" \
       "预期指向仓库根 .cache/workflows，实际未解析到任何目录。" \
       "请检查 worktree_path 是否符合 .worktree/<branch-name> 约定。" >&2
  exit 1
fi
```

自检失败时给出的诊断信息明确指向"符号链接深度算错"，避免被后续步骤误判为
"合约丢失"（这正是 Issue #322 描述的下游排障困境）。

### 3. 文档改动范围

- `SKILL.md` Phase 3 Step 1：替换硬编码 `../../` 为动态计算 + 自检片段
- `references.md` Worktree Preflight 示例代码块：
  - 把示例分支名从单段 `feat-146-worktree-path`（掩盖了真实的含 `/` 约定）
    换成真实形态 `feat/146-worktree-path`，避免示例本身继续误导
  - 补充上面的实测数据脚注，说明为什么公式是 `段数 + 1` 而不是固定值
  - 新增一条含 `/` 分支名场景的验证说明（对应验收标准第 2 条）

## 验收标准对照

- [x] 符号链接相对深度按 `worktree_path` 实际段数动态计算，不再硬编码 `../../`
- [x] 补充分支名含 `/` 场景的验证用例/说明（含单段场景本身也曾是错的这一发现）
- [x] 符号链接创建后增加存在性自检，失败时给出针对性诊断信息
