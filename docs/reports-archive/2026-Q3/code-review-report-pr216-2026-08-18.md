## PR Review — #216 (wf-2026-08-18-008)

> ⚠️ 自审限制：作者与评审账号均为 `byx-darwin`，以 comment 记录，不构成正式 approve。

**变更面：** hook + skill + params + bats（上报路径 `gf`→`gh` 边界修正），4 文件。

6 维度评估：

| 维度 | 判定 | 说明 |
|---|---|---|
| Correctness | ✅ | 上报路径全用 `gh`（hook 认证/去重/建 Issue/bats mock）；被上报对象保留 `gf`（标题前缀/pending command）——边界正确 |
| Architecture | ✅ | 上报机制独立于被上报组件（gf 故障仍可上报）；符合 GitHub 托管单一平台 |
| Security | ✅ | 无敏感变更；gh auth 检查保持 |
| Maintainability | ✅ | skill 476 词（<500）+ When to Use/Red Flags/Rationalization + 副本一致 |
| Test coverage | ✅ | bats mock 改 gh + 手动模拟验证；`make test` 1315 全过 |
| Documentation | ✅ | skill CLI 要求含「Why gh」理由；PR body 含 `Closes #215` |

**边界核验：** 其他 skills（gf-issue/gf-pr 等 24 个）的 gf 用法未受影响；仅「上报 bug」路径改 gh。

**Verification：** make test 1315 · clippy · check-agent-sync · skill 476 词 · CI 3/3

Closes #215
