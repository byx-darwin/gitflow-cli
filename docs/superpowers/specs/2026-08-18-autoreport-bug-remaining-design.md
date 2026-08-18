# 设计文档 — 主动上报 bug 功能遗留问题修复

> **设计日期：** 2026-08-18
> **工作流：** `wf-2026-08-18-009`（standard 模式）
> **来源：** 评估报告遗留 P1/P2 + 实现/文档偏差
> **Issue：** #217

---

## 1. 目标

解决「主动上报 bug」功能 6 项遗留问题，提升可靠性（不丢报告、行为确定、可观测）+ 合规（公开预览选择）。

## 2. 变更面

| # | 子任务 | 变更 |
|---|--------|------|
| T1 | P1-2 去重命令一致性 | skill + params 统一含 `--state all` |
| T2 | P1-3 处理端日志 | hook 写 `hook.log`；skill 成功写 `processing.log` |
| T3 | P1-5 多报告（覆盖前归档） | error_reporter 写前归档旧 pending 为 `pending.<ts>.json` |
| T4 | P2-3 公开预览 | skill 创建前打印草案 + 用户选择 |
| T5 | B1 auth_cache_ttl 实现 | hook 读取 pending.json 的 `auth_cache_ttl` 覆盖硬编码 |
| T6 | B2 残留清理 | 删除积压 pending.json |

## 3. 详细设计

### T1: 去重命令一致性

**统一为：** `gh issue list --repo byx-darwin/gitflow-cli --search "[auto-report] {command} {error_code}" --state all`

- `skills/gf-autoreport-bug/SKILL.md` Workflow Step 3 加 `--state all`
- `docs/references/gf-autoreport-bug-params.md` 命令速查对齐（已是含 `--state all`）

**验证：** 两处命令一致；skill 副本同步。

### T2: 处理端日志

**hook 日志：** `.cache/bug-reports/hook.log`，追加式：
- 每次触发（检测到 pending.json）记 `[时间] detect pending.json`
- 认证失败记 `[时间] auth failed (platform)`
- banner 输出记 `[时间] banner emitted`

**skill 日志：**
- 创建成功：`.cache/bug-reports/processing.log` 记 `[时间] issue created: <url>`
- 失败：`failed.log`（已有）

**验证：** hook 触发后 hook.log 存在且含时间戳；skill 步骤补日志命令。

### T3: 多报告（覆盖前归档）

**error_reporter.rs `write_to_disk` 修改：**
```
写 pending.json 前：
  if pending.json 存在：
    rename pending.json → pending.<毫秒时间戳>.json
写 pending.json
```

**语义：** `pending.json` = 最新待处理；`pending.<ts>.json` = 历史未处理（不丢报告）。

**注意：** rename 需在 `set_pending_file_permissions` 之前（先归档旧文件再写新文件）。

**验证：** 单测：先写一次，再写第二次 → 存在 `pending.json` + `pending.*.json`。

### T4: 公开预览（skill 交互）

**skill Workflow 插入（Step 4 创建前）：**
```
打印 pending 摘要（command/platform/error_code/error_message 已脱敏）
打印拟创建 Issue 标题 + body 草案
用户选择：create / skip / modify（交互会话可做；非交互默认 create）
```

**验证：** skill 文档含预览步骤；非交互默认行为明确。

### T5: auth_cache_ttl 实现

**auto-report-bug.sh 修改：**
```
读取 pending.json 的 auth_cache_ttl 字段（缺省 86400）
AUTH_CACHE_TTL=${auth_cache_ttl:-86400}
```

**验证：** pending.json 含 `auth_cache_ttl` 时 hook 使用覆盖值；缺省 86400。

### T6: 残留清理

- 删除 `.cache/bug-reports/pending.json`（gh 已认证，确认是测试残留）
- 验证：目录干净或仅剩日志

## 4. 验收标准

- [ ] T1: skill/params 去重命令一致（含 `--state all`）；skill 副本同步
- [ ] T2: hook.log + processing.log 存在且含时间戳
- [ ] T3: 二次写入保留旧报告为 `pending.<ts>.json`
- [ ] T4: skill 含预览步骤（create/skip/modify）
- [ ] T5: hook 读取 auth_cache_ttl 覆盖（缺省 86400）
- [ ] T6: 残留 pending.json 清理
- [ ] make test + clippy + hook bats 通过

## 5. 范围外

- 不实施 P1-5 的完整队列/多消费者（覆盖前归档足够）
- 不增加 skill 自动触发之外的机制
