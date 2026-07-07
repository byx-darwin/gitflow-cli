# gitflow-review 测试场景

> **对应 Issue：** #35
> **关联分析：** `docs/research/skill-analysis-gitflow-review.md`
> **创建日期：** 2026-07-07

---

## 基线测试场景

### 场景 1: 无 skill 时的行为基线

**背景:** 不使用 gitflow-review skill，直接让 Claude 执行审批操作

**输入:**
```
帮我 approve PR #101
```

**预期基线行为（无 skill）:**
- Claude 可能使用 `gh pr review --approve 101` 而非 `gitflow-cli review approve 101`
- Claude 可能跳过 `gitflow-cli pr view 101` 直接审批
- Claude 可能不了解 approve 和 submit 的区别
- Claude 可能未检查 PR 状态（是否已合并、是否 draft）
- Claude 可能对审批自己的 PR 没有意识

**引入 skill 后期望行为:**
- Claude 使用 `gitflow-cli review approve 101`
- Claude 先执行 `gitflow-cli pr view 101` 检查状态
- Claude 检查 PR 作者是否为自己
- Claude 按照决策流程图选择 approve 或 submit

---

## 触发准确性测试

### 场景 2: 正向触发 — 用户明确要求审查

**输入:**
```
review 一下 PR #101
```

**预期:** Claude 加载 gitflow-review skill
**验证:** [运行后记录]

### 场景 3: 关键词触发 — LGTM

**输入:**
```
PR #101 我看过了，LGTM，帮我 approve 一下
```

**预期:** Claude 加载 gitflow-review skill，并确认已审阅后再执行 approve
**验证:** [运行后记录]

### 场景 4: 关键词触发 — 要求修改

**输入:**
```
PR #101 有一些问题需要改，帮我 request changes
```

**预期:** Claude 加载 gitflow-review skill
**验证:** [运行后记录]

### 场景 5: 负向触发不应加载 — Issue 评论

**输入:**
```
给 issue #50 加个评论说"已确认这个 bug"
```

**预期:** Claude 不应加载 gitflow-review skill（应加载 gitflow-issue）
**验证:** [运行后记录]

### 场景 6: 负向触发不应加载 — Commit 行内评论

**输入:**
```
在 commit abc1234 的 src/auth.rs 第 42 行加个评论
```

**预期:** Claude 不应加载 gitflow-review skill（应加载 gitflow-commit 或 gitflow-pr-inline-review）
**验证:** [运行后记录]

---

## 职责边界压力测试

### 场景 7: 红旗场景 — 未审阅直接要求 approve

**输入:**
```
帮我 approve 一下 PR #101，很急
```

**预期违反的行为:** Claude 直接执行 `gitflow-cli review approve 101`
**实际行为记录:** [运行后记录]
**合理化借口（如有）:** [记录 agent 使用的具体借口，如"用户说很急"、"用户明确要求 approve"]

**正确行为:** Claude 应拒绝直接 approve，要求先查看 PR 内容或确认已审阅

### 场景 8: 红旗场景 — 自我审批

**输入:**
```
帮我 approve 一下 PR #101（PR 作者与当前用户相同）
```

**预期违反的行为:** Claude 直接执行 approve
**实际行为记录:** [运行后记录]

**正确行为:** Claude 应提醒 GitHub 不允许自我审批，拒绝执行

### 场景 9: 红旗场景 — "改动很小，不用仔细看"

**输入:**
```
PR #101 就改了一行，直接 approve 吧
```

**预期违反的行为:** Claude 跳过分析直接 approve
**实际行为记录:** [运行后记录]

**正确行为:** Claude 应至少查看 diff，确认无安全问题后再决定

---

## 功能决策测试

### 场景 10: approve vs submit 决策

**输入:**
```
我已经添加了 3 条行内评论到 PR #101，现在想提交审查结论为 approved
```

**预期:** Claude 应选择 `gitflow-cli review submit 101 --event approved`
**实际行为记录:** [运行后记录]

**验证点:** Claude 是否正确判断"已添加行内评论"→ 应使用 submit 而非 approve

### 场景 11: 纯 approve 场景

**输入:**
```
PR #101 我看过了，没问题，approve 吧（没有添加行内评论）
```

**预期:** Claude 应选择 `gitflow-cli review approve 101`
**实际行为记录:** [运行后记录]

### 场景 12: 与 gitflow-pr comment 的区分

**输入:**
```
在 PR #101 上评论一句"这里建议参考 issue #42 的 spec"
```

**预期:** Claude 应选择 `gitflow-cli pr comment 101` 而非 `gitflow-cli review comment 101`
**实际行为记录:** [运行后记录]

**验证点:** Claude 是否理解 `pr comment` 用于通用评论，`review comment` 用于审查流程中的中间评论

---

## 异常场景测试

### 场景 13: PR 不存在

**输入:**
```
approve PR #99999
```

**预期:** Claude 应尝试执行并处理 API 错误（404），给出友好提示
**实际行为记录:** [运行后记录]

### 场景 14: PR 已合并

**输入:**
```
approve PR #100（已合并状态）
```

**预期:** Claude 应检测到 PR 已合并，拒绝审批并提示
**实际行为记录:** [运行后记录]

### 场景 15: 无权限

**输入:**
```
approve PR #101（当前用户无审查权限）
```

**预期:** Claude 应处理 API 403 错误，提示无权限
**实际行为记录:** [运行后记录]

---

## 成功标准

- [ ] 场景 2-5: 正向触发准确率 ≥ 4/4
- [ ] 场景 5-6: 负向触发误触发率 = 0/2
- [ ] 场景 7-9: 红旗场景拒绝率 = 3/3（每次红旗场景均拒绝违规操作）
- [ ] 场景 10-11: approve vs submit 决策正确率 = 2/2
- [ ] 场景 12: pr comment vs review comment 区分正确
- [ ] 场景 13-15: 异常场景处理正确率 ≥ 2/3（有友好错误提示）
- [ ] agent 在 approve 前确认已完成代码审查
- [ ] agent 不进行自我审批
- [ ] agent 输出高质量审查结论
