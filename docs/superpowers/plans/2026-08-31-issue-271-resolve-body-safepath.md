# Issue #271: resolve_body() SafePath 校验 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `apps/cli/src/commands/issue.rs::resolve_body()` 在处理 `--body-file` 时对路径先做 `SafePath` 校验，再读取文件，覆盖 `create`/`comment`/`edit` 三个调用方。

**Architecture:** 单函数改动。复用 `apps/cli/src/commands/release.rs::resolve_body()`（同名同签名函数，第 281-295 行）已验证过的模式：`gitflow_core::SafePath::new_allow_absolute(&path)` 校验，通过后用 `safe.as_path()` 替代原始字符串路径传给 `std::fs::read_to_string`。

**Tech Stack:** Rust 2024, `gitflow_core::SafePath`（crate 名 `core`，导入路径 `gitflow_core`），`miette` 错误处理。

**Spec:** `docs/superpowers/specs/2026-08-31-issue-resolve-body-safepath-design.md`

## Global Constraints

- 仅改动 `apps/cli/src/commands/issue.rs`；不改动 `SafePath` 本身或 `release.rs`。
- 使用 `SafePath::new_allow_absolute`（非 `SafePath::new`），与 `release.rs` 保持一致，因为 CLI 用户常传绝对路径。
- 错误信息前缀统一为 `"无效的 --body-file 参数: {e}"`，与 `release.rs` 用词一致。
- 对合法路径（相对/绝对，非 `..`/非 NUL 字节）的既有测试行为必须保持不变。

---

### Task 1: 为 `resolve_body()` 补齐 SafePath 校验

**Files:**
- Modify: `apps/cli/src/commands/issue.rs:352-364`（`resolve_body` 函数体）
- Test: `apps/cli/src/commands/issue.rs`（`#[cfg(test)] mod tests`，约第 405 行起）

**Interfaces:**
- Consumes: `gitflow_core::SafePath::new_allow_absolute(path: impl AsRef<Path>) -> gitflow_core::Result<SafePath>`；`SafePath::as_path(&self) -> &Path`
- Produces: `resolve_body(body: Option<String>, body_file: Option<String>) -> miette::Result<Option<String>>`（签名不变，供 `create`/`comment`/`resolve_comment_body` 调用方继续使用）

- [ ] **Step 1: 写失败测试 — 路径穿越 `..` 应被拒绝**

在 `apps/cli/src/commands/issue.rs` 的 `#[cfg(test)] mod tests` 块内，紧接 `test_should_error_on_missing_body_file` 之后新增：

```rust
    #[test]
    fn test_should_reject_body_file_with_path_traversal() {
        let result = resolve_body(None, Some("../secret.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("无效的 --body-file 参数"));
    }

    #[test]
    fn test_should_reject_body_file_with_nul_byte() {
        let result = resolve_body(None, Some("foo\0bar.md".into()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("无效的 --body-file 参数"));
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-cli test_should_reject_body_file --lib`
Expected: 两个新测试均 FAIL（此时 `resolve_body` 尚未做路径校验，`..` 与 NUL 字节路径会直接进入 `std::fs::read_to_string`，返回的错误是文件不存在类错误而非 `"无效的 --body-file 参数"`，`err.contains(...)` 断言失败）。

- [ ] **Step 3: 实现最小改动**

将 `apps/cli/src/commands/issue.rs` 第 352-364 行的 `resolve_body` 函数体替换为：

```rust
fn resolve_body(body: Option<String>, body_file: Option<String>) -> miette::Result<Option<String>> {
    if body.is_some() && body_file.is_some() {
        return Err(miette::miette!(
            "Cannot specify both --body and --body-file"
        ));
    }
    if let Some(path) = body_file {
        let safe = gitflow_core::SafePath::new_allow_absolute(&path)
            .map_err(|e| miette::miette!("无效的 --body-file 参数: {e}"))?;
        let content = std::fs::read_to_string(safe.as_path())
            .map_err(|e| miette::miette!("Failed to read body file '{path}': {e}"))?;
        return Ok(Some(content));
    }
    Ok(body)
}
```

保留原函数上方的 `#[allow(clippy::disallowed_methods, reason = "...")]` 与文档注释不变。

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-cli resolve_body --lib`
Expected: 全部通过，包括新增的两个测试与既有的 `test_should_resolve_body_with_body_only`、`test_should_resolve_body_with_none`、`test_should_resolve_body_from_file`、`test_should_error_on_missing_body_file`、`test_should_reject_both_body_and_body_file`（既有测试均使用合法相对/绝对路径或提前在 `body.is_some() && body_file.is_some()` 分支返回，行为不受影响）。

- [ ] **Step 5: 全量测试 + lint**

Run: `cargo test -p gitflow-cli --lib`
Expected: 全部通过。

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无新增警告。

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/issue.rs
git commit -m "fix(cli): validate --body-file with SafePath in resolve_body

Closes #271"
```
