---
name: gitflow-auth
description: gitflow CLI 的认证操作命令封装，支持登录、登出、状态查询和 Token 获取
---

# gitflow auth

封装 `gitflow auth` 命令族，用于管理 GitHub/GitLab/GitCode 等平台的认证状态和凭据。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `login` | 执行登录流程（交互式） |
| `logout` | 清除本地凭据，执行登出 |
| `status` | 查询当前认证状态 |
| `token` | 获取当前有效的访问 Token |

## 参数说明

### `gitflow auth login`

无需额外参数。执行后会引导用户完成交互式认证流程，登录成功后凭据会自动持久化到本地存储。

### `gitflow auth logout`

无需额外参数。执行后会清除本地存储的所有认证凭据。

### `gitflow auth status`

无需额外参数。返回当前认证状态，包括是否已登录、当前用户名和已授权的权限范围（scopes）。

### `gitflow auth token`

无需额外参数。返回当前有效的访问 Token，可用于后续的平台 API 调用。

## 使用示例

### 首次使用，执行登录

```bash
gitflow auth login
```

登录成功后会显示当前用户和权限范围。

### 检查认证状态

```bash
gitflow auth status
```

输出示例：
```json
{
  "loggedIn": true,
  "user": "octocat",
  "scopes": ["repo", "read:org"]
}
```

### 获取 Token 用于脚本调用

```bash
TOKEN=$(gitflow auth token)
curl -H "Authorization: token $TOKEN" https://api.github.com/user
```

### 切换账号前登出

```bash
gitflow auth logout
gitflow auth login
```
