# Dogfooding Report — v1.0.0 Release

**Date:** 2026-08-04 ~ 2026-08-05
**Executor:** Claude gf-workflow orchestrator（wf-issue-113）
**Scope:** Full release dogfooding（GitHub 可执行项）
**Result:** PASS（#130 已修复并复验）

## Platform Results

| Platform | Items | Passed | Failed | Skipped | Notes |
|----------|-------|--------|--------|---------|-------|
| GitHub   | 4     | 4      | 0      | 0       | 首轮发现 #130，修复后复验全部通过 |
| GitLab   | 5     | 0      | 0      | 5       | ⏭️ 环境不可用（未认证） |
| GitCode  | 4     | 0      | 0      | 4       | ⏭️ `gitcode` CLI 未安装 |

## GitHub Risk Items

| Check | Status | Notes |
|-------|--------|-------|
| `gf release create --tag-name <tag>` | ✅ PASS | Release 创建成功，URL 有效 |
| Release 在 GitHub 网页可见 | ✅ PASS | 确认 |
| 非交互模式 `--yes` 传递 | ✅ PASS | `echo "y" \| gf release delete` 无提示完成 |
| 删除操作幂等，重复删除不报错 | ✅ PASS | 修复后复验：重复删除成功 |
| 删除后 git tag 清理（`--cleanup-tag`） | ✅ PASS | 修复后复验：tag 随 release 删除 |

## Bug 记录

### #130: `release delete` git tag 残留 + 幂等性失败（已修复 ✅）

- **现象:** 删除 GitHub Release 后 git tag 残留；重复删除已不存在的 release 报错
- **根因:** `crates/github/src/release.rs` 的 `delete` 调用 `gh release delete` 未传 `--cleanup-tag`；对 "release not found" 未做幂等处理
- **修复:** PR #131 — 传 `--cleanup-tag` + `is_release_not_found` 幂等判断（JSON code NOT_FOUND 或纯文本 "not found"）
- **测试:** 新增 3 个测试（幂等 JSON / 幂等文本 / `--cleanup-tag` 参数断言），202 个 gf-github 测试通过，clippy pedantic 干净
- **复验:** 创建 → 删除 → tag 清理确认 → 重复删除幂等，全部 PASS

## 环境约束

- GitLab: 未认证（`glab auth` 401），中文标签 CRUD 跳过
- GitCode: `gitcode` CLI 二进制未安装，PR 操作无法真正执行

## Bugs Found

1（#130，已修复并复验）

## Release Decision

**APPROVED** — 所有可执行风险项通过，#130 已修复。GitLab/GitCode 为环境约束（记录，不阻塞）。
