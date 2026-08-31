# Design: `resolve_body()` --body-file 补齐 SafePath 校验

- **Issue:** #271
- **Date:** 2026-08-31
- **Classification:** Bounded（既有代码的单函数修复，非新增子系统）

## 背景

`apps/cli/src/commands/issue.rs::resolve_body(body, body_file)` 在处理 `--body-file`
时直接调用 `std::fs::read_to_string(&path)`，未经过 `gitflow_core::SafePath` 校验。
该 helper 被 `gf issue create` / `gf issue comment`，以及经 `resolve_comment_body`
间接被 `gf issue edit` 共用。`CLAUDE.md` 要求所有外部输入的文件路径参数必须先经
`SafePath` 校验（拒绝 `..`、绝对路径歧义、NUL 字节等）。

`apps/cli/src/commands/release.rs::resolve_body()`（第 281-295 行）已存在完全同名同
签名的函数，并已正确应用 `SafePath::new_allow_absolute()` 校验模式——本设计直接复用
该既有模式，不引入新抽象。

## 改动范围

仅 `apps/cli/src/commands/issue.rs`。

## 设计

在 `resolve_body()` 的 `if let Some(path) = body_file` 分支中插入校验：

```rust
if let Some(path) = body_file {
    let safe = gitflow_core::SafePath::new_allow_absolute(&path)
        .map_err(|e| miette::miette!("无效的 --body-file 参数: {e}"))?;
    let content = std::fs::read_to_string(safe.as_path())
        .map_err(|e| miette::miette!("Failed to read body file '{path}': {e}"))?;
    return Ok(Some(content));
}
```

`SafePath::new_allow_absolute`（而非 `SafePath::new`）与 `release.rs` 保持一致，
允许绝对路径（CLI 场景下用户常传绝对路径），同时仍拒绝 `..`、NUL 字节等越权输入。

`create`（issue.rs:201）、`comment`（issue.rs:221）、`edit`（经 `resolve_comment_body`
→ `resolve_body`，issue.rs:374）三个调用方无需改动，行为随 `resolve_body` 内部实现
自动继承。

## 错误处理

- 校验失败（`..`、NUL 字节等）→ 返回 `miette::miette!("无效的 --body-file 参数: {e}")`，
  与 `release.rs` 用词一致。
- 校验通过但文件不存在/不可读 → 沿用现有 IO 错误信息，行为不变。

## 测试计划（TDD）

- RED：新增测试，`resolve_body(None, Some(".."))` 与含 NUL 字节路径均应返回 `Err`。
- GREEN：应用上述改动后测试通过。
- 回归：既有测试（`test_should_resolve_body_from_file`、
  `test_should_resolve_body_with_body_only` 等）保持通过，证明合法路径行为不变。

## 验证

- `cargo test -p gitflow-cli`（`issue.rs` 测试模块）
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
