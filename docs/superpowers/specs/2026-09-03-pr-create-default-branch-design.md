# pr create 默认分支检测设计

**Issue**: #305
**分类**: bounded
**日期**: 2026-09-03

## 问题

`apps/cli/src/commands/pr.rs:251`：

```rust
let resolved_base = base.unwrap_or_else(|| "main".to_string());
```

未显式传 `--base` 时硬编码为 `"main"`，在默认分支非 `main` 的仓库（如 `dev`）上会
生成目标错误的 PR/MR。代码库内未发现任何查询仓库真实默认分支的逻辑
（`default_branch` / `symbolic-ref` / `defaultBranch` 均无命中）。

## 方案

在 `PrProvider` trait 新增一个方法，未传 `--base` 时优先查询平台真实默认分支，
查询失败（含平台能力缺失）时回退 `"main"`。

### 1. `crates/core/src/pr.rs`

`PrProvider` trait 新增：

```rust
/// 查询仓库配置的默认分支（如 `main`、`dev`）。
///
/// # Errors
///
/// 当平台 API 调用失败或平台不支持该查询时返回错误。
async fn default_branch(&self) -> Result<String>;
```

### 2. `crates/github/src/pr.rs` — `GitHubPrProvider`

```
gh repo view --repo <repo> --json defaultBranchRef
```

解析 JSON，取 `defaultBranchRef.name`。

### 3. `crates/gitlab/src/mr.rs` — `GitLabMrProvider`

```
glab repo view --repo <repo_target> --output json
```

解析 JSON 顶层 `default_branch` 字段。

### 4. `crates/gitcode/src/pr.rs` — `GitCodePrProvider`

`gc` CLI 无已知的 `repo view --json` 能力（先例：`pr merge --auto` 对 GitCode
直接返回 `CoreError::Platform` 且不发起 CLI 调用，见
`docs/references/gf-pr-params.md:56`）。`default_branch()` 同样直接返回
`CoreError::Platform("...")`，不做实际 CLI 调用。

### 5. `apps/cli/src/commands/pr.rs` — 调用点（约 line 251）

```rust
let resolved_base = match base {
    Some(b) => b,
    None => provider
        .default_branch()
        .await
        .unwrap_or_else(|e| {
            tracing::debug!(error = %e, "default_branch query failed, falling back to \"main\"");
            "main".to_string()
        }),
};
```

显式传 `--base` 时行为完全不变，跳过查询。

## 测试

- 三个 provider 各补充 `default_branch()` 成功 / 失败用例（`test_should_*` 命名）。
- GitCode 补充一条断言：直接返回 `CoreError::Platform`，且 `runner.recorded_calls()`
  为空（未发起任何 CLI 调用）。
- `apps/cli/src/commands/pr.rs` 补充集成测试：
  - 未传 `--base` 且 provider 查询成功 → 使用查询结果。
  - 未传 `--base` 且 provider 查询失败 → fallback `"main"`。
  - 显式传 `--base` → 直接使用，不触发查询。

## 验收标准（同 Issue #305）

- [ ] `gf pr create` 未传 `--base` 时命中仓库真实默认分支
- [ ] 在默认分支非 `main` 的仓库上验证通过
- [ ] 显式传 `--base` 行为不变
