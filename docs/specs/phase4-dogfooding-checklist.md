# Phase 4 Dogfooding Checklist

**适用版本:** v0.6.x+
**最后更新:** 2026-07-10

> 每次发布前执行此 checklist，用真实 workflow 场景验证核心命令。
> 发现 bug 时通过 `gf issue create` 手动创建 Issue，所有检查项通过后才能发布。

## Prerequisites

- [ ] 当前版本已完成 Phase 1-3（issue → plan → implement → test → PR）
- [ ] 三个平台的认证状态正常（`gf auth status --platform github/gitlab/gitcode`）
- [ ] 工作目录干净，无未提交变更

---

## GitHub Dogfooding

### 风险项：release 命令

> GitHub release 命令涉及版本标签创建和远程操作，历史上有过非交互模式兼容性问题。

- [ ] 创建 release：`gf release create v0.x.x --notes "test release"`
- [ ] 删除 release：`gf release delete v0.x.x --yes`
- [ ] 非交互模式验证：`echo "y" | gf release create v0.x.x --notes "test"`
- [ ] 清理：`gf release delete v0.x.x --yes`

### 验证要点

- release 创建后在 GitHub 网页可见
- 非交互模式下 `--yes` 标志正确传递，无交互式确认提示
- 删除操作幂等，重复删除不报错

---

## GitLab Dogfooding

### 风险项：issue label 中文编码

> GitLab API 在处理中文标签时可能出现编码问题，需验证 CRUD 操作。

- [ ] 创建中文标签：`gf label create "测试标签" --color "#ff0000"`
- [ ] 创建带中文标签的 issue：`gf issue create --title "Dogfooding test" --labels "测试标签"`
- [ ] 查询 issue 标签：`gf issue view <n>` 确认标签正确显示
- [ ] 删除测试 issue：`gf issue close <n>`
- [ ] 删除测试标签：`gf label delete "测试标签" --yes`

### 验证要点

- 中文标签在 GitLab 网页正确显示，无乱码
- label CRUD 全流程无编码错误
- `--yes` 标志在删除操作中正确传递

---

## GitCode Dogfooding

### 风险项：pr merge 非交互模式

> Issue #70: `pr merge` 在非交互 shell 中因缺少 `--yes` 传递而失败。这是 dogfooding 发现的第一个 bug。

- [ ] 创建测试 PR：`gf pr create --title "Dogfooding test" --body "test"`
- [ ] 非交互 merge：`gf pr merge <n>`（在无 TTY 的 shell 中执行）
- [ ] 验证 `--yes` 传递：确认命令不提示确认，直接完成 merge
- [ ] 清理：删除测试分支

### 验证要点

- `pr merge` 在非交互 shell（`echo "y" | ...` 或 CI 环境）中正常完成
- 不出现 `confirmation required in non-interactive mode` 错误
- `--yes` 标志正确传递到 `gc pr merge` 底层命令

---

## Bug 记录模板

发现 bug 时，使用 `gf issue create` 手动创建 Issue，正文包含以下信息：

```json
{
  "source": "dogfooding",
  "platform": "<github|gitlab|gitcode>",
  "command": "<失败的命令，如 pr merge>",
  "phase4_checklist_item": "<所属检查项，如 GitCode Dogfooding > pr merge 非交互模式>",
  "exit_code": "<命令退出码>",
  "error_code": "DOGFOODING_ERROR",
  "error_message": "<错误信息摘要>",
  "steps_to_reproduce": "<复现步骤>",
  "timestamp": "<ISO 8601 时间戳>",
  "dogfooding_version": "<当前版本，如 0.6.x>"
}
```

**注意：** 以上字段仅作为 Issue 正文的信息模板，供人工填写和整理，不对应任何自动化文件格式。

---

## Summary Report

所有检查项执行完成后，生成汇总报告：

```markdown
## Dogfooding Summary — v0.x.x

**Date:** YYYY-MM-DD
**Executor:** <name>
**Result:** PASS / FAIL

| Platform | Items | Passed | Failed | Notes |
|----------|-------|--------|--------|-------|
| GitHub   | 4     | 4      | 0      | OK    |
| GitLab   | 5     | 5      | 0      | OK    |
| GitCode  | 4     | 4      | 0      | OK    |

**Bugs Found:** 0
**Release Decision:** APPROVED / BLOCKED
```

- 如果所有检查项通过且无新增 bug → `Result: PASS`，`Release Decision: APPROVED`
- 如果有任何检查项失败或新增 bug → `Result: FAIL`，`Release Decision: BLOCKED`

---

## References

- Issue #70: `pr merge` 非交互式模式失败（Dogfooding 发现）
- Issue #73: Phase 4 Dogfooding 常态化
- Design Spec: [`docs/superpowers/specs/2026-07-10-dogfooding-checklist-design.md`](../superpowers/specs/2026-07-10-dogfooding-checklist-design.md)
- Bug Reports: 通过 `gf issue create` 手动创建
- Phase 4 Coverage TDD: [`docs/superpowers/specs/2026-07-09-phase4-coverage-tdd-design.md`](../superpowers/specs/2026-07-09-phase4-coverage-tdd-design.md)
