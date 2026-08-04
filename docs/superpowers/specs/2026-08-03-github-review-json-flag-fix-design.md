# GitHub Review `--json` 标志修复设计文档

**日期**: 2026-08-03
**Issue**: #119
**优先级**: High
**类型**: Bug Fix

## 问题描述

`gf review` 的全部方法（`approve` / `comment` / `request-changes` / `submit_review`）在 GitHub 平台失败，报错：`gh: unknown flag: --json`。

### 根因

`crates/github/src/review.rs` 第 60 / 85 / 118 / 147 行向 `gh pr review` 传递 `--json REVIEW_FIELDS`，但该命令**完全不支持** `--json` 标志（已通过 `gh pr review --help` 确认）。

### 影响

- 4 个 review 子命令在 GitHub 平台完全不可用
- 与 #60（`issue comment` / `pr comment` / `pr create` 的同类问题）同族，但当时的修复遗漏了 `pr review`
- 在 PR #118 审查现场复现

### 复现步骤

```bash
gf review approve 118 --body "test"
# × Failed to approve PR #118: platform error: gh: unknown flag: --json
```

## 修复方案

### 方案选择

采用 **方案 A：使用 `gh api` 获取最新 review**，与 `issue.rs` 的修复模式完全一致。

**理由：**
1. ✅ 与现有修复模式一致（PR #60 已验证）
2. ✅ 使用稳定的 REST API，不依赖 `gh` CLI 的 JSON 输出
3. ✅ 可获取完整的 review 数据（ID、状态、时间戳等）
4. ✅ 代码可维护性高

**被否决的方案：**
- **方案 B（解析人类可读输出）**：`gh pr review` 输出过于简洁（通常只是 "Done"），不包含 review ID、时间戳等详细信息，且解析不稳定
- **方案 C（混合方案）**：与方案 A 本质相同，只是强调两步分离

## 技术设计

### 实现模式

遵循 `issue.rs` 中 `comment` 方法的模式：

```rust
// 1. 执行 gh pr review（不期望 JSON 输出）
let output = self.runner.run("gh", &["pr", "review", ...]).await?;

// 2. 使用 gh api 获取最新 review
let api_path = format!("repos/{repo}/pulls/{number}/reviews?per_page=1");
let api_output = self.runner.run("gh", &["api", &api_path]).await?;

// 3. 解析 API 响应并转换为 ReviewData
let reviews: Vec<GitHubReviewApiResponse> = serde_json::from_slice(&api_output.stdout)?;
Ok(reviews.into_iter().next().ok_or(...)?.into())
```

### 新增结构体

```rust
/// GitHub API Review 响应结构。
///
/// 用于解析 `gh api repos/{owner}/{repo}/pulls/{number}/reviews` 的返回数据。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubReviewApiResponse {
    pub id: u64,
    pub state: String,  // APPROVED, CHANGES_REQUESTED, COMMENTED, etc.
    #[serde(default)]
    pub body: Option<String>,
    pub user: GitHubUser,
    pub submitted_at: String,
}

impl From<GitHubReviewApiResponse> for ReviewData {
    fn from(api: GitHubReviewApiResponse) -> Self {
        Self {
            id: api.id,
            state: api.state.parse().unwrap_or(ReviewState::Commented),
            body: api.body,
            author: UserSummary {
                login: api.user.login,
                id: api.user.id,
            },
            submitted_at: api.submitted_at.parse().unwrap_or_else(|_| Utc::now()),
        }
    }
}
```

### 修改的方法

所有 4 个方法都遵循相同模式：

1. **`comment`** - 发表评论
   - 移除 `--json` 标志
   - 添加 `gh api` 调用获取最新 review

2. **`approve`** - 批准 PR
   - 移除 `--json` 标志
   - 添加 `gh api` 调用获取最新 review

3. **`request_changes`** - 要求修改
   - 移除 `--json` 标志
   - 添加 `gh api` 调用获取最新 review

4. **`submit_review`** - 提交审查（通用方法）
   - 移除 `--json` 标志
   - 添加 `gh api` 调用获取最新 review

### 错误处理

**特殊场景：GitHub 不允许批准自己的 PR**

当用户尝试批准自己的 PR 时，GitHub API 返回错误：
```
Review Can not approve your own pull request
```

需要在错误处理中识别这种情况并提供清晰的错误信息：

```rust
if !output.status.success() {
    let gh_err = parse_gh_error(&output.stderr);
    // 检测 "approve your own pull request" 错误
    if gh_err.user_message.contains("approve your own pull request") {
        return Err(CoreError::Platform(
            "GitHub 不允许批准自己的 PR。可以请求其他维护者审查。".to_string()
        ));
    }
    return Err(gh_err.into());
}
```

### 测试覆盖

#### 单元测试

1. **成功路径测试**
   - 验证 `GitHubReviewApiResponse` 解析
   - 验证转换为 `ReviewData` 的正确性
   - 测试所有 state 类型的映射（APPROVED, CHANGES_REQUESTED, COMMENTED）

2. **错误路径测试**
   - 验证 "不能批准自己的 PR" 错误处理
   - 验证 API 调用失败的错误处理
   - 验证空响应的错误处理

3. **集成测试**
   - 使用 mock runner 模拟 `gh` CLI 输出
   - 验证完整的命令执行流程

#### 测试用例示例

```rust
#[test]
fn test_should_convert_github_review_api_response_to_review_data() {
    let api_response = GitHubReviewApiResponse {
        id: 12345,
        state: "APPROVED".to_string(),
        body: Some("LGTM".to_string()),
        user: GitHubUser {
            login: "octocat".to_string(),
            id: 1,
        },
        submitted_at: "2026-08-03T10:00:00Z".to_string(),
    };

    let review_data: ReviewData = api_response.into();

    assert_eq!(review_data.id, 12345);
    assert_eq!(review_data.state, ReviewState::Approved);
    assert_eq!(review_data.body, Some("LGTM".to_string()));
    assert_eq!(review_data.author.login, "octocat");
}

#[test]
fn test_should_handle_own_pr_approval_error() {
    // 模拟 gh pr review 返回错误
    let mock_runner = MockCommandRunner::new()
        .with_stderr("Review Can not approve your own pull request");

    let provider = GitHubReviewProvider::with_runner("owner/repo", mock_runner);

    let result = provider.approve(123, Some("LGTM")).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不允许批准自己的 PR"));
}
```

## 文件变更清单

### 修改的文件

- `crates/github/src/review.rs`
  - 移除 `REVIEW_FIELDS` 常量
  - 移除所有 `--json` 标志
  - 添加 `GitHubReviewApiResponse` 结构体
  - 添加 `From<GitHubReviewApiResponse> for ReviewData` 实现
  - 修改 4 个方法：`comment`、`approve`、`request_changes`、`submit_review`
  - 添加单元测试

### 不受影响的文件

- `crates/core/src/review.rs` - `ReviewData` 结构体不变
- `apps/cli/src/commands/review.rs` - CLI 命令层不变
- 其他平台实现（GitLab、GitCode）- 不受影响

## 退出标准

- [ ] 所有 4 个 review 方法在 GitHub 平台可正常工作
- [ ] 单元测试覆盖成功路径和错误路径
- [ ] `cargo test` 全部通过
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` 无警告
- [ ] 手动测试验证（在真实 PR 上执行 review 操作）

## 风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| GitHub API 响应格式变化 | 使用稳定的 REST API，字段与 `ReviewData` 对齐；添加 `#[serde(default)]` 处理可选字段 |
| `gh` CLI 版本差异 | 不依赖 `--json` 输出，只依赖 `gh api` 的 REST 响应 |
| "不能批准自己的 PR" 错误 | 在错误处理中识别并提供清晰的中文错误信息 |
| 测试覆盖不足 | 添加单元测试 + 集成测试，覆盖所有错误路径 |

## 参考资料

- PR #60 (commit b2b2c5c) - 同类问题的修复模式
- GitHub REST API: `GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews`
- `gh pr review --help` - 确认不支持 `--json` 标志
- `crates/github/src/issue.rs` - 参考实现模式

## 实现计划

1. **阶段 1：重构 `review.rs`**
   - 移除 `REVIEW_FIELDS` 常量
   - 添加 `GitHubReviewApiResponse` 结构体
   - 添加 `From` 转换实现

2. **阶段 2：修改 4 个方法**
   - 逐个修改 `comment`、`approve`、`request_changes`、`submit_review`
   - 每个方法都添加 `gh api` 调用
   - 添加错误处理（特别是 "不能批准自己的 PR"）

3. **阶段 3：添加测试**
   - 添加单元测试（成功路径 + 错误路径）
   - 使用 mock runner 模拟 `gh` CLI 输出
   - 验证 API 响应解析和转换

4. **阶段 4：验证**
   - 运行 `cargo test` 确保所有测试通过
   - 运行 `cargo clippy` 确保无警告
   - 手动测试验证（在真实 PR 上执行 review 操作）

---

**文档状态**: 待用户审查
**下一步**: 用户审查通过后，调用 `writing-plans` 生成实施计划
