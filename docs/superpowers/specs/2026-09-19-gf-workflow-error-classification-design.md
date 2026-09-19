# gf-workflow 错误分类恢复表与重试上限 — 设计

**Issue:** #336
**Date:** 2026-09-19

## Context

`skills/gf-workflow/SKILL.md` 的「Error Handling & Common Mistakes」表（第 459-468 行）只覆盖
**编排器层面**的错误（合约缺失、gate 校验失败、sub-skill 未返回等），没有覆盖 **Phase 3 执行
过程中**的失败（build / test / lint / merge_conflict / ci / auth / rate_limit / network /
issue_unclear / unknown）——也没有任何重试上限，执行阶段遇到失败时缺乏收敛保证。

参考 `smallnest/goal-workflow` 的 `skills/loop-it/SKILL.md:419-438`：一张 10 类错误的恢复表，
每类配独立恢复策略与最大重试次数。移植时不能照搬两类：

- **`merge_conflict`**：本仓库 Phase 3 Step 3 已有专门处理——`git merge --abort`，`branch`/
  worktree 保持不变，交回用户手动解决，不做自动重试或静默切换到 PR 路径。
- **`ci`**：本仓库 Phase 3 Step 5 使用「排队合并」（`gf pr merge --auto`），排队绑定的是已通过
  检查的那个 SHA；一旦排队，往同一分支追加 commit 不会被带上（已实测踩过）。因此 CI 失败后
  的恢复策略绝不能是「重推 commit」。

## Goal

在 `skills/gf-workflow/references.md` 新增「Phase 3 Execution Error Classification」一节，
定义 10 类错误各自的恢复策略与重试上限；`skills/gf-workflow/SKILL.md` 的 Phase 3 Step 2
（执行引擎）在遇到执行失败时引用该表分类处理，而非笼统重试。

## Design

### 错误分类表（10 类，`references.md` 新增章节）

| 类别 | 触发场景 | 恢复策略 | 重试上限 |
|---|---|---|---|
| `build` | 编译/构建失败 | 修复代码后重新构建 | 3 |
| `test` | 测试失败 | 修复代码或测试后重跑 | 3 |
| `lint` | `cargo clippy`/`cargo fmt` 等静态检查失败 | 修复后重跑 `make lint` | 3 |
| `merge_conflict` | `git merge` 冲突（Phase 3 Step 3 local_merge） | `git merge --abort`，branch/worktree 保持不变，**不自动重试**，交回用户手动解决后重跑本 Step | 0（自动重试为 0；用户解决冲突后重新触发算新一轮，不计入本次上限） |
| `ci` | 排队合并后平台必需检查失败 | **不得重推 commit**（排队绑定的 SHA 不会带上新 commit）；先确认是否为已知 flaky（如 #373 类），触发平台侧同 SHA 重跑一次；仍失败则需新 commit + 重新排队，退回 Step 2 由执行引擎产出修复后回到 Step 3 起 | 1（仅限同 SHA 重跑一次） |
| `auth` | `gf auth status` 失败 / API 返回 401/403 | 不重试，立即升级交还用户（需要人工 `auth login`） | 0 |
| `rate_limit` | API 返回 429 / 平台限流 | 不重试，立即升级交还用户（等待或更换凭据是人工决策） | 0 |
| `network` | 连接超时/重置等瞬时网络错误 | 退避后重试 | 3 |
| `issue_unclear` | 执行中发现需求歧义，实现引擎无法判断 | 不属于可重试错误；暂停并向用户澄清，不得猜测 | 0（每次澄清后是新一轮，不计入重试） |
| `unknown` | 未归类错误 | 记录完整错误信息，保守重试一次；仍失败则升级 | 1 |

### 升级（escalation）语义

达到某类别的重试上限后：

1. 停止自动重试。
2. 向用户展示：错误类别、已尝试的恢复策略列表（含每次尝试的简要结果）、重试次数。
3. 等待用户决策（继续/换策略/中止该 Task），不得静默放弃或静默切换交付路径。

`auth`、`rate_limit`、`issue_unclear` 三类重试上限为 0，属于「一出现就升级」，不需要先尝试
再判定是否达到上限——这是与其余类别的关键区别，必须在 Phase 3 Step 2 的引用文字中明确写出，
避免被误当作「先重试 0 次再升级」这种无意义的表述。

### Phase 3 Step 2 改动点

`skills/gf-workflow/SKILL.md` Phase 3 Step 2（执行引擎）追加一句：执行过程中遇到失败，先按
`references.md` → Phase 3 Execution Error Classification 表分类，再按该类别的恢复策略与
重试上限处理；不得笼统重试或忽略分类直接升级/直接重试。

## Acceptance Criteria（同 Issue 原文，补充范围澄清）

- [ ] 错误分类表覆盖 build / test / lint / merge_conflict / ci / auth / rate_limit / network /
      issue_unclear / unknown 十类（评审建议已采纳：Context 提到的 `issue_unclear` 纳入范围，
      与 Acceptance Criteria 原文九类列表不一致之处以 Context 全量十类为准）
- [ ] 每类配独立恢复策略与最大重试次数，缺少重试上限的条目则为假
- [ ] `auth` 与 `rate_limit` 类不重试而是立即升级交还用户
- [ ] `merge_conflict` 的恢复策略与排队合并语义兼容，触发重推 commit 则为假
- [ ] Phase 3 遇到失败时引用该表分类而非笼统重试，无分类动作则为假
- [ ] 达到重试上限后升级交还用户并记录已尝试的策略
