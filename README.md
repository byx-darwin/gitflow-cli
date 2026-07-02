# gitflow-cli

[![CI](https://github.com/byx-darwin/gitflow-cli/actions/workflows/build.yml/badge.svg)](https://github.com/byx-darwin/gitflow-cli/actions/workflows/build.yml)

多平台 Git 锻造 CLI 工具 — 统一 GitHub、GitLab 和 GitCode 的命令行接口。

## 安装

### Homebrew (macOS)

```bash
brew tap byx-darwin/gitflow-cli
brew install gitflow-cli
```

### Cargo

```bash
cargo install gitflow-cli
```

### 源码编译

```bash
git clone https://github.com/byx-darwin/gitflow-cli
cd gitflow-cli
make build
```

## 快速开始

```bash
# 1. 安装 Shell 补全
gitflow completions --install

# 2. 登录平台
gitflow auth login

# 3. 创建第一个 Issue
gitflow issue create --title "feat: my first feature" --label enhancement
```

## 命令一览

| 命令 | 功能 | 示例 |
|------|------|------|
| `gitflow issue {create,list,view,close,reopen,comment}` | Issue 管理 | `gitflow issue list --state open` |
| `gitflow pr {create,list,view,close,merge,checkout}` | Pull Request 管理 | `gitflow pr create --title "feat: ..." --head my-branch` |
| `gitflow release {create,list,view,edit}` | 发布管理 | `gitflow release create --tag v1.0.0` |
| `gitflow review {comment,approve,request-changes,submit}` | 代码审查 | `gitflow review approve 42` |
| `gitflow auth {login,logout,status,token}` | 认证管理 | `gitflow auth status` |
| `gitflow label {create,list,edit,delete}` | 标签管理 | `gitflow label create --name "bug" --color "d73a4a"` |
| `gitflow milestone {create,list,edit,close,reopen}` | 里程碑管理 | `gitflow milestone list` |
| `gitflow commit {view,diff,patch,comment}` | 提交操作 | `gitflow commit diff abc123` |
| `gitflow pipeline {status,logs,jobs,report}` | CI/CD 流水线 | `gitflow pipeline report --days 7` |
| `gitflow skills {install,list,uninstall}` | Skills 管理 | `gitflow skills install` |
| `gitflow completions {bash,zsh,fish}` | Shell 补全 | `gitflow completions --install` |

支持 `--platform github|gitlab|gitcode` 和 `--output json|text` 全局参数。

## Skills 列表（26 个）

### 核心命令层

| Skill | 说明 |
|-------|------|
| gitflow-issue | Issue 操作命令封装 |
| gitflow-issue-create | Issue 创建引导工作流 |
| gitflow-pr | PR 操作命令封装 |
| gitflow-pr-create | PR 创建引导工作流 |
| gitflow-release | Release 操作命令封装 |
| gitflow-review | 代码审查命令封装 |
| gitflow-auth | 认证操作命令封装 |
| gitflow-commit | Commit 操作命令封装 |
| gitflow-label-milestone | Label 和 Milestone 命令封装 |
| gitflow-repo | 仓库操作命令封装 |

### 工作流层

| Skill | 说明 |
|-------|------|
| gitflow-pr-review | 6 维度代码审查工作流 |
| gitflow-issue-review | Issue 需求分析 |
| gitflow-issue-triage | Issue 分类分流 |
| gitflow-pr-inline-review | PR 行内评论 |
| gitflow-pr-apply-feedback | 应用审查反馈 |
| gitflow-release-helper | 发布助手 |
| gitflow-pipeline-analyzer | 流水线分析 |
| gitflow-repo-onboarding | 仓库入门指引 |
| gitflow-security-check | 安全审计工作流 |

### 研发辅助层

| Skill | 说明 |
|-------|------|
| gitflow-workflow | 工作流编排 |
| gitflow-quality | 质量门检查 |
| gitflow-precommit | Pre-commit 检查 |
| gitflow-regression | 冒烟测试 |
| gitflow-label-stats | 标签统计分析 |
| gitflow-autoreport-bug | Bug 自动上报 |

## 开发

```bash
# 安装开发工具
make install-tools

# 构建
make build

# 运行测试
make test

# 代码检查
make lint

# TDD 开发模式
make test-watch
```

详见 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [CLAUDE.md](CLAUDE.md)。

## 许可证

MIT — 详见 [LICENSE.md](LICENSE.md)。
