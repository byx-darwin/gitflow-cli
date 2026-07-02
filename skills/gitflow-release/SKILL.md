---
name: gitflow-release
description: gitflow-cli 的 Release 操作命令封装，支持创建、列表、查看、编辑、上传/下载资源和删除
---

# gitflow-cli release

封装 `gitflow-cli release` 命令族，用于在 GitHub/GitLab/GitCode 等平台上管理版本发布（Release）。

## 命令概览

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新 Release |
| `list` | 列出仓库的 Release 列表 |
| `view` | 查看指定 Release 的详情 |
| `edit` | 编辑 Release 元数据 |
| `upload` | 上传资源文件到 Release |
| `download` | 下载 Release 的资源文件 |
| `delete` | 删除指定 Release |

## 参数说明

### `gitflow-cli release create`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `--tag` | string | 是 | 关联的 Git tag 名 |
| `--name` | string | 否 | Release 标题 |
| `--body` | string | 否 | Release 正文（Markdown） |
| `--draft` | flag | 否 | 以草稿方式创建 |
| `--prerelease` | flag | 否 | 标记为预发布版本 |
| `--target` | string | 否 | 目标 commitish（默认当前分支 HEAD） |

### `gitflow-cli release list`

无需额外参数。

### `gitflow-cli release view`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<tag>` | string | 是 | Release 的 tag 名 |

### `gitflow-cli release edit`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<tag>` | string | 是 | Release 的 tag 名 |
| `--name` | string | 否 | 新标题 |
| `--body` | string | 否 | 新正文（Markdown） |
| `--draft` | flag | 否 | 切换为草稿状态 |
| `--prerelease` | flag | 否 | 切换为预发布状态 |

### `gitflow-cli release upload`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<tag>` | string | 是 | Release 的 tag 名 |
| `--file` | string | 是 | 本地文件路径 |
| `--asset-name` | string | 否 | 资源显示名（默认使用文件名） |

### `gitflow-cli release download`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<tag>` | string | 是 | Release 的 tag 名 |
| `--asset` | string | 是 | 资源文件名 |
| `--dest` | string | 否 | 本地目标路径（默认当前目录） |

### `gitflow-cli release delete`

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `<tag>` | string | 是 | Release 的 tag 名 |

## 使用示例

### 创建正式版本的 Release

```bash
gitflow-cli release create --tag v1.0.0 --name "Version 1.0.0" --body "## 新特性\n- 用户认证\n- Issue 管理"
```

### 创建预发布草稿版本

```bash
gitflow-cli release create --tag v2.0.0-rc1 --name "v2.0 Release Candidate 1" --draft --prerelease --target main
```

### 上传编译产物到 Release

```bash
gitflow-cli release upload v1.0.0 --file ./dist/app-linux-amd64.tar.gz --asset-name "app-linux-amd64-v1.0.0.tar.gz"
```

### 下载 Release 资源并编辑元数据

```bash
gitflow-cli release download v1.0.0 --asset "app-linux-amd64-v1.0.0.tar.gz" --dest ./downloads/
gitflow-cli release edit v1.0.0 --body "## 更新\n修复了安装脚本问题"
```
