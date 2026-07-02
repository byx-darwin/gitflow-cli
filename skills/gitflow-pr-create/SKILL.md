---
name: gitflow-pr-create
description: 引导用户完成 Pull Request 创建工作流 — 检查分支、变更和 base 状态，填写标题描述后调用 gitflow-cli pr create
---

# gitflow-cli pr create 工作流

引导用户通过结构化的流程创建高质量的 Pull Request，包括分支状态检查、变更审查、标题和描述编写，最终调用 `gitflow-cli pr create` 完成创建。

## 工作流

### 步骤 1：检查当前分支

确认当前所在分支及其跟踪关系：

```bash
git branch --show-current
git rev-parse --abbrev-ref --symbolic-full-name @{u}
```

- 确保当前不在 `main` 或其他保护分支上
- 确认分支已推送到远程（否则 PR 无法创建）

### 步骤 2：检查变更范围

查看当前分支相对于 base branch 的变更：

```bash
git diff --stat main...HEAD
git log --oneline main...HEAD
```

- 确认提交信息遵循 conventional commits 格式
- 如果有不符合的提交，建议用户使用 `git commit --amend` 或 `git rebase -i` 修正

### 步骤 3：检查 base branch 是否最新

确保 base branch（通常是 `main`）在本地是最新的：

```bash
git fetch origin main
git merge-base --is-ancestor origin/main HEAD
```

- 如果不是最新（merge-base 检查失败），建议用户先执行 `git rebase origin/main` 或 `git merge origin/main`
- 避免创建基于过时 base 的 PR，减少合并冲突

### 步骤 4：收集 PR 标题

引导用户提供清晰、具体的 PR 标题，遵循 conventional commits 格式：

| 前缀 | 用途 |
|------|------|
| `feat:` | 新功能 |
| `fix:` | 缺陷修复 |
| `docs:` | 文档更新 |
| `refactor:` | 代码重构 |
| `chore:` | 维护性任务 |
| `test:` | 测试相关 |
| `perf:` | 性能优化 |

示例：`feat(cli): add two-factor authentication support`

### 步骤 5：收集 PR 描述

引导用户提供结构化的 PR 描述（Markdown 格式）。推荐模板：

```markdown
## 变更说明

<!-- 简要说明本次变更的核心内容 -->

## 相关 Issue

Closes #N  <!-- 关联的 Issue 编号，多个用逗号分隔 -->

## 验证步骤

<!-- 审查者如何验证本次变更有效 -->

## 截图 / 示例输出

<!-- 如有 UI 变更或命令行输出，提供截图 -->

## Checklist

- [ ] 代码遵循项目规范
- [ ] 已添加/更新测试
- [ ] 文档已更新
- [ ] 无敏感信息泄露
```

### 步骤 6：确定目标分支

确认 `--head`（来源分支）和 `--base`（目标分支）：

- `--head`：当前分支名
- `--base`：通常为目标保护分支（如 `main`），需与用户确认

### 步骤 7：创建 PR

调用 `gitflow-cli pr create` 命令：

```bash
gitflow-cli pr create --title "<标题>" --body "<描述>" --head <来源分支> --base <目标分支>
```

如果需要以草稿方式创建（后续再标记为 ready），添加 `--draft` 标志。

### 步骤 8：输出结果

解析命令输出，提取并展示 PR URL。建议用户：

- 草稿 PR：审查自身变更后可调用 `gitflow-cli pr ready <number>` 标记为就绪
- 正式 PR：通知相关审查者进行审查

## 使用示例

### 创建功能分支 PR

```bash
# 检查分支状态后执行：
gitflow-cli pr create \
  --title "feat(cli): add two-factor authentication support" \
  --body "## 变更说明\n实现 TOTP 双因素认证流程\n\n## 相关 Issue\nCloses #42\n\n## 验证步骤\n1. 运行 \`make test\`\n2. 使用 auth skill 执行登录，验证 TOTP 码校验" \
  --head feature/two-factor-auth \
  --base main
```

### 以草稿方式创建 PR

```bash
# 变更尚未完成，先创建草稿 PR 进行早期审查
gitflow-cli pr create \
  --title "WIP: feat(cache): implement LRU cache with TTL" \
  --body "## 变更说明\n草稿：LRU 缓存实现，性能测试待补充\n\n## 相关 Issue\nRelated #55" \
  --head feature/lru-cache \
  --base main \
  --draft
```

### 修复类 PR

```bash
gitflow-cli pr create \
  --title "fix(auth): handle expired token redirect loop" \
  --body "## 变更说明\n修复 token 过期时的重定向循环问题\n\n## 相关 Issue\nCloses #73\n\n## Checklist\n- [x] 已添加回归测试\n- [x] 无敏感信息泄露" \
  --head fix/redirect-loop \
  --base main
```

## 注意事项

- 必须确认 base branch 是最新的，避免合并冲突
- 标题应精确描述变更内容，方便审查者快速理解
- PR 描述中的 Checklist 应在创建前逐项确认
- 如果变更较大，建议先用 `--draft` 创建，完成后再标记为 ready
