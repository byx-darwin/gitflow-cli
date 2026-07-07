# gitflow-autoreport-bug-params — 完整参考

> 本文档为 `gitflow-autoreport-bug` skill 的外部化引用。

## pending.json Schema

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "command": "gitflow issue create",
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
[2026-07-03T10:00:00Z] 命令: gitflow issue create | 平台: github | 错误: 401 | 失败原因: auth 检查失败
[2026-07-03T11:30:00Z] 命令: gitflow pr create | 平台: gitlab | 错误: 500 | 失败原因: issue create 失败
```

## Auth Cache 机制

| 项 | 说明 |
|----|------|
| 缓存路径 | `.cache/auth-cache/{platform}.ttl`（每平台独立） |
| 缓存内容 | Unix 时间戳（认证成功时刻） |
| TTL | 默认 86400 秒，可由 `auth_cache_ttl` 覆盖 |
| 缓存失效 | TTL 过期后下次重新调用 `gitflow-cli auth status` |

## 命令速查

```bash
command -v gitflow-cli                                   # CLI 可用性检查
gitflow-cli auth status --platform {platform}            # 平台认证状态
gitflow-cli issue list --search "[auto-report] {cmd} {err}" --state all
gitflow-cli issue create --title "[auto-report] gitflow {cmd} — {err}" \
                         --body "..." --label "auto-report"
```

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
