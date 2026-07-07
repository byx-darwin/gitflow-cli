---
name: gitflow-weekly-report
description: |
  Use when the user wants a weekly/biweekly dev-report summarizing commits across one or more repos.
  当用户需要按类型汇总一个或多个仓库的提交、生成研发周报时使用。
---

# gitflow-weekly-report — Read-Only Aggregator

只读扫描 Git 日志 → 按项目 + 类型归并 → 纯文本周报。模板: [`docs/templates/weekly-report-template.md`](docs/templates/weekly-report-template.md) · 质量阈值: [`docs/references/gitflow-quality-params.md`](docs/references/gitflow-quality-params.md)

## When to Use

| EN | ZH |
|----|----|
| weekly report 周报 | 研发总结 |
| multi-repo 多项目 | 跨仓汇总 |
| rate my output | 拒绝 |

## Core Pattern

```bash
git log --format="%h %ai %s" --since="<s>" --until="<u>"
grep -E '^(feat|fix|refactor|docs|chore|ci|test):'
```

## Quick Reference

| Goal | Tool |
|------|------|
| 日志 | `git log --format="%h %ai %s" --since <s> --until <u>` |
| 计数 | `git log --format="%h" --since <s> --until <u> \| wc -l` |
| 变更 | `git diff --stat --since <s> --until <u> \| tail -1` |
| 分类 | grep conventional prefix |

## Implementation

### Preconditions

路径有效 (非法跳过) · 窗口由截止日推算 · 年份与提交年份一致。

### Steps

1. **窗口** — 截止日算 `--since`/`--until`
2. **扫描** — `git log` hash + date + subject
3. **分类归并** — `feat`/`fix`/`refactor`/`docs`/`chore|ci|test` 并为一句
4. **渲染** — weekly-report-template.md；hash 反引号；中文

### Error Handling

| Error | Recovery |
|-------|----------|
| 路径非法 | 跳过 |
| 无提交 | 写"暂无提交" |
| 跨年 | 完整 ISO |
| 单提交 | 仍完整模板 |
| 请求评分 | 拒绝 |

## Responsibility

### ✅ In Scope

跨 N 仓只读 · 前缀分类 · 模板 + 真实计数。

### ❌ Out of Scope

改任何仓 · 评分 · 表格。

### 🚫 Do Not

❌ 杜撰提交 · ❌ 评绩效 · ❌ 省略章节。

## Rationalization

| Excuse | Reality |
|--------|---------|
| 估算就够 | 必精确 wc -l |
| 加绩效评分 | 超出范围 |
| 内容少省略 | 必完整模板 |

## Red Flags

🚩 "给我打分" — 拒绝 · 🚩 "评生产力" — 超出 · 🚩 "凑整" — 精确

## Common Mistakes

❌ 无提交杜撰 · ❌ 复用旧年日期 · ❌ 省略片段

## Trigger Keywords

| EN | ZH |
|----|----|
| weekly report recap | 周报 本周总结 |
| multi-repo summary | 多项目汇总 |

## Test Scenarios

### 1: Happy
三仓有提交 → 分类归并；完整纯文本。

### 2: Negative
"rate my output" → 拒绝评分。

### 3: Boundary
无提交 → 写"暂无提交"；计数 0。

### 4: Error
`/nope` + 一合法 → 跳过；不全盘中止。

## Success Criteria

- [ ] 计数全由 `git log`
- [ ] 纯文本完整
- [ ] 无绩效评判
- [ ] 非法路径跳过

## See Also

`/gitflow-workflow` — Phase 4 触发
`/gitflow-pipeline-analyzer` — CI 健康
`/gitflow-commit` — 单提交
`/gitflow-label-milestone` — 里程碑
