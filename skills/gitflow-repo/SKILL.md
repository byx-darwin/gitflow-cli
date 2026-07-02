---
name: gitflow-repo
description: 仓库操作核心命令工作流 — 封装 gitflow repo 的 clone、list、create、stats、sync、view 操作，提供统一的仓库管理入口
---

# gitflow-repo

封装 `gitflow repo` 系列命令，提供仓库级别的常用操作入口。涵盖仓库克隆、列表查看、创建、统计数据获取、上游同步和详情查看，帮助开发者高效管理多个仓库。

## 工作流

### 步骤 1：确认操作类型

根据用户需求，确定要执行的仓库操作：

| 操作 | 命令 | 用途 |
|------|------|------|
| 克隆仓库 | `gitflow repo clone` | 克隆远程仓库到本地 |
| 列出仓库 | `gitflow repo list` | 列出用户或组织下的仓库 |
| 创建仓库 | `gitflow repo create` | 在远程平台创建新仓库 |
| 仓库统计 | `gitflow repo stats` | 查看仓库统计数据 |
| 同步上游 | `gitflow repo sync` | 同步上游仓库的最新变更 |
| 查看详情 | `gitflow repo view` | 查看仓库详细信息 |

### 步骤 2：执行对应操作

#### 2.1 克隆仓库

克隆远程仓库到本地：

```bash
# 克隆仓库（标准格式）
gitflow repo clone <owner>/<repo>

# 克隆到指定目录
gitflow repo clone <owner>/<repo> --dir <target-dir>

# 克隆指定分支
gitflow repo clone <owner>/<repo> --branch <branch>
```

克隆后自动执行初始化操作：

```bash
cd <repo-dir>

# 检查远程配置
git remote -v

# 如果是 fork，添加上游远程
git remote add upstream https://github.com/<original-owner>/<repo>.git 2>/dev/null || true

# 拉取所有分支
git fetch --all
```

#### 2.2 列出仓库

列出用户或组织下的仓库列表：

```bash
# 列出当前用户的仓库
gitflow repo list

# 列出指定组织的仓库
gitflow repo list --org <organization>

# 按条件过滤
gitflow repo list --visibility public
gitflow repo list --language rust
```

输出格式：

```markdown
| # | 仓库 | 可见性 | 语言 | 描述 | 更新时间 |
|---|------|--------|------|------|----------|
| 1 | org/repo-a | public | Rust | 描述 A | 2026-07-01 |
| 2 | org/repo-b | private | Rust | 描述 B | 2026-06-28 |
```

#### 2.3 创建仓库

在远程平台创建新仓库：

```bash
# 创建公开仓库
gitflow repo create --name <repo-name> --visibility public

# 创建私有仓库
gitflow repo create --name <repo-name> --visibility private

# 创建并初始化
gitflow repo create --name <repo-name> --visibility public --init
```

创建后的初始化流程：

```bash
# 如果是 --init 创建，克隆到本地
gitflow repo clone <owner>/<repo-name>
cd <repo-name>

# 初始化 Git 仓库
git init -b main

# 配置基础文件
# - README.md
# - LICENSE
# - .gitignore
# - CLAUDE.md

# 首次提交
git add .
git commit -m "chore: initial project setup"
git remote add origin <repo-url>
git push -u origin main
```

#### 2.4 查看仓库统计

调用 `gh repo view --json` 获取仓库统计数据：

```bash
# 获取仓库统计数据
gitflow repo stats

# 等价于调用 gh CLI：
gh repo view --json \
  name,description,stargazerCount,forkCount,watchers,\
  createdAt,updatedAt,pushedAt,\
  primaryLanguage,languages,\
  defaultBranchRef,isEmpty,isArchived,isFork,\
  issues,pullRequests,\
  licenseInfo,repositoryTopics
```

输出格式：

```markdown
## 仓库统计 — <owner>/<repo>

### 基本信息

| 属性 | 值 |
|------|------|
| 名称 | <repo> |
| 描述 | <description> |
| 主语言 | <language> |
| 默认分支 | <default-branch> |
| 许可证 | <license> |
| 创建时间 | <created-at> |
| 最后更新 | <updated-at> |
| 最后推送 | <pushed-at> |

### 数据统计

| 指标 | 数值 |
|------|------|
| ⭐ Stars | <stargazer-count> |
| 🍴 Forks | <fork-count> |
| 👀 Watchers | <watcher-count> |
| 📋 Open Issues | <open-issues> |
| 🔀 Open PRs | <open-prs> |

### 语言分布

| 语言 | 占比 |
|------|------|
| <language-1> | <percentage-1>% |
| <language-2> | <percentage-2>% |

### 主题标签

<topic-1> <topic-2> <topic-3>
```

#### 2.5 同步上游变更

同步上游仓库的最新变更（适用于 fork 仓库）：

```bash
# 同步上游仓库
gitflow repo sync

# 底层执行以下命令序列：
git fetch upstream
git merge upstream/main
# 或对于不同的默认分支：
# git merge upstream/<default-branch>
```

同步流程：

```bash
# 1. 确认 upstream 远程存在
git remote -v | grep upstream

# 如果不存在，需要先添加 upstream
git remote add upstream <upstream-url>

# 2. 获取上游变更
git fetch upstream

# 3. 合并上游变更到当前分支
git merge upstream/main

# 4. 如果有冲突，解决后提交
# git add <resolved-files>
# git commit -m "chore: sync with upstream/main"

# 5. 推送同步结果到 origin
git push origin <current-branch>
```

#### 2.6 查看仓库详情

查看仓库的完整信息：

```bash
# 查看当前仓库详情
gitflow repo view

# 查看指定仓库详情
gitflow repo view <owner>/<repo>
```

输出格式：

```markdown
## <owner>/<repo>

<description>

**URL:** <repo-url>
**可见性:** public / private
**主语言:** <language>
**默认分支:** <default-branch>
**许可证:** <license>

### 快速统计

⭐ <stars> | 🍴 <forks> | 👀 <watchers>

### 最近活动

- 最后推送: <pushed-at>
- Open Issues: <open-issues>
- Open PRs: <open-prs>
```

## 使用示例

### 克隆并设置上游仓库

```bash
# 克隆一个 fork 仓库
gitflow repo clone myuser/gitflow-cli

# 查看远程配置
git remote -v

# 添加上游仓库
git remote add upstream https://github.com/original-org/gitflow-cli.git

# 后续通过 sync 保持同步
gitflow repo sync
```

### 查看组织下所有 Rust 仓库

```bash
gitflow repo list --org my-org --language rust
```

### 创建新仓库并初始化

```bash
# 创建仓库
gitflow repo create --name my-new-project --visibility public --init

# 克隆到本地
gitflow repo clone myuser/my-new-project
cd my-new-project

# 开始开发
```

### 查看仓库统计并分析健康度

```bash
gitflow repo stats

# 基于统计结果分析：
# - Star 增长趋势 → 项目受欢迎程度
# - Issue / PR 积压量 → 维护活跃度
# - 最后推送时间 → 项目是否还在维护
```

## 注意事项

- `gitflow repo stats` 依赖 `gh repo view --json`，需要确保 `gh` CLI 已安装并认证
- `gitflow repo sync` 前应先确认 `upstream` 远程已配置，否则会报错
- 同步操作可能在本地产生合并冲突，需要先解决冲突再推送
- 创建仓库时应根据实际需要选择 `public` 或 `private` 可见性
- 列出仓库时可以使用过滤条件缩小范围，避免返回过多结果
- 如果 `gitflow repo` 命令尚未实现，可通过直接使用 `gh` CLI 和 `git` 命令完成等效操作
- 仓库统计信息应定期查看，帮助了解项目健康状况和维护需求
- 克隆仓库后建议立即配置 Git hooks 和开发环境（参考 `gitflow-repo-onboarding` 工作流）
