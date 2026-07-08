# Issue #59 修复计划：GitCode pr merge 命令不支持策略标志

## 问题描述

`gitflow-cli pr merge` 命令在 GitCode 平台下无法正常工作，所有合并策略（merge/squash/rebase）均报错：

```
× Failed to merge pr #13: platform error: gitcode: Error: unknown flag: --merge
```

## 根因分析

`crates/gitcode/src/pr.rs:269` 的 `merge` 方法将 `--merge`/`--squash`/`--rebase` 标志传递给 GitCode CLI (`gc pr merge`)，但 GitCode CLI 不支持这些标志格式。

对比 GitHub 实现：
- GitHub CLI (`gh pr merge`) 支持 `--merge`/`--squash`/`--rebase` 标志
- GitCode CLI (`gc pr merge`) 使用不同的语法或不支持策略标志

## 修复方案

**方案**：移除 GitCode 实现中的策略标志，添加警告日志说明暂不支持合并策略

**理由**：
1. 当前命令完全无法使用，需要优先修复基本功能
2. 无法访问 GitCode CLI 文档确认正确的策略标志语法
3. 移除策略标志后，至少可以让基本的 `pr merge` 命令工作
4. 后续可以通过调研 GitCode CLI 文档来添加策略支持

## 任务清单

### Task 1: 修复 GitCode pr merge 实现

**文件**：`crates/gitcode/src/pr.rs`

**修改内容**：
1. 移除 `merge` 方法中的 `match strategy` 分支
2. 不再传递 `--merge`/`--squash`/`--rebase` 标志
3. 添加警告日志：当用户指定策略时，记录 "Merge strategies are not yet supported on GitCode platform"
4. 更新文档注释，说明当前不支持合并策略

**TDD 循环**：
- [ ] **RED**：编写测试验证修复后的行为
  - 测试 1：调用 `merge` 方法时不应传递策略标志
  - 测试 2：当指定策略时，应记录警告日志
- [ ] **GREEN**：实现最小修复使测试通过
- [ ] **REFACTOR**：优化代码，确保符合 Rust 规范
- [ ] **验证**：运行 `make test` 确认所有测试通过

**代码审查**：
- [ ] 调用 `superpowers:requesting-code-review` 进行代码审查
- [ ] 根据审查意见修复问题

**提交**：
- [ ] `git add -A`
- [ ] `git commit -m "fix(gitcode): remove unsupported strategy flags from pr merge (#59)"`

### Task 2: 质量关卡

- [ ] 调用 `gitflow-quality` 技能，运行 6 项质量检查
  ```
  使用 gitflow-quality 技能，对当前分支运行 6 项质量检查。
  ```
  检查项：
  - Build 检查
  - Test 检查
  - Coverage 检查（默认 > 80%）
  - Format 检查
  - Static 检查
  - Pre-commit 检查
- [ ] 确认 Quality Report 结果为 `ALL CHECKS PASSED`
- [ ] 如有失败项，按报告修复建议修复后重新运行

### Task 3: 交付

- [ ] 创建 PR：
  ```bash
  gitflow-cli pr create --title "fix(gitcode): remove unsupported strategy flags from pr merge" --body "## Issues Fixed\n- Closes #59\n\n## Changes\n- Remove --merge/--squash/--rebase flags from GitCode pr merge implementation\n- Add warning log when merge strategy is specified\n- Update documentation to reflect current limitation"
  ```
- [ ] PR 审查：调用 `gitflow-pr-review` 技能
- [ ] 合并 PR：
  ```bash
  gitflow-cli pr merge <PR_NUMBER>
  ```

### Task 4: 收尾

- [ ] 同步 Issue 状态为 done：
  ```bash
  gitflow-cli issue update 59 --state done
  ```
- [ ] 关闭 Issue：
  ```bash
  gitflow-cli issue close 59
  ```
- [ ] 验证所有关联 Issue 已关闭：
  ```bash
  gitflow-cli issue view 59
  ```
- [ ] 清理 worktree 和分支：
  ```bash
  cd /Users/baoyx/Documents/workspace/github.com/byx-darwin/gitflow-cli
  git checkout main
  git pull origin main
  git worktree remove ../gitflow-cli-59
  git branch -d fix/issue-59
  ```

## 验收标准

- [ ] `gitflow-cli pr merge <NUMBER>` 在 GitCode 平台可以正常执行
- [ ] 当用户指定 `--strategy` 参数时，命令仍能执行（忽略策略参数）
- [ ] 警告日志正确记录 "Merge strategies are not yet supported on GitCode platform"
- [ ] 所有测试通过
- [ ] 代码审查完成
- [ ] PR 已合并
- [ ] Issue #59 已关闭

## 风险与后续工作

**风险**：
- 移除策略标志后，用户无法在 GitCode 平台使用 squash/rebase 合并策略
- 如果 GitCode CLI 实际支持其他格式的策略标志，当前方案可能不是最优解

**后续工作**：
- 调研 GitCode CLI 官方文档，确认正确的合并策略语法
- 如果 GitCode CLI 支持策略标志，重新实现策略支持
- 考虑在命令输出中提示用户当前平台的限制
