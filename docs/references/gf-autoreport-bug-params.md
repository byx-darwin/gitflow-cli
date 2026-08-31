# gf-autoreport-bug-params — 完整参考

> 本文档为 `gf-autoreport-bug` skill 的外部化引用。

## pending.json Schema

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "gh issue create",
  "platform": "github",
  "error_code": "401",
  "error_message": "Unauthorized",
  "timestamp": "2026-07-03T10:00:00Z",
  "auth_cache_ttl": 86400
}
```

`auth_cache_ttl` 可选，缺省 86400 秒；其余字段必填。
缺少任一字段即视为无效 JSON。

## failed.log 格式

```
[2026-07-03T10:00:00Z] 命令: gh issue create | 平台: github | 错误: 401 | 失败原因: auth 检查失败
[2026-07-03T11:30:00Z] 命令: gh pr create | 平台: gitlab | 错误: 500 | 失败原因: issue create 失败
```

## Auth Cache 机制

| 项 | 说明 |
|----|------|
| 缓存路径 | `.cache/auth-cache/{platform}.ttl`（每平台独立） |
| 缓存内容 | Unix 时间戳（认证成功时刻） |
| TTL | 默认 86400 秒，可由 `auth_cache_ttl` 覆盖 |
| 缓存失效 | TTL 过期后下次重新调用 `gh auth status` |

## 命令速查

```bash
command -v gh                                   # CLI 可用性检查
gh auth status                                  # GitHub 认证状态
gh issue list --repo {repo} --search "[auto-report] {cmd} {err}" --state all
gh issue create --repo {repo} --title "[auto-report] gf {cmd} — {err}" \
                         --body "..." --label "auto-report"
```

`{repo}` 来自 `Cargo.toml` 的 `repository` 字段（编译期通过 `CARGO_PKG_REPOSITORY` 读取），不再是硬编码字面量。

## 安全网关（2026-08-30 加固）

- **CI 跳过**：`error_reporter` 检测到 `CI`/`GITHUB_ACTIONS`/`GITLAB_CI`/`CI_PIPELINE_ID`/`CIRCLECI`/`BUILDKITE`/`JENKINS_URL` 任一环境变量存在时，直接跳过写入 `pending.json`，不会产生上报。
- **标签预检查**：Stop Hook 在认证成功后会先执行 `gh label list --repo {repo} --search auto-report`，标签不存在则打印修复命令并停止，不再触发 skill。
- **非交互默认值**：Preview 阶段在非交互场景（Stop Hook 触发即是此场景）下默认 `skip`，不会自动创建 Issue；只有交互式确认才会创建。
- **仓库参数化（2026-08-31 加固）**：目标仓库不再硬编码为 `byx-darwin/gitflow-cli`，而是在 `gf skills install` 安装 Stop Hook 时从 `Cargo.toml` 的 `repository` 字段解析并作为参数传给 `hooks/auto-report-bug.sh`，模板 fork 只需更新该字段即可自动定位到自己的仓库。

## Issue 正文模板

```
## 错误摘要

- 命令: {command}
- 平台: {platform}
- 错误码: {error_code}
- 错误信息: {error_message}

## 可能原因

（基于 error_code / error_message 推断的可能根因）

## 建议修复方向

（具体建议或排查方向）

## 严重程度

critical / high / medium / low

## 环境信息

- 时间: {timestamp}
- ID: {id}
```
