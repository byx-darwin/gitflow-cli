---
name: gitflow-pr-inline-review
description: PR 行内评论工作流 — 获取 PR diff 并逐文件分析，针对具体代码行生成行内评论，覆盖逻辑错误、安全隐患、命名规范、边界条件四个维度
---

# gitflow pr inline review 工作流

引导用户对 Pull Request 进行精细化的行内代码审查。通过获取 PR diff，逐文件分析代码变更，针对具体的代码行生成行内评论，并将评论通过 `gitflow commit comment` 命令直接发布到对应的代码行上，帮助 PR 作者快速定位和修复问题。

## 工作流

### 步骤 1：获取 PR Diff

调用 `gitflow pr diff` 获取 PR 的完整 diff：

```bash
gitflow pr diff <pr-number>
```

解析 diff 输出，提取以下信息：

- 变更文件列表
- 每个文件的 hunk（变更块）
- 每个 hunk 中的新增行（`+` 开头）和删除行（`-` 开头）
- 每行的文件路径和行号

### 步骤 2：逐文件分析

对每个变更文件，按照以下四个维度进行代码审查：

#### 2.1 逻辑错误

检查：

- 条件判断是否正确（边界值、空值、类型转换）
- 循环逻辑是否正确（终止条件、迭代变量）
- 状态变更是否一致
- 异步操作是否正确 await
- 错误传播是否正确处理

#### 2.2 安全隐患

检查：

- 用户输入是否经过验证
- 是否存在注入风险（SQL、命令、XSS）
- 敏感信息是否泄露（日志、错误消息）
- 权限检查是否完整
- 加密/哈希算法是否安全

#### 2.3 命名规范

检查：

- 变量/函数命名是否符合项目约定（`snake_case`、`camelCase` 等）
- 命名是否清晰、自解释
- 是否存在缩写不当或含义模糊的名称
- 布尔变量是否以 `is_`、`has_`、`can_` 等前缀命名

#### 2.4 边界条件

检查：

- 空集合/空字符串是否处理
- 整数溢出/下溢是否考虑
- 并发场景下的竞态条件
- 超时和重试机制是否合理
- 大输入是否会导致性能问题

### 步骤 3：生成行内评论

对每个发现的问题，生成行内评论。评论格式：

```markdown
**[维度标签]** 问题简述

问题详情和原因说明。

**建议修改：**

```<language>
// 建议的修改代码
```
```

维度标签对应：

| 维度 | 标签 |
|------|------|
| 逻辑错误 | `[logic]` |
| 安全隐患 | `[security]` |
| 命名规范 | `[naming]` |
| 边界条件 | `[boundary]` |

### 步骤 4：发布行内评论

调用 `gitflow commit comment` 将评论发布到具体的代码行：

```bash
gitflow commit comment <commit-sha> --body "<评论内容>" --path <file-path> --line <line-number>
```

**注意事项：**

- `<commit-sha>` 使用 PR 的 HEAD commit SHA
- `<file-path>` 是相对于仓库根目录的文件路径
- `<line-number>` 是变更后的行号（diff 中 `+` 行的行号）
- 每个问题一条评论，避免将多个问题合并

### 步骤 5：输出审查汇总

所有行内评论发布完毕后，输出审查汇总：

```markdown
## 行内审查汇总

**PR:** #<number>
**审查文件数:** <n>
**发现问题数:** <total>

### 按维度统计

| 维度 | 数量 |
|------|------|
| 逻辑错误 | <n> |
| 安全隐患 | <n> |
| 命名规范 | <n> |
| 边界条件 | <n> |

### 问题清单

| 文件 | 行号 | 维度 | 简述 |
|------|------|------|------|
| <file> | <line> | [logic] | 问题简述 |
| ... | ... | ... | ... |
```

## 使用示例

### 对一个功能 PR 进行行内审查

```bash
# 获取 PR diff
gitflow pr diff 101

# 分析后发现 3 个问题，逐一行内评论

# 问题 1：安全隐患 — 未验证的用户输入
gitflow commit comment abc1234 --body "**[security]** 用户输入未经验证

\`username\` 直接拼接到 SQL 查询中，存在 SQL 注入风险。

**建议修改：**

\`\`\`rust
let username = validate_input(&username)?;
db.query(\"SELECT * FROM users WHERE name = ?\", &[&username])
\`\`\`" --path src/user/service.rs --line 42

# 问题 2：逻辑错误 — 边界条件遗漏
gitflow commit comment abc1234 --body "**[logic]** 空集合未处理

当 \`items\` 为空时，\`items[0]\` 会 panic。

**建议修改：**

\`\`\`rust
let first = items.first().ok_or(Error::EmptyList)?;
\`\`\`" --path src/cart/calculator.rs --line 87

# 问题 3：命名规范
gitflow commit comment abc1234 --body "**[naming]** 命名不够清晰

\`calc\` 缩写不当，建议使用完整名称。

**建议修改：** 将 \`calc\` 重命名为 \`calculate_total\`" --path src/order/processor.rs --line 15
```

### 对修复 PR 进行行内审查

```bash
gitflow pr diff 55

# 修复 PR 通常改动较小，重点关注修复是否完整
gitflow commit comment def5678 --body "**[boundary]** 修复不完整

虽然修复了 \`None\` 的情况，但 \`Some(0)\` 仍然会导致除零错误。

**建议修改：**

\`\`\`rust
if count.map_or(true, |c| c == 0) {
    return Err(Error::DivisionByZero);
}
\`\`\`" --path src/stats/averager.rs --line 23
```

## 注意事项

- 行内评论应针对具体的代码行，避免泛泛而谈。如果问题涉及整体架构，使用 `gitflow review comment` 发表总体评论
- 每条评论只关注一个问题，保持聚焦，便于作者逐条处理
- 评论语气应建设性而非指责性，提供具体的修改建议
- 行号应以 diff 中的新文件行号为准（`+` 行），而非旧文件行号
- 对于删除的代码行，通常不需要行内评论，除非删除本身存在问题
- 如果发现大量问题（>15 个），建议先与 PR 作者沟通整体方案，再逐行评论
- 安全隐患（`[security]`）类问题应优先评论，确保作者首先注意到
