# TOON Output Format Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add TOON (Token-Oriented Object Notation) output format to gitflow-cli, reducing token consumption by 15-40% for LLM consumers.

**Architecture:** A new `crates/core/src/toon.rs` module provides structure analysis (`analyze`), strategy selection (`select_strategy`), and TOON encoding (`encode`) via the `toon-format` crate. The CLI's `OutputFormat` enum gains `Toon` and `Auto` variants; `print_output` dispatches accordingly. Skills documentation is updated to default to `--output auto`.

**Tech Stack:** Rust 2024, `toon-format` 0.5, `serde_json`, `clap`, `miette`, `assert_cmd`

## Global Constraints

- Use Rust 2024 edition with the pinned toolchain in `rust-toolchain.toml`
- Follow TDD: RED → GREEN → REFACTOR for every task
- All public items require documentation
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` must pass
- `cargo +nightly fmt` must pass
- Skills default to `--output auto`

---

### Task 1: Add `toon-format` Dependency

**Files:**
- Modify: `Cargo.toml:16` (workspace dependencies)
- Modify: `crates/core/Cargo.toml:15` (core dependencies)

**Interfaces:**
- Produces: `toon-format` crate available for use in `crates/core`

- [ ] **Step 1: Add workspace dependency**

Add to `Cargo.toml` in `[workspace.dependencies]` section after line 23:

```toml
toon-format = "0.5"
```

- [ ] **Step 2: Add core crate dependency**

Add to `crates/core/Cargo.toml` in `[dependencies]` section after line 21:

```toml
toon-format.workspace = true
```

- [ ] **Step 3: Verify build**

Run: `cargo build -p gitflow-cli-core`
Expected: BUILD SUCCESS

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/core/Cargo.toml
git commit -m "feat: add toon-format dependency to workspace and core crate"
```

---

### Task 2: Implement Core Types with Tests

**Files:**
- Create: `crates/core/src/toon.rs`
- Modify: `crates/core/src/lib.rs:31` (add `pub mod toon;`)

**Interfaces:**
- Produces: `JsonShape`, `TopLevel`, `ToonStrategy` types
- Produces: `analyze(&Value) -> JsonShape`
- Produces: `select_strategy(&JsonShape) -> ToonStrategy`
- Produces: `encode(&Value) -> Result<String, ToonError>`
- Consumes: `serde_json::Value`, `toon_format::encode_default`

- [ ] **Step 1: Write failing tests for `analyze()`**

Create `crates/core/src/toon.rs`:

```rust
//! TOON (Token-Oriented Object Notation) output support.
//!
//! Provides structure analysis, strategy selection, and TOON encoding
//! for token-efficient output. Uses the `toon-format` crate for encoding.

use serde_json::Value;

/// The top-level type of a JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopLevel {
    /// A JSON object (`{...}`).
    Object,
    /// A JSON array (`[...]`).
    Array,
    /// A JSON scalar (string, number, bool, null).
    Scalar,
}

/// Structural characteristics of a JSON value.
///
/// Produced by [`analyze`] and consumed by [`select_strategy`] to decide
/// whether TOON or JSON encoding is more token-efficient.
#[derive(Debug, Clone)]
pub struct JsonShape {
    /// The top-level JSON type.
    pub top_level: TopLevel,
    /// Number of items if top-level is an array, 0 otherwise.
    pub item_count: usize,
    /// `true` if top-level is an array and all items are objects with identical key sets.
    pub is_uniform_array: bool,
    /// Maximum nesting depth (0 for flat values).
    pub max_depth: usize,
    /// Total character count of the pretty-printed JSON representation.
    pub char_count: usize,
}

/// Strategy selected by [`select_strategy`] for encoding output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToonStrategy {
    /// Encode as TOON (uniform arrays or deeply nested structures).
    Toon,
    /// Keep as JSON (small data or structures not suited for TOON).
    Json,
}

/// Error type for TOON encoding failures.
#[derive(Debug, Clone)]
pub struct ToonError {
    /// Human-readable error message.
    pub message: String,
}

impl std::fmt::Display for ToonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOON encoding error: {}", self.message)
    }
}

impl std::error::Error for ToonError {}

/// Analyze the structure of a JSON value.
///
/// Performs a single-pass O(n) traversal to collect structural metrics
/// used by [`select_strategy`].
pub fn analyze(value: &Value) -> JsonShape {
    let char_count = serde_json::to_string_pretty(value)
        .map_or(0, |s| s.len());

    let top_level = match value {
        Value::Object(_) => TopLevel::Object,
        Value::Array(_) => TopLevel::Array,
        _ => TopLevel::Scalar,
    };

    let (item_count, is_uniform_array) = match value {
        Value::Array(arr) => {
            let count = arr.len();
            let uniform = count > 0
                && arr.iter().all(|item| {
                    item.as_object().is_some_and(|obj| {
                        let keys: Vec<&String> = {
                            let mut ks: Vec<_> = obj.keys().collect();
                            ks.sort_unstable();
                            ks
                        };
                        // Compare with first item's keys
                        arr[0].as_object().is_some_and(|first| {
                            let mut first_keys: Vec<_> = first.keys().collect();
                            first_keys.sort_unstable();
                            keys == first_keys
                        })
                    })
                });
            (count, uniform)
        }
        _ => (0, false),
    };

    let max_depth = compute_depth(value, 0);

    JsonShape {
        top_level,
        item_count,
        is_uniform_array,
        max_depth,
        char_count,
    }
}

/// Recursively compute the maximum nesting depth.
fn compute_depth(value: &Value, current: usize) -> usize {
    match value {
        Value::Object(obj) => {
            obj.values()
                .map(|v| compute_depth(v, current + 1))
                .max()
                .unwrap_or(current)
        }
        Value::Array(arr) => {
            arr.iter()
                .map(|v| compute_depth(v, current + 1))
                .max()
                .unwrap_or(current)
        }
        _ => current,
    }
}

/// Select the optimal encoding strategy based on structural analysis.
///
/// Rules (applied in order):
/// 1. `char_count < 200` → [`ToonStrategy::Json`] (not worth converting)
/// 2. Uniform array with `item_count >= 5` → [`ToonStrategy::Toon`]
/// 3. `max_depth > 3` → [`ToonStrategy::Toon`]
/// 4. Otherwise → [`ToonStrategy::Json`]
pub fn select_strategy(shape: &JsonShape) -> ToonStrategy {
    if shape.char_count < 200 {
        return ToonStrategy::Json;
    }
    if shape.top_level == TopLevel::Array && shape.is_uniform_array && shape.item_count >= 5 {
        return ToonStrategy::Toon;
    }
    if shape.max_depth > 3 {
        return ToonStrategy::Toon;
    }
    ToonStrategy::Json
}

/// Encode a JSON value as TOON format.
///
/// # Errors
///
/// Returns [`ToonError`] if the `toon-format` crate fails to encode.
pub fn encode(value: &Value) -> Result<String, ToonError> {
    toon_format::encode_default(value).map_err(|e| ToonError {
        message: e.to_string(),
    })
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Tests legitimately unwrap known-good fixture data"
)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── analyze tests ──────────────────────────────────────────────

    #[test]
    fn test_analyze_uniform_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"},
            {"id": 3, "name": "c"},
            {"id": 4, "name": "d"},
            {"id": 5, "name": "e"}
        ]);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Array);
        assert_eq!(shape.item_count, 5);
        assert!(shape.is_uniform_array);
    }

    #[test]
    fn test_analyze_mixed_array() {
        let value = json!([
            {"a": 1, "b": 2},
            {"a": 3, "c": 4}
        ]);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Array);
        assert_eq!(shape.item_count, 2);
        assert!(!shape.is_uniform_array);
    }

    #[test]
    fn test_analyze_deep_nesting() {
        let value = json!({"a": {"b": {"c": {"d": {"e": 1}}}}});
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Object);
        assert_eq!(shape.max_depth, 5);
    }

    #[test]
    fn test_analyze_small_data() {
        let value = json!({"x": 1});
        let shape = analyze(&value);
        assert!(shape.char_count < 200);
    }

    #[test]
    fn test_analyze_scalar() {
        let value = json!(42);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Scalar);
        assert_eq!(shape.item_count, 0);
        assert!(!shape.is_uniform_array);
    }

    // ── select_strategy tests ──────────────────────────────────────

    #[test]
    fn test_strategy_small_data_returns_json() {
        let shape = JsonShape {
            top_level: TopLevel::Array,
            item_count: 10,
            is_uniform_array: true,
            max_depth: 1,
            char_count: 100,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Json);
    }

    #[test]
    fn test_strategy_uniform_array_returns_toon() {
        let shape = JsonShape {
            top_level: TopLevel::Array,
            item_count: 10,
            is_uniform_array: true,
            max_depth: 1,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Toon);
    }

    #[test]
    fn test_strategy_deep_nesting_returns_toon() {
        let shape = JsonShape {
            top_level: TopLevel::Object,
            item_count: 0,
            is_uniform_array: false,
            max_depth: 5,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Toon);
    }

    #[test]
    fn test_strategy_default_returns_json() {
        let shape = JsonShape {
            top_level: TopLevel::Object,
            item_count: 0,
            is_uniform_array: false,
            max_depth: 2,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Json);
    }

    // ── encode tests ───────────────────────────────────────────────

    #[test]
    fn test_toon_encode_simple_object() {
        let value = json!({"name": "Alice", "age": 30});
        let result = encode(&value);
        assert!(result.is_ok());
        let encoded = result.unwrap();
        assert!(encoded.contains("name"));
        assert!(encoded.contains("Alice"));
    }

    #[test]
    fn test_toon_encode_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"},
            {"id": 3, "name": "c"},
            {"id": 4, "name": "d"},
            {"id": 5, "name": "e"}
        ]);
        let result = encode(&value);
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-cli-core --lib toon`
Expected: FAIL — module `toon` not yet declared in `lib.rs`

- [ ] **Step 3: Register module in lib.rs**

Add `pub mod toon;` to `crates/core/src/lib.rs` after `pub mod types;` (line 37).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli-core --lib toon`
Expected: All 12 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/toon.rs crates/core/src/lib.rs
git commit -m "feat(core): add TOON module with analyze, select_strategy, encode"
```

---

### Task 3: Extend CLI OutputFormat Enum

**Files:**
- Modify: `apps/cli/src/main.rs:396-403`

**Interfaces:**
- Consumes: `clap::ValueEnum` derive
- Produces: `OutputFormat::Toon`, `OutputFormat::Auto` variants

- [ ] **Step 1: Write failing integration test**

Create `apps/cli/tests/toon_output_test.rs`:

```rust
//! TOON output format integration tests.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Integration tests unwrap known-good binary handles"
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_should_accept_output_toon_flag() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("toon"));
}

#[test]
fn test_should_accept_output_auto_flag() {
    let mut cmd = Command::cargo_bin("gitflow-cli").expect("binary exists");
    cmd.args(["--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("auto"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli --test toon_output_test`
Expected: FAIL — `toon` and `auto` not in help output

- [ ] **Step 3: Add Toon and Auto variants to OutputFormat**

In `apps/cli/src/main.rs`, replace the `OutputFormat` enum (lines 396-403):

```rust
/// Output format for CLI command results.
///
/// Controls how command output is rendered to the terminal.
#[derive(Debug, Default, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// Structured JSON output (default; for machine consumption by skills).
    #[default]
    Json,
    /// Human-readable plain text output.
    Text,
    /// TOON (Token-Oriented Object Notation) — compact format for LLM consumption.
    Toon,
    /// Automatically select the best format based on data structure analysis.
    Auto,
}
```

Also update the `--output` arg help text (line 414):

```rust
    /// Output format (json, text, toon, or auto).
    #[arg(long, global = true, default_value = "json")]
    output: OutputFormat,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-cli --test toon_output_test`
Expected: Both tests PASS

- [ ] **Step 5: Commit**

```bash
git add apps/cli/src/main.rs apps/cli/tests/toon_output_test.rs
git commit -m "feat(cli): add Toon and Auto variants to OutputFormat enum"
```

---

### Task 4: Extend print_output with Toon/Auto Branches

**Files:**
- Modify: `apps/cli/src/commands/output.rs`

**Interfaces:**
- Consumes: `OutputFormat::Toon`, `OutputFormat::Auto` from `main.rs`
- Consumes: `gitflow_cli_core::toon::{analyze, select_strategy, encode, ToonStrategy}`
- Produces: `print_toon()` and `print_auto()` functions

- [ ] **Step 1: Write failing tests for print_toon and print_auto**

Add to `apps/cli/src/commands/output.rs` in the `tests` module (after existing tests, around line 298):

```rust
    use crate::OutputFormat;
    use gitflow_cli_core::toon::ToonStrategy;

    #[test]
    fn test_should_output_toon_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Toon);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_auto_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Auto);
        assert!(result.is_ok());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli --lib commands::output`
Expected: FAIL — `Toon`/`Auto` arms missing in `print_output`

- [ ] **Step 3: Implement print_toon and print_auto**

Replace the contents of `apps/cli/src/commands/output.rs` with:

```rust
//! 共享输出格式化模块。
//!
//! 提供统一的 [`print_output`] 函数，支持 JSON（默认）、Text、TOON 和 Auto 格式。
//! - JSON: 美化 JSON（默认，机器消费）
//! - Text: 人类友好的表格/键值对格式
//! - TOON: Token-Oriented Object Notation，紧凑格式（LLM 消费）
//! - Auto: 根据数据结构自动选择 JSON 或 TOON

use crate::OutputFormat;

/// 将可序列化的值按指定格式打印到 stdout。
///
/// # Errors
///
/// 返回错误当：
/// - JSON 序列化失败。
/// - Text/TOON 格式化时内部转换失败。
pub fn print_output<T: serde::Serialize>(value: &T, format: &OutputFormat) -> miette::Result<()> {
    match format {
        OutputFormat::Json => print_json(value),
        OutputFormat::Text => print_text(value),
        OutputFormat::Toon => print_toon(value),
        OutputFormat::Auto => print_auto(value),
    }
}

/// JSON 格式输出。
fn print_json<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| miette::miette!("Failed to serialize output to JSON: {e}"))?;
    println!("{json}");
    Ok(())
}

/// TOON 格式输出。
fn print_toon<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for TOON output: {e}"))?;
    let toon = gitflow_cli_core::toon::encode(&json_value)
        .map_err(|e| miette::miette!("Failed to encode TOON output: {e}"))?;
    println!("{toon}");
    Ok(())
}

/// 自动选择格式输出。
///
/// 根据数据结构分析选择 JSON 或 TOON：
/// - 小数据 (< 200 字符) → JSON
/// - 均匀数组 (≥5 元素) → TOON
/// - 深度嵌套 (> 3 层) → TOON
/// - 其他 → JSON
fn print_auto<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let json_value = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for auto output: {e}"))?;
    let shape = gitflow_cli_core::toon::analyze(&json_value);
    let strategy = gitflow_cli_core::toon::select_strategy(&shape);

    match strategy {
        gitflow_cli_core::toon::ToonStrategy::Toon => {
            let toon = gitflow_cli_core::toon::encode(&json_value)
                .map_err(|e| miette::miette!("Failed to encode TOON output: {e}"))?;
            println!("{toon}");
        }
        gitflow_cli_core::toon::ToonStrategy::Json => {
            let json = serde_json::to_string_pretty(&json_value)
                .map_err(|e| miette::miette!("Failed to serialize output to JSON: {e}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Text 格式输出。先将值序列化为 [`serde_json::Value`]，再根据结构选择格式化策略。
fn print_text<T: serde::Serialize>(value: &T) -> miette::Result<()> {
    let v = serde_json::to_value(value)
        .map_err(|e| miette::miette!("Failed to serialize value for text output: {e}"))?;
    match &v {
        serde_json::Value::Array(arr) => {
            print_text_array(arr);
        }
        serde_json::Value::Object(obj) => {
            print_text_object(obj);
        }
        serde_json::Value::Null => {
            println!("(空)");
        }
        other => {
            // 标量值直接打印
            println!("{other}");
        }
    }
    Ok(())
}

/// 数组 → 表格格式。
///
/// 自动检测数组中每个元素的公共字段作为列名。
/// 空数组打印 "(空)"。
fn print_text_array(arr: &[serde_json::Value]) {
    if arr.is_empty() {
        println!("(空)");
        return;
    }

    // 收集所有可能的列名（按首次出现顺序）
    let mut columns: Vec<String> = Vec::new();
    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            for key in obj.keys() {
                let col = key.clone();
                if !columns.contains(&col) {
                    columns.push(col);
                }
            }
        }
    }

    if columns.is_empty() {
        // 数组元素不是对象（如字符串数组），按列表打印
        for (i, item) in arr.iter().enumerate() {
            println!("  {}. {}", i + 1, format_value(item));
        }
        return;
    }

    // 计算列宽
    let mut col_widths: Vec<usize> = columns.iter().map(String::len).collect();
    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            for (ci, col) in columns.iter().enumerate() {
                if let Some(val) = obj.get(col) {
                    let display = format_value(val);
                    if let Some(w) = col_widths.get_mut(ci) {
                        *w = (*w).max(display.chars().count());
                    }
                }
            }
        }
    }

    // 打印表头
    let header: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| pad_right(c, *col_widths.get(i).unwrap_or(&0)))
        .collect();
    println!("{}", header.join(" │ "));

    // 打印分隔线
    let sep: Vec<String> = col_widths.iter().map(|w| "─".repeat(*w)).collect();
    println!("{}", sep.join("─┼─"));

    // 打印数据行
    for item in arr {
        let row: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(ci, col)| {
                if let serde_json::Value::Object(obj) = item {
                    let val = obj.get(col).map_or(String::new(), format_value);
                    pad_right(&val, *col_widths.get(ci).unwrap_or(&0))
                } else {
                    String::new()
                }
            })
            .collect();
        println!("{}", row.join(" │ "));
    }
}

/// 对象 → 键值对格式。
fn print_text_object(obj: &serde_json::Map<String, serde_json::Value>) {
    if obj.is_empty() {
        println!("(空)");
        return;
    }

    // 计算最大键宽
    let max_key_len = obj.keys().map(|k| k.chars().count()).max().unwrap_or(0);

    for (key, val) in obj {
        let display = format_value(val);
        if display.contains('\n') {
            // 多行值：键单独一行，值缩进
            println!("{key}:");
            for line in display.lines() {
                println!("  {line}");
            }
        } else {
            println!("  {key:>max_key_len$}: {display}");
        }
    }
}

/// 将 [`serde_json::Value`] 转为人类友好的字符串。
fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::from("-"),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            items.join(", ")
        }
        serde_json::Value::Object(_) => String::from("{…}"),
    }
}

/// 右填充字符串到指定宽度（按字符数，非字节数）。
fn pad_right(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - char_count))
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "允许在测试中使用 expect/unwrap/panic"
)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct TestIssue {
        number: u64,
        title: String,
        state: String,
    }

    #[derive(Serialize)]
    struct TestPr {
        number: u64,
        title: String,
        state: String,
        branch: String,
        draft: bool,
    }

    #[test]
    fn test_should_output_json_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("测试标题"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_text_format_for_array() {
        let issues = vec![
            TestIssue {
                number: 1,
                title: String::from("修复登录 bug"),
                state: String::from("open"),
            },
            TestIssue {
                number: 2,
                title: String::from("添加新功能"),
                state: String::from("closed"),
            },
        ];
        let result = print_output(&issues, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_text_format_for_object() {
        let pr = TestPr {
            number: 42,
            title: String::from("重构模块"),
            state: String::from("open"),
            branch: String::from("feat/refactor"),
            draft: false,
        };
        let result = print_output(&pr, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_handle_empty_array() {
        let empty: Vec<TestIssue> = vec![];
        let result = print_output(&empty, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_handle_null_value() {
        let null_val: Option<String> = None;
        let result = print_output(&null_val, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_value_null() {
        assert_eq!(format_value(&serde_json::Value::Null), "-");
    }

    #[test]
    fn test_format_value_bool() {
        assert_eq!(format_value(&serde_json::Value::Bool(true)), "true");
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&serde_json::json!(42)), "42");
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(
            format_value(&serde_json::Value::String(String::from("hello"))),
            "hello"
        );
    }

    #[test]
    fn test_format_value_array() {
        let arr = serde_json::json!(["bug", "enhancement"]);
        assert_eq!(format_value(&arr), "bug, enhancement");
    }

    #[test]
    fn test_pad_right_pads() {
        assert_eq!(pad_right("hi", 5), "hi   ");
    }

    #[test]
    fn test_pad_right_no_truncation() {
        assert_eq!(pad_right("hello", 3), "hello");
    }

    #[test]
    fn test_should_output_toon_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Toon);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_output_auto_format() {
        let issues = vec![TestIssue {
            number: 1,
            title: String::from("test"),
            state: String::from("open"),
        }];
        let result = print_output(&issues, &OutputFormat::Auto);
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli --lib commands::output`
Expected: All tests PASS (including the 2 new ones)

- [ ] **Step 5: Run full test suite**

Run: `make test`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/output.rs apps/cli/tests/toon_output_test.rs
git commit -m "feat(cli): implement TOON and Auto output format branches"
```

---

### Task 5: Quality Gates — Clippy + Format

**Files:**
- No new files; may fix lint issues in files from Tasks 2-4.

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: No warnings

- [ ] **Step 2: Run formatter**

Run: `cargo +nightly fmt --all`
Expected: No changes (or auto-formats)

- [ ] **Step 3: Fix any issues and re-run**

If clippy or fmt report issues, fix them and re-run until clean.

- [ ] **Step 4: Commit (if any fixes)**

```bash
git add -A
git commit -m "style: fix clippy pedantic warnings and formatting for TOON module"
```

---

### Task 6: Update Skills Documentation

**Files:**
- Modify: `.claude/skills/gitflow-workflow/SKILL.md:25`
- Modify: `.claude/skills/gitflow-issue/SKILL.md` (add `--output auto` to command examples)
- Modify: `.claude/skills/gitflow-pr/SKILL.md` (add `--output auto` to command examples)
- Modify: `.claude/skills/gitflow-release/SKILL.md` (add `--output auto` to command examples)
- Modify: `.claude/skills/gitflow-issue-triage/SKILL.md` (add `--output auto` to command examples)

**Interfaces:**
- Consumes: Skill SKILL.md files
- Produces: Updated command examples using `--output auto`

- [ ] **Step 1: Update gitflow-workflow SKILL.md**

Change line 25 from:
```
gitflow-cli issue list --state open --output json
```
to:
```
gitflow-cli issue list --state open --output auto
```

- [ ] **Step 2: Update gitflow-issue SKILL.md**

Find the command examples section (around line 32-33) and add `--output auto` to list/view commands:

```
gitflow-cli issue list [--state open|closed|all] [--label <l>] [--limit <n>] [--output auto]
gitflow-cli issue view <number> [--output auto]
```

- [ ] **Step 3: Update gitflow-pr SKILL.md**

Find the command examples (around line 41) and update:

```
| List/View | `gitflow-cli pr list --output auto` / `pr view <n> --output auto` |
```

- [ ] **Step 4: Update gitflow-release SKILL.md**

Find the command table (around lines 35-36) and update:

```
| List | `gitflow-cli release list --output auto` |
| View | `gitflow-cli release view <tag> --output auto` |
```

- [ ] **Step 5: Update gitflow-issue-triage SKILL.md**

Find the command examples (around lines 25, 33, 35) and add `--output auto`:

```
gitflow-cli issue list --state open [--since <date>] --output auto
```

```
| List open | `gitflow-cli issue list --state open [--since <date>] --output auto` |
| Filter by label | `gitflow-cli issue list --label "<l>" --state open --output auto` |
```

- [ ] **Step 6: Commit**

```bash
git add .claude/skills/
git commit -m "docs(skills): update command examples to use --output auto"
```

---

### Task 7: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `make test`
Expected: ALL TESTS PASS

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: No warnings

- [ ] **Step 3: Run formatter**

Run: `cargo +nightly fmt --all -- --check`
Expected: No formatting changes needed

- [ ] **Step 4: Manual smoke test**

Run:
```bash
cargo run -- --output toon issue list --platform github --state open --limit 3
cargo run -- --output auto issue list --platform github --state open --limit 3
cargo run -- --help | grep -A 5 "output"
```
Expected:
- `--output toon` produces TOON formatted output
- `--output auto` auto-selects format
- `--help` shows `json`, `text`, `toon`, `auto` options
