---
name: gitflow-issue-create
description: 引导用户完成 Issue 创建工作流 — 从模板填写到调用 gitflow issue create 并输出 Issue URL
---

# gitflow issue create 工作流

引导用户通过结构化的交互流程创建高质量的 Issue，包括标题、描述、标签和里程碑的配置，并调用 `gitflow issue create` 完成创建。

## 工作流

### 步骤 1：收集 Issue 标题

引导用户提供清晰、具体的 Issue 标题。标题应遵循 conventional commits 前缀约定：

- `feat:` 新功能
- `fix:` 缺陷修复
- `docs:` 文档更新
- `refactor:` 代码重构
- `chore:` 维护性任务
- `test:` 测试相关
- `perf:` 性能优化

示例：`feat(auth): add two-factor authentication support`

### 步骤 2：收集 Issue 描述

引导用户提供结构化的描述（Markdown 格式）。推荐模板：

```markdown
## 背景

<!-- 说明问题或需求的上下文 -->

## 目标

<!-- 完成后应该达到什么效果 -->

## 验收标准

- [ ] 标准 1
- [ ] 标准 2
- [ ] 标准 3

## 备注

<!-- 补充信息、参考链接、设计图等 -->
```

### 步骤 3：配置标签（可选）

询问用户是否需要附加标签。常见标签：

| 标签 | 用途 |
|------|------|
| `bug` | 缺陷 |
| `enhancement` | 功能增强 |
| `documentation` | 文档 |
| `high-priority` | 高优先级 |
| `good-first-issue` | 适合新人 |

可通过多次调用 `--label` 参数添加多个标签。

### 步骤 4：配置指派人（可选）

询问是否需要指定 Issue 指派人。需提供指派人的登录名。

### 步骤 5：创建 Issue

调用 `gitflow issue create` 命令，传入收集到的参数：

```bash
gitflow issue create --title "<标题>" --body "<描述>" --label <标签1> --label <标签2> --assignee <指派人>
```

### 步骤 6：输出结果

解析命令输出，提取并展示 Issue URL，方便用户后续追踪。

## 使用示例

### 创建 bug 类 Issue

```bash
# 交互式引导后等价执行：
gitflow issue create \
  --title "fix(auth): login redirect loops on expired token" \
  --body "## 背景\nAuth middleware 在 token 过期时产生重定向循环\n\n## 目标\n过期后应跳转登录页而非循环\n\n## 验收标准\n- [ ] 过期 token 不产生循环\n- [ ] 正确跳转登录页" \
  --label bug \
  --label high-priority \
  --assignee alice
```

### 创建轻量功能 Issue

```bash
gitflow issue create \
  --title "feat(cli): add --dry-run flag to pr create" \
  --body "## 背景\n创建 PR 前需要预览参数\n\n## 目标\n添加 --dry-run 标志，只打印最终命令不执行" \
  --label enhancement
```

### 仅标题的最小创建

```bash
gitflow issue create --title "docs: update CLAUDE.md with new lint rules"
```

## 注意事项

- 标题必须遵循 conventional commits 格式，便于自动归类
- 描述中的验收标准应使用 Markdown checkbox（`- [ ]`）格式
- 如果用户未指定标签，不附加 `--label` 参数
- 创建完成后应展示 Issue URL 供用户确认
