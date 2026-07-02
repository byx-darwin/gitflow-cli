---
name: gitflow-pr-apply-feedback
description: PR 审查反馈应用工作流 — 获取 PR 评论和审查意见，列出待处理项，逐条在本地应用修改，并标记已处理的评论为 resolved
---

# gitflow pr apply feedback 工作流

引导用户处理 PR 审查过程中收到的反馈。通过获取 PR 详情和审查评论，列出所有待处理的审查意见，逐条在本地代码中应用修改，并将已处理的评论标记为 resolved，确保审查反馈被完整处理。

## 工作流

### 步骤 1：获取 PR 详情和审查评论

调用 `gitflow pr view` 获取 PR 的完整信息，包括审查评论：

```bash
gitflow pr view <pr-number>
```

提取以下信息：

- PR 的基本信息（标题、分支、状态）
- 所有审查评论（review comments）
- 行内评论（inline comments）
- 总体审查结论（approve / request-changes / comment）
- 每条评论的状态（pending / resolved）

### 步骤 2：列出待处理的审查意见

筛选所有未 resolved 的审查意见，按优先级排序后列出：

**优先级排序：**

1. 🔴 `[security]` 或 `[logic]` 类问题 — 必须修复
2. 🟠 `[boundary]` 或 `[performance]` 类问题 — 强烈建议修复
3. 🟡 `[naming]` 或 `[style]` 类问题 — 建议修复
4. 🟢 建议性评论（suggestion） — 可选修复

**输出格式：**

```markdown
## 待处理审查意见

**PR:** #<number> — <标题>
**待处理总数:** <n>

### 🔴 必须修复

| # | 评论者 | 文件 | 行号 | 内容摘要 |
|---|--------|------|------|----------|
| 1 | @reviewer | src/auth.rs | 42 | SQL 注入风险 |

### 🟠 强烈建议

| # | 评论者 | 文件 | 行号 | 内容摘要 |
|---|--------|------|------|----------|

### 🟡 建议修复

| # | 评论者 | 文件 | 行号 | 内容摘要 |
|---|--------|------|------|----------|

### 🟢 可选

| # | 评论者 | 文件 | 行号 | 内容摘要 |
|---|--------|------|------|----------|
```

### 步骤 3：逐条应用修改

按照优先级从高到低，逐条处理审查意见：

#### 3.1 理解审查意见

仔细阅读每条评论，确认：

- 评论指出的具体问题是什么
- 建议的修改方案是什么
- 修改是否会引入新的问题

#### 3.2 在本地应用修改

切换到 PR 对应的本地分支，编辑对应文件的对应行：

```bash
git checkout <pr-branch>
```

使用编辑器修改代码，应用审查意见中建议的修改。

#### 3.3 验证修改

运行相关的测试和 lint 检查，确保修改没有引入回归：

```bash
cargo test -- <relevant-test>
cargo clippy -- -D warnings
```

#### 3.4 提交修改

为每条审查意见的修改创建独立的 commit（或按审查者分组）：

```bash
git add <changed-files>
git commit -m "fix: address review comment from @reviewer on <file>:<line>"
```

**Commit 消息约定：**

- 前缀使用 `fix:` 或 `refactor:` 等合适的类型
- 消息中提及评论者和文件位置，便于追溯
- 如果多条相关评论来自同一审查者，可以合并为一个 commit

### 步骤 4：标记已处理的评论为 resolved

对每条已应用的审查意见，调用相应的命令将其标记为 resolved：

```bash
gitflow pr resolve-comment <pr-number> --comment-id <comment-id>
```

如果 CLI 不支持单条 resolve，可以在所有修改完成后统一标记：

```bash
gitflow pr resolve-all <pr-number>
```

### 步骤 5：推送修改并通知审查者

推送本地修改到远程分支：

```bash
git push origin <pr-branch>
```

通知审查者修改已完成：

```bash
gitflow pr comment <pr-number> --body "已处理所有审查意见，请重新审查。

已修复：
- [x] @reviewer 关于 SQL 注入的建议（src/auth.rs:42）
- [x] @reviewer 关于边界条件的建议（src/cart.rs:15）
- [x] @reviewer 关于命名的建议（src/utils.rs:8）"
```

### 步骤 6：输出处理结果汇总

```markdown
## 审查反馈处理汇总

**PR:** #<number>
**处理评论数:** <total>
**已修复:** <resolved>
**已拒绝（附理由）:** <rejected>
**已推迟：** <deferred>

### 详细处理记录

| # | 评论者 | 文件 | 处理结果 | 说明 |
|---|--------|------|----------|------|
| 1 | @reviewer | src/auth.rs:42 | ✅ 已修复 | 改为参数化查询 |
| 2 | @reviewer | src/cart.rs:15 | ✅ 已修复 | 添加空集合检查 |
| 3 | @reviewer | src/utils.rs:8 | ❌ 已拒绝 | 当前命名符合项目约定，见回复 |
```

## 使用示例

### 处理一个有 3 条审查意见的 PR

```bash
# 获取 PR 详情和评论
gitflow pr view 101

# 切换到 PR 分支
git checkout feature/add-auth

# 逐条修改
# 评论 1：修复 SQL 注入
# 编辑 src/auth.rs:42，改为参数化查询
cargo test -p auth
git add src/auth.rs
git commit -m "fix: address SQL injection concern from @alice on src/auth.rs:42"

# 评论 2：添加边界检查
# 编辑 src/cart.rs:15，添加空集合处理
cargo test -p cart
git add src/cart.rs
git commit -m "fix: add empty list guard from @alice on src/cart.rs:15"

# 评论 3：改进命名（不同意，回复说明理由）
# 在 PR 上回复拒绝理由

# 标记评论为 resolved
gitflow pr resolve-comment 101 --comment-id c001
gitflow pr resolve-comment 101 --comment-id c002

# 推送修改
git push origin feature/add-auth

# 通知审查者
gitflow pr comment 101 --body "已处理所有审查意见，请重新审查。\n\n- [x] SQL 注入已改为参数化查询\n- [x] 已添加空集合检查\n- [ ] 命名建议已回复说明"
```

### 处理安全相关的紧急审查意见

```bash
gitflow pr view 55

# 安全审查意见应最高优先级处理
# 1. 立即切换到 PR 分支
git checkout fix/security-patch

# 2. 修复安全问题
# 编辑相关文件

# 3. 运行安全测试
cargo audit
cargo test -- security

# 4. 提交并推送
git commit -am "fix(security): address critical auth bypass from @security-reviewer"
git push origin fix/security-patch

# 5. 标记 resolved 并通知
gitflow pr resolve-comment 55 --comment-id s001
gitflow pr comment 55 --body "关键安全问题已修复，请重新审查确认。"
```

## 注意事项

- 处理审查意见时应从最高优先级开始，确保安全问题和逻辑错误首先被修复
- 如果不同意某条审查意见，应在 PR 上回复说明理由，而非直接忽略
- 每条修复应尽量独立成 commit，便于回滚和追溯
- 应用修改后必须运行相关测试，确保不引入新的回归
- 如果审查意见涉及的改动较大（如架构调整），应先与审查者讨论方案再动手
- 标记 resolved 前应确认修改确实解决了评论中指出的问题
- 对于跨多个文件的审查意见，可以在一个 commit 中统一处理，但 commit 消息应列出所有相关的评论
- 处理完毕后应主动通知审查者进行重新审查，不要等待审查者自行发现
