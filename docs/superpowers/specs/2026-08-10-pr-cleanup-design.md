# gf pr cleanup 命令设计

**Issue**: #174
**日期**: 2026-08-10
**状态**: 已批准

## 概述

添加 `gf pr cleanup` 命令，安全地处理 PR 合并后的分支和 worktree 清理工作。解决使用 `gh pr merge --delete-branch` 在 worktree 中失败的问题，提供更好的 worktree 工作流体验。

## 目标

1. 提供安全的 post-merge 清理命令
2. 自动处理 worktree 场景
3. 支持单个、多个和批量清理
4. 强制执行安全检查（受保护分支、当前分支）
5. 提供 dry-run 模式预览操作

## 非目标

- 不替代 `gh pr merge`（仅处理合并后的清理）
- 不处理未合并/未关闭的 PR（除非使用 `--force`）
- 不实现远程分支保护状态查询（Phase 2 考虑）

## 命令接口

### 基本用法

```bash
gf pr cleanup <NUMBERS...> [OPTIONS]
gf pr cleanup --merged [OPTIONS]
gf pr cleanup --closed [OPTIONS]
```

### 参数

| 参数 | 描述 |
|------|------|
| `<NUMBERS...>` | 一个或多个 PR 编号（与 `--merged`/`--closed` 互斥） |

**互斥规则：**
- `<NUMBERS>` 与 `--merged`/`--closed` 互斥
- 如果同时提供，返回错误："不能同时指定 PR 编号和 --merged/--closed"
- `--merged` 和 `--closed` 可以组合使用，清理所有已合并和已关闭的 PR

### 选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `--worktree <PATH>` | 移除指定的 worktree 路径 | 无（保留 worktree） |
| `--remote` | 删除远程分支 | `true` |
| `--local` | 删除本地分支 | `true` |
| `--force` | 强制删除未合并的分支 | `false` |
| `--dry-run` | 仅显示将执行的操作，不实际删除 | `false` |
| `--yes` | 跳过交互式确认 | `false` |
| `--merged` | 清理所有已合并的 PR 分支 | `false` |
| `--closed` | 清理所有已关闭的 PR 分支 | `false` |

### 示例

```bash
# 清理单个 PR
gf pr cleanup 172

# 清理多个 PR
gf pr cleanup 172 173 174

# 清理并移除 worktree
gf pr cleanup 172 --worktree .claude/worktrees/feat-172

# 仅查看将执行的操作
gf pr cleanup 172 --dry-run

# 强制删除未合并的分支
gf pr cleanup 172 --force

# 清理所有已合并的 PR
gf pr cleanup --merged

# 跳过确认（用于脚本）
gf pr cleanup 172 --yes
```

## 架构设计

### 模块职责

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Layer (pr.rs)                    │
│  - 解析参数                                              │
│  - 调用 CleanupService                                  │
│  - 格式化输出                                            │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              CleanupService (cleanup.rs)                │
│  - 协调清理流程                                          │
│  - 调用 PrProvider 检查 PR 状态                          │
│  - 调用 GitOps 执行 git 操作                             │
│  - 执行安全检查                                          │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
┌──────────────┐        ┌──────────────────┐
│  PrProvider  │        │    GitOps        │
│  (trait)     │        │  (git_ops.rs)    │
│--------------│        │------------------│
│ - view()     │        │ - delete_branch()│
│ - list()     │        │ - detect_worktree│
│              │        │ - remove_worktree│
└──────────────┘        └──────────────────┘
```

### 文件结构

```
apps/cli/src/commands/
├── pr.rs                    # 添加 Cleanup 子命令
└── ...

crates/core/src/
├── cleanup.rs               # 新增：CleanupService
├── git_ops.rs               # 新增：Git 操作抽象
├── pr.rs                    # PrProvider trait（保持不变）
└── lib.rs                   # 导出新模块

crates/github/src/
├── cleanup.rs               # 新增：GitHub 特定清理逻辑（如需要）
└── ...

crates/gitlab/src/
├── cleanup.rs               # 新增：GitLab 特定清理逻辑（如需要）
└── ...
```

## 数据流

### 清理流程

```
用户执行: gf pr cleanup 172 --worktree .claude/worktrees/feat-172

1. CLI 解析参数
   ├─ numbers: [172]
   ├─ worktree: Some(".claude/worktrees/feat-172")
   └─ dry_run: false

2. CleanupService.cleanup(args)
   │
   ├─ for each PR number:
   │   │
   │   ├─ 步骤 1: 检查 PR 状态
   │   │   └─ provider.view(172) → PrData { state: Merged, head_branch: "feature/x" }
   │   │
   │   ├─ 步骤 2: 安全检查
   │   │   ├─ 分支是否受保护？ → 否
   │   │   ├─ 是否是当前分支？ → 否
   │   │   └─ PR 是否已合并/关闭？ → 是（已合并）
   │   │
   │   ├─ 步骤 3: 检测 worktree
   │   │   └─ git_ops::detect_worktree() → Some("/path/to/worktree")
   │   │
   │   ├─ 步骤 4: 用户确认（如果非 --yes）
   │   │   └─ 提示："删除远程分支 'feature/x' 和本地分支 'feature/x'？[y/N]"
   │   │   └─ 批量清理时，每个 PR 单独确认（除非使用 --yes）
   │   │
   │   ├─ 步骤 5: 执行清理（如果非 dry-run）
   │   │   ├─ 删除远程分支 → git push origin --delete feature/x
   │   │   ├─ 删除本地分支 → git branch -d feature/x
   │   │   ├─ 退出 worktree（如果在 worktree 中）→ cd 到主仓库
   │   │   └─ 移除 worktree（如果指定 --worktree）→ git worktree remove <path>
   │   │
   │   └─ 步骤 6: 返回结果
   │       └─ CleanupResult { pr_number: 172, remote_deleted: true, ... }
   │
   └─ 汇总所有结果 → Vec<CleanupResult>

3. CLI 输出结果（JSON 或 Toon 格式）
```

### 批量清理流程

```
用户执行: gf pr cleanup --merged

1. CLI 解析参数
   └─ merged: true

2. CleanupService.cleanup_merged(args)
   │
   ├─ 步骤 1: 获取所有已合并的 PR
   │   └─ provider.list(ListPrArgs { state: Closed }) → Vec<PrData>
   │
   ├─ 步骤 2: 过滤已合并的 PR
   │   └─ filter(|pr| pr.merged == true)
   │
   ├─ 步骤 3: 对每个 PR 执行清理流程（同上）
   │
   └─ 步骤 4: 汇总结果
       └─ 成功数、失败数、跳过数
```

## 安全检查

### 检查矩阵

| 检查项 | 行为 | 可覆盖 |
|--------|------|--------|
| **PR 状态** | 仅允许已合并或已关闭的 PR | `--force` 可覆盖（允许未合并） |
| **受保护分支** | 硬拒绝，始终不删除 | 不可覆盖 |
| **当前分支** | 拒绝删除当前检出的分支 | 不可覆盖 |
| **未合并提交** | 警告并拒绝（除非 `--force`） | `--force` 可覆盖 |
| **交互式确认** | 删除前提示确认 | `--yes` 跳过确认 |

### 受保护分支检测

**Phase 1（本地检测）：**
- 硬编码常见主分支名：`main`、`master`、`develop`、`release/*`
- 检查分支名是否匹配这些模式

**Phase 2（远程检测，可选）：**
- 调用 GitHub/GitLab API 查询分支保护状态
- 缓存结果以避免重复 API 调用

### 安全检查伪代码

```rust
fn check_safety(pr: &PrData, current_branch: &str, force: bool) -> Result<()> {
    // 1. 检查 PR 状态
    if !force && pr.state != State::Merged && pr.state != State::Closed {
        return Err("PR 尚未合并或关闭。使用 --force 强制清理。");
    }

    // 2. 检查受保护分支
    if is_protected_branch(&pr.head_branch) {
        return Err(format!("分支 '{}' 受保护，拒绝删除", pr.head_branch));
    }

    // 3. 检查当前分支
    if pr.head_branch == current_branch {
        return Err(format!("无法删除当前检出的分支 '{}'", pr.head_branch));
    }

    // 4. 检查未合并提交（如果非 --force）
    if !force && has_unmerged_commits(&pr.head_branch, &pr.base_branch) {
        return Err(format!("分支 '{}' 包含未合并的提交。使用 --force 强制删除。", pr.head_branch));
    }

    Ok(())
}

fn is_protected_branch(branch: &str) -> bool {
    matches!(branch, "main" | "master" | "develop" | "release/*")
}
```

## Worktree 处理

### 场景

**场景 1：不在 worktree 中**
```bash
$ gf pr cleanup 172
✓ 远程分支 'feature/x' 已删除
✓ 本地分支 'feature/x' 已删除
```

**场景 2：在 worktree 中，未指定 --worktree**
```bash
$ gf pr cleanup 172
✓ 远程分支 'feature/x' 已删除
✓ 本地分支 'feature/x' 已删除
✓ 已退出 worktree，返回主仓库
⚠ Worktree 目录保留：.claude/worktrees/feat-172
  使用 `git worktree remove .claude/worktrees/feat-172` 手动移除
```

**场景 3：在 worktree 中，指定 --worktree**
```bash
$ gf pr cleanup 172 --worktree .claude/worktrees/feat-172
✓ 远程分支 'feature/x' 已删除
✓ 本地分支 'feature/x' 已删除
✓ 已退出 worktree
✓ Worktree 已移除：.claude/worktrees/feat-172
```

### 自动退出逻辑

```rust
fn handle_worktree(worktree_path: Option<&str>) -> Result<()> {
    // 1. 检测是否在 worktree 中
    if !is_in_worktree()? {
        return Ok(()); // 不在 worktree 中，无需处理
    }

    // 2. 获取主仓库路径
    let main_repo = get_main_repo_path()?;

    // 3. 切换到主仓库
    std::env::set_current_dir(&main_repo)?;
    println!("✓ 已退出 worktree，返回主仓库");

    // 4. 如果指定 --worktree，移除 worktree
    if let Some(path) = worktree_path {
        remove_worktree(path)?;
        println!("✓ Worktree 已移除：{}", path);
    } else {
        println!("⚠ Worktree 目录保留：{}", get_current_worktree_path()?);
        println!("  使用 `git worktree remove <path>` 手动移除");
    }

    Ok(())
}

fn is_in_worktree() -> Result<bool> {
    // 检查 .git 是否为文件（worktree）而非目录（主仓库）
    let git_path = Path::new(".git");
    Ok(git_path.exists() && git_path.is_file())
}

fn get_main_repo_path() -> Result<PathBuf> {
    // 使用 git rev-parse --git-common-dir 获取主仓库的 .git 目录
    // 然后取其父目录
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(git_dir).parent().unwrap().to_path_buf())
}
```

## 错误处理

### 错误场景

| 错误场景 | 错误消息 | 建议操作 |
|----------|----------|----------|
| PR 不存在 | `PR #999 不存在` | 检查 PR 编号 |
| PR 未合并/关闭 | `PR #172 尚未合并或关闭` | 使用 `--force` 强制清理 |
| 分支受保护 | `分支 'main' 受保护，拒绝删除` | 无（硬拒绝） |
| 当前分支 | `无法删除当前检出的分支 'feature/x'` | 先切换到其他分支 |
| worktree 有未提交变更 | `Worktree 包含未提交变更` | 提交或暂存变更后重试 |
| 远程分支不存在 | `远程分支 'feature/x' 不存在` | 跳过远程删除，继续本地删除 |
| 本地分支不存在 | `本地分支 'feature/x' 不存在` | 跳过本地删除，继续其他操作 |

### 批量清理错误处理

- 单个 PR 清理失败不阻塞其他 PR
- 最终输出汇总：
  ```
  清理完成：
  ✓ 成功: 3
  ✗ 失败: 1 (PR #175: 分支受保护)
  ⊘ 跳过: 0
  ```

## 数据结构

### CleanupArgs

```rust
#[derive(Debug, Clone)]
pub struct CleanupArgs {
    /// PR 编号列表（与 merged/closed 互斥）
    pub numbers: Vec<u64>,
    /// 清理所有已合并的 PR
    pub merged: bool,
    /// 清理所有已关闭的 PR
    pub closed: bool,
    /// 移除指定的 worktree 路径
    pub worktree: Option<String>,
    /// 删除远程分支
    pub remote: bool,
    /// 删除本地分支
    pub local: bool,
    /// 强制删除未合并的分支
    pub force: bool,
    /// 仅显示将执行的操作
    pub dry_run: bool,
    /// 跳过交互式确认
    pub yes: bool,
}
```

### CleanupResult

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CleanupResult {
    /// PR 编号
    pub pr_number: u64,
    /// PR 标题
    pub pr_title: String,
    /// 分支名
    pub branch: String,
    /// 远程分支是否已删除
    pub remote_deleted: bool,
    /// 本地分支是否已删除
    pub local_deleted: bool,
    /// 是否退出了 worktree
    pub worktree_exited: bool,
    /// Worktree 是否已移除
    pub worktree_removed: bool,
    /// 是否为 dry-run
    pub dry_run: bool,
    /// 错误消息（如果有）
    pub error: Option<String>,
}
```

## 测试策略

### 单元测试

**CleanupService 测试：**
```rust
#[test]
fn test_should_refuse_to_delete_protected_branch() {
    // Mock PrProvider 返回 main 分支
    // 调用 cleanup
    // 断言返回错误 "分支 'main' 受保护"
}

#[test]
fn test_should_allow_cleanup_of_merged_pr() {
    // Mock PrProvider 返回已合并的 PR
    // 调用 cleanup
    // 断言成功删除分支
}

#[test]
fn test_should_require_force_for_unmerged_pr() {
    // Mock PrProvider 返回未合并的 PR
    // 调用 cleanup（无 --force）
    // 断言返回错误 "PR 尚未合并或关闭"
}

#[test]
fn test_should_skip_dry_run() {
    // 执行 cleanup --dry-run
    // 断言分支未被实际删除
}
```

**GitOps 测试：**
```rust
#[test]
fn test_should_detect_worktree() {
    // 创建 test worktree
    // 调用 detect_worktree()
    // 断言返回 Some(path)
}

#[test]
fn test_should_auto_exit_worktree() {
    // 创建 test worktree
    // 在 worktree 中执行 cleanup
    // 断言当前目录已切换到主仓库
}

#[test]
fn test_should_remove_worktree_with_flag() {
    // 创建 test worktree
    // 执行 cleanup --worktree <path>
    // 断言 worktree 已被移除
}
```

### 集成测试

```rust
#[test]
fn test_cleanup_merged_pr_end_to_end() {
    // 1. 创建测试分支
    // 2. 创建测试 PR（mock）
    // 3. 模拟 PR 已合并
    // 4. 执行 cleanup
    // 5. 验证分支已删除
}

#[test]
fn test_batch_cleanup_merged_prs() {
    // 1. 创建多个测试分支和 PR
    // 2. 模拟所有 PR 已合并
    // 3. 执行 cleanup --merged
    // 4. 验证所有分支已删除
}
```

## 实现计划

### Phase 1（MVP）

1. **核心功能**
   - [ ] 实现 `CleanupArgs` 和 `CleanupResult` 数据结构
   - [ ] 实现 `GitOps` 模块（分支删除、worktree 检测）
   - [ ] 实现 `CleanupService`（协调清理流程）
   - [ ] 添加 `PrCommand::Cleanup` 到 CLI
   - [ ] 实现安全检查（受保护分支、当前分支、PR 状态）

2. **Worktree 处理**
   - [ ] 实现 worktree 检测
   - [ ] 实现自动退出 worktree
   - [ ] 实现 `--worktree` 选项移除 worktree

3. **用户交互**
   - [ ] 实现交互式确认
   - [ ] 实现 `--yes` 跳过确认
   - [ ] 实现 `--dry-run` 模式

4. **测试**
   - [ ] 单元测试（CleanupService、GitOps）
   - [ ] 集成测试（端到端清理流程）

### Phase 2（增强）

1. **批量清理**
   - [ ] 实现 `--merged` 选项
   - [ ] 实现 `--closed` 选项
   - [ ] 支持多个 PR 编号

2. **远程保护检测**
   - [ ] 调用 GitHub/GitLab API 查询分支保护状态
   - [ ] 缓存保护状态

3. **增强错误处理**
   - [ ] 批量清理时的错误汇总
   - [ ] 更详细的错误消息和建议

## 验收标准

- [ ] `gf pr cleanup <NUMBER>` 命令可用
- [ ] 检测 PR 是否已合并/关闭
- [ ] 安全处理 worktree 场景
- [ ] 删除远程分支（带确认）
- [ ] 删除本地分支（带安全检查）
- [ ] `--worktree` 选项移除 worktree
- [ ] `--dry-run` 显示将执行的操作
- [ ] `--force` 选项用于未合并的分支
- [ ] `--merged` 和 `--closed` 批量清理
- [ ] `gf pr --help` 中包含文档
- [ ] worktree 和非 worktree 场景的测试

## 参考

- Issue #174: https://github.com/byx-darwin/gitflow-cli/issues/174
- Issue #173: CLI 要求标准化
- PR #172: 发现问题来源
