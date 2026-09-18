# 列表命令分页实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 消除 gf 全部列表/分页命令的静默截断——默认取全量（上限 1000），触顶或用户显式限制时输出机器可读的 `pagination` 元数据与 stderr 警告。

**Architecture:** 在 `gitflow-core` 新增 `Paged<T>` 与通用分页算法 `fetch_capped`（N+1 探测）。三个平台适配器把各自的 `list` 实现改为通过 `fetch_capped` 驱动，按平台能力声明 `FetchStrategy`（gh/gitcode 单次取满，glab 逐页循环）。CLI 层把 `Paged<T>` 拆成 `data`（纯数组，形状不变）与信封上的 `pagination` 字段。六个命令（`issue/pr/release/label/milestone list` 与 `issue comments`）**全部走同一套机制**——api 端点用 `?per_page=N&page=K` 查询参数即可逐页取，无需 `--paginate`。

**Tech Stack:** Rust 2024 / toolchain 1.96.0 · `async-trait` · `serde` · `tokio` · `rstest`（已在 core dev-deps）· `MockCommandRunner`（各适配器 crate 内 `#[cfg(test)]`）

**Spec:** `docs/superpowers/specs/2026-09-17-list-pagination-design.md`

## Global Constraints

- Rust 2024 edition，toolchain 固定 `1.96.0`（`rust-toolchain.toml`）。不得修改该文件。
- 生产代码**禁止** `unwrap()` / `expect()` / `panic!()` / `todo!()`。测试代码中 `expect` 可用。
- 所有 crate 根 `#![forbid(unsafe_code)]`，公开项必须有文档注释，含 `# Errors` 段。
- 所有类型 derive 或实现 `Debug`。
- 必须通过 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`。
- TDD 强制：RED → GREEN → REFACTOR。每个 Task 先写失败测试并**实际运行确认其失败**。
- 测试命名 `test_should_<expected_behavior>`；单元测试放同文件 `#[cfg(test)] mod tests`。
- 日志用 `tracing`，禁止 `println!`/`dbg!`（stderr 警告除外，见 Task 2，那是面向用户的 CLI 输出而非日志）。
- **禁止修改** `deny.toml`、`.pre-commit-config.yaml`、`rust-toolchain.toml`。
- **不得提交**（`git commit`）除非用户明确许可。计划中的 "Commit" 步骤须先向用户请示。
- `DEFAULT_LIST_LIMIT = 1000`（`cap`）。N+1 探测：向下游要 `cap + 1` 条。
- Skill 文档只改 `skills/<name>/SKILL.md`，**不改** `.claude/skills/` 下的副本。

## 与设计文档的两处偏离（已核实，计划以本节为准）

1. **分页模块落位**：设计文档 §6 写 `crates/cli-adapter-utils/src/paging.rs`。**改为 `crates/core/src/paging.rs`**。
   理由：`cli-adapter-utils` 的依赖仅有 `async-trait` + `tokio`，**不依赖 `gitflow-core`**；而
   `Paged<T>` 必须住在 core（它出现在 core 的 provider trait 签名中）。放前者需给一个定位为
   「进程拉起工具」的 crate 新增对领域 core 的依赖，方向颠倒；放 core 则**新增依赖边为零**。
2. **`ListFetcher` trait 改为闭包**：设计文档 §6 定义了 `trait ListFetcher`。按 trait 实现需要
   12 个（4 命令 × 3 平台）样板结构体。toolchain 1.96 的 `AsyncFn` 已稳定，改用
   `fetch: impl AsyncFn(u32, u32) -> Result<Vec<T>>` 后每个调用点只需一个内联闭包。
   语义完全等价，`FetchStrategy` 保留不变。

## 核查中新增的发现（设计文档成文后发现，本计划一并处理）

- `crates/gitcode/src/label.rs:99` 的 `list` 与 `:310` 的 milestone `list` **同样绕过 `runner`**，
  直接用 `tokio::process::Command`。设计文档初版只记了 github milestone 一处。三处一并改走
  `runner`（Task 5 / Task 6），否则它们无法被 `MockCommandRunner` 断言，本计划的 argv 测试对其失效。
- `gf issue list --label` 在 github 与 gitlab 上静默失效（实测：传不存在的标签仍返回全部 33 条）。
  已经用户确认纳入本次，见 Task 3。
- **设计文档 §4 已按实测结论修订**：初版的「A 族 / B 族」两套机制取消。`gh api` 与 `glab api`
  直接接受 `per_page` / `page` 查询参数（已实测），故 api 路径与 list 子命令共用同一分页机制。

## File Structure

| 文件 | 责任 | 动作 |
|---|---|---|
| `crates/core/src/paging.rs` | `Paged<T>`、`FetchStrategy`、`fetch_capped`、`DEFAULT_LIST_LIMIT` | 新建 |
| `crates/core/src/lib.rs` | 导出 `paging` 模块与其关键类型 | 修改 |
| `crates/core/src/output.rs` | `PaginationMeta`、`CliOutput.pagination`、`success_paged` | 修改 |
| `apps/cli/src/commands/output.rs` | `print_list_output`：打印信封 + 截断时向 stderr 警告 | 修改 |
| `crates/core/src/{issue,pr,label,release}.rs` | 四个 provider trait 的 `list` 返回类型改为 `Paged<T>`；`ListReleaseArgs` 新建 | 修改 |
| `crates/{github,gitlab,gitcode}/src/{issue,pr,mr,label,release}.rs` | 各平台 `list` 改由 `fetch_capped` 驱动；`api` 调用加 `--paginate`；三处 runner 旁路修正 | 修改 |
| `apps/cli/src/commands/{issue,pr,label,release}.rs` | 改用 `print_list_output`；`label list` 新增 `--limit`；`release list` 传递 `limit` | 修改 |
| `skills/gf-issue-triage/SKILL.md`、`skills/gf-label-stats/SKILL.md` | 全量调用方式与 `truncated` 处理 | 修改 |

---

### Task 1: core 分页原语

纯函数 + 纯数据，无 I/O、无子进程。这是全计划的地基，必须先独立可测。

**Files:**
- Create: `crates/core/src/paging.rs`
- Modify: `crates/core/src/lib.rs`（新增 `pub mod paging;` 与 re-export）
- Test: 同文件 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: `crate::Result`、`crate::CoreError`
- Produces:
  - `pub const DEFAULT_LIST_LIMIT: u32 = 1000;`
  - `pub struct Paged<T> { pub items: Vec<T>, pub truncated: bool, pub limit: u32, pub total_count: Option<u32> }`
  - `pub enum FetchStrategy { SingleShot, Paged { per_page: u32 } }`
  - `pub async fn fetch_capped<T, F>(strategy: FetchStrategy, cap: u32, fetch: F) -> Result<Paged<T>> where F: AsyncFn(u32, u32) -> Result<Vec<T>>`

- [ ] **Step 1: 写失败测试**

在新建的 `crates/core/src/paging.rs` 末尾写入：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 记录每次 fetch 调用的 (page, limit)，并按预设总量返回数据。
    #[derive(Debug)]
    struct Source {
        total: usize,
        calls: Mutex<Vec<(u32, u32)>>,
    }

    impl Source {
        fn new(total: usize) -> Self {
            Self { total, calls: Mutex::new(Vec::new()) }
        }

        /// 模拟「取第 page 页，每页 limit 条」，数据为 0..total 的连续整数。
        fn page(&self, page: u32, limit: u32) -> Vec<usize> {
            if let Ok(mut calls) = self.calls.lock() {
                calls.push((page, limit));
            }
            let start = (page as usize - 1) * limit as usize;
            if start >= self.total {
                return Vec::new();
            }
            let end = std::cmp::min(start + limit as usize, self.total);
            (start..end).collect()
        }

        fn recorded(&self) -> Vec<(u32, u32)> {
            self.calls.lock().map(|c| c.clone()).unwrap_or_default()
        }
    }

    async fn run_single_shot(total: usize, cap: u32) -> (Paged<usize>, Vec<(u32, u32)>) {
        let src = Source::new(total);
        let paged = fetch_capped(FetchStrategy::SingleShot, cap, async |page, limit| {
            Ok(src.page(page, limit))
        })
        .await
        .expect("fetch_capped should succeed");
        let calls = src.recorded();
        (paged, calls)
    }

    async fn run_paged(total: usize, cap: u32, per_page: u32) -> (Paged<usize>, Vec<(u32, u32)>) {
        let src = Source::new(total);
        let paged = fetch_capped(FetchStrategy::Paged { per_page }, cap, async |page, limit| {
            Ok(src.page(page, limit))
        })
        .await
        .expect("fetch_capped should succeed");
        let calls = src.recorded();
        (paged, calls)
    }

    #[tokio::test]
    async fn test_should_request_cap_plus_one_in_single_shot() {
        let (_, calls) = run_single_shot(5, 10).await;
        assert_eq!(calls, vec![(1, 11)], "single-shot 必须一次要 cap+1 条");
    }

    #[tokio::test]
    async fn test_should_not_truncate_when_total_below_cap() {
        let (paged, _) = run_single_shot(5, 10).await;
        assert_eq!(paged.items.len(), 5);
        assert!(!paged.truncated);
        assert_eq!(paged.limit, 10);
    }

    #[tokio::test]
    async fn test_should_not_truncate_when_total_equals_cap() {
        let (paged, _) = run_single_shot(10, 10).await;
        assert_eq!(paged.items.len(), 10);
        assert!(!paged.truncated, "恰好 cap 条不算截断");
    }

    #[tokio::test]
    async fn test_should_truncate_when_total_exceeds_cap_by_one() {
        let (paged, _) = run_single_shot(11, 10).await;
        assert_eq!(paged.items.len(), 10, "超出部分必须被截掉");
        assert!(paged.truncated, "N+1 探测必须发现还有更多");
    }

    #[tokio::test]
    async fn test_should_truncate_when_total_far_exceeds_cap() {
        let (paged, _) = run_single_shot(500, 10).await;
        assert_eq!(paged.items.len(), 10);
        assert!(paged.truncated);
    }

    #[tokio::test]
    async fn test_should_return_empty_without_truncation_when_source_empty() {
        let (paged, _) = run_single_shot(0, 10).await;
        assert!(paged.items.is_empty());
        assert!(!paged.truncated);
    }

    #[tokio::test]
    async fn test_should_stop_paging_on_short_page() {
        // total=5 < per_page=10 ⇒ 第一页就短，必须停在第 1 页
        let (paged, calls) = run_paged(5, 100, 10).await;
        assert_eq!(paged.items.len(), 5);
        assert!(!paged.truncated);
        assert_eq!(calls, vec![(1, 10)], "短页即取尽，不得再请求第 2 页");
    }

    #[tokio::test]
    async fn test_should_walk_pages_until_cap_plus_one_collected() {
        // cap=25，per_page=10 ⇒ 需要凑够 26 条：第 1/2 页各 10 条，第 3 页再 10 条 = 30 ≥ 26
        let (paged, calls) = run_paged(100, 25, 10).await;
        assert_eq!(paged.items.len(), 25);
        assert!(paged.truncated);
        assert_eq!(calls, vec![(1, 10), (2, 10), (3, 10)]);
    }

    #[tokio::test]
    async fn test_should_stop_paging_when_exhausted_exactly_at_page_boundary() {
        // total=20，per_page=10：第 1、2 页各满 10 条，第 3 页返回 0 条 ⇒ 取尽
        let (paged, calls) = run_paged(20, 100, 10).await;
        assert_eq!(paged.items.len(), 20);
        assert!(!paged.truncated);
        assert_eq!(calls, vec![(1, 10), (2, 10), (3, 10)]);
    }

    #[tokio::test]
    async fn test_should_preserve_item_order_across_pages() {
        let (paged, _) = run_paged(25, 100, 10).await;
        assert_eq!(paged.items, (0..25).collect::<Vec<usize>>());
    }

    #[tokio::test]
    async fn test_should_reject_zero_per_page() {
        let src = Source::new(10);
        let result = fetch_capped(FetchStrategy::Paged { per_page: 0 }, 10, async |page, limit| {
            Ok(src.page(page, limit))
        })
        .await;
        assert!(
            matches!(result, Err(CoreError::Platform(_))),
            "per_page = 0 会导致死循环，必须直接报错"
        );
    }

    #[tokio::test]
    async fn test_should_propagate_fetch_error() {
        let result: Result<Paged<usize>> =
            fetch_capped(FetchStrategy::SingleShot, 10, async |_page, _limit| {
                Err(CoreError::Platform("boom".to_string()))
            })
            .await;
        assert!(matches!(result, Err(CoreError::Platform(msg)) if msg == "boom"));
    }

    #[tokio::test]
    async fn test_should_return_empty_and_truncated_when_cap_is_zero() {
        let (paged, _) = run_single_shot(5, 0).await;
        assert!(paged.items.is_empty());
        assert!(paged.truncated, "cap=0 但源里有数据，必须诚实报告截断");
    }

    #[test]
    fn test_should_expose_default_list_limit_of_1000() {
        assert_eq!(DEFAULT_LIST_LIMIT, 1000);
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

先在 `crates/core/src/lib.rs` 的模块列表中（第 36 行 `pub mod output;` 之后，按字母序）加入：

```rust
pub mod paging;
```

Run: `cargo test -p gitflow-core --lib paging`
Expected: FAIL —— 编译错误，`cannot find type Paged`、`cannot find function fetch_capped` 等。

- [ ] **Step 3: 写最小实现**

在 `crates/core/src/paging.rs` **顶部**（测试模块之前）写入：

```rust
//! 列表分页原语。
//!
//! 底层平台 CLI 的列表命令默认只返回首页（通常 30 条）且不给任何截断信号，
//! 调用方无从判断拿到的是全集还是首页。本模块提供统一的「取到上限为止 +
//! 触顶时诚实上报」语义。
//!
//! 上限不是设计偏好而是硬约束：`gh` 没有「无限」选项，`--limit` 只能是一个
//! 具体数字，因此全量必然有界，有界就必须能被发现。

use crate::{CoreError, Result};

/// 列表命令默认抓取的条目上限。
///
/// 超过此数时结果被截断，且 [`Paged::truncated`] 置为 `true`。
pub const DEFAULT_LIST_LIMIT: u32 = 1000;

/// 一次分页抓取的结果。
///
/// 这是 provider trait 的内部返回类型，**不实现 `Serialize`**：CLI 层在输出前
/// 会把它拆成纯数组（落到 `data`）与元数据（落到输出信封的 `pagination`），
/// 以保证 `data` 的形状不因本特性而改变。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Paged<T> {
    /// 已截断到上限之内的条目，顺序与平台返回顺序一致。
    pub items: Vec<T>,
    /// 是否因触顶而丢弃了更多条目。由 N+1 探测得出，而非推测。
    pub truncated: bool,
    /// 本次生效的上限。
    pub limit: u32,
    /// 平台原生便宜可得时的总数，否则为 `None`。
    ///
    /// GitHub 上恒为 `None`：`gh issue list --json` 的字段集中没有总数，
    /// 取真实总数需另发一次 GraphQL/search 查询，不值得为此多打一轮 API。
    pub total_count: Option<u32>,
}

/// 平台的分页能力。
///
/// 用枚举显式表达，而非靠算术巧合让循环在单次取满的平台上恰好只跑一轮。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchStrategy {
    /// 底层 CLI 自行翻页，一次调用即可取至多 N 条。
    ///
    /// `gh`（`--limit`）与 gitcode（`--limit`）适用。
    SingleShot,
    /// 必须由调用方逐页请求，每页至多 `per_page` 条。
    ///
    /// `glab` 适用：其 `--per-page` 受 GitLab API 限制，上限 100。
    Paged {
        /// 每页条数，必须大于 0。
        per_page: u32,
    },
}

/// 按 `strategy` 抓取至多 `cap` 条，并诚实报告是否还有更多。
///
/// 内部向下游请求 `cap + 1` 条（N+1 探测）：只要 `cap` 条时无法区分
/// 「恰好 cap 条」与「还有更多」。收到多于 `cap` 条即截断并置
/// [`Paged::truncated`]。
///
/// `fetch` 的两个参数是 `(page, limit)`：
/// - [`FetchStrategy::SingleShot`]：`page` 恒为 1，`limit` 为 `cap + 1`。
/// - [`FetchStrategy::Paged`]：`page` 从 1 递增，`limit` 为 `per_page`。
///
/// # Errors
///
/// - `fetch` 返回的错误原样透传。
/// - [`FetchStrategy::Paged`] 的 `per_page` 为 0 时返回
///   [`CoreError::Platform`]：该取值会导致翻页永不前进。
/// - 页号溢出 `u32` 时返回 [`CoreError::Platform`]。
pub async fn fetch_capped<T, F>(strategy: FetchStrategy, cap: u32, fetch: F) -> Result<Paged<T>>
where
    F: AsyncFn(u32, u32) -> Result<Vec<T>>,
{
    let want = cap.saturating_add(1) as usize;
    let mut items: Vec<T> = Vec::new();

    match strategy {
        FetchStrategy::SingleShot => {
            let limit = cap.saturating_add(1);
            items = fetch(1, limit).await?;
        }
        FetchStrategy::Paged { per_page } => {
            if per_page == 0 {
                return Err(CoreError::Platform(
                    "pagination per_page must be greater than zero".to_string(),
                ));
            }
            let mut page: u32 = 1;
            loop {
                let batch = fetch(page, per_page).await?;
                let batch_len = batch.len();
                items.extend(batch);

                if batch_len < per_page as usize {
                    break; // 短页 ⇒ 已取尽
                }
                if items.len() >= want {
                    break; // 已够 N+1 探测所需
                }
                page = page.checked_add(1).ok_or_else(|| {
                    CoreError::Platform("pagination page number overflowed".to_string())
                })?;
            }
        }
    }

    let truncated = items.len() > cap as usize;
    if truncated {
        items.truncate(cap as usize);
    }

    Ok(Paged {
        items,
        truncated,
        limit: cap,
        total_count: None,
    })
}
```

然后在 `crates/core/src/lib.rs` 的 re-export 区（第 51 行 `pub use output::{CliError, CliOutput};` 之后）加入：

```rust
pub use paging::{DEFAULT_LIST_LIMIT, FetchStrategy, Paged, fetch_capped};
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-core --lib paging`
Expected: PASS，14 个测试全绿。

- [ ] **Step 5: 静态检查**

Run: `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警。若报 `cast_possible_truncation`（`cap as usize`），改用 `usize::try_from(cap).unwrap_or(usize::MAX)` 并补一行注释说明 32→64 位在受支持平台上无损。

- [ ] **Step 6: 请示后提交**

⚠️ 先向用户请求提交许可（项目规定禁止未经许可提交）。获准后：

```bash
git add crates/core/src/paging.rs crates/core/src/lib.rs
git commit -m "feat(core): add Paged<T> and fetch_capped pagination primitive"
```

---

### Task 2: 输出信封与截断警告

把 `Paged<T>` 拆成 `data` + `pagination` 的边界层。核心约束是**零破坏**：非列表命令的输出必须与改动前逐字节相同。

**Files:**
- Modify: `crates/core/src/output.rs`（新增 `PaginationMeta`、`CliOutput.pagination`、`success_paged`）
- Modify: `crates/core/src/paging.rs`（新增 `Paged::into_parts`）
- Modify: `apps/cli/src/commands/output.rs`（新增 `truncation_warning`、`print_list_output`）
- Test: 三个文件各自的 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: Task 1 的 `Paged<T>`
- Produces:
  - `pub struct PaginationMeta { pub truncated: bool, pub returned: usize, pub limit: u32, pub total_count: Option<u32> }`
  - `impl<T> Paged<T> { pub fn into_parts(self) -> (Vec<T>, PaginationMeta) }`
  - `pub fn CliOutput::success_paged(data: T, pagination: PaginationMeta, platform: &str, command: &str) -> Self`
  - `pub fn truncation_warning(meta: &PaginationMeta) -> Option<String>`（apps/cli）
  - `pub fn print_list_output<T: Serialize>(items: T, meta: PaginationMeta, platform: &str, command: &str, format: &OutputFormat) -> miette::Result<()>`（apps/cli）

- [ ] **Step 1: 写失败测试（core 侧）**

在 `crates/core/src/output.rs` 的 `#[cfg(test)] mod tests` 中追加：

```rust
    #[test]
    fn test_should_omit_pagination_key_for_non_list_output() {
        let output = CliOutput::success("payload", "github", "issue view");
        let json = serde_json::to_value(&output).expect("serialize");
        assert!(
            json.get("pagination").is_none(),
            "非列表命令的输出必须与改动前逐字节相同，不得出现 pagination 键"
        );
    }

    #[test]
    fn test_should_keep_data_as_array_for_paged_output() {
        let paged = crate::paging::Paged {
            items: vec![1_u32, 2, 3],
            truncated: false,
            limit: 1000,
            total_count: None,
        };
        let (items, meta) = paged.into_parts();
        let output = CliOutput::success_paged(items, meta, "github", "issue list");
        let json = serde_json::to_value(&output).expect("serialize");
        assert!(
            json["data"].is_array(),
            "data 必须仍是数组，不得被包装成 {{items: [..]}}"
        );
        assert_eq!(json["data"].as_array().map(Vec::len), Some(3));
    }

    #[test]
    fn test_should_emit_pagination_meta_in_camel_case() {
        let paged = crate::paging::Paged {
            items: vec![1_u32],
            truncated: true,
            limit: 1,
            total_count: None,
        };
        let (items, meta) = paged.into_parts();
        let output = CliOutput::success_paged(items, meta, "github", "issue list");
        let json = serde_json::to_value(&output).expect("serialize");
        assert_eq!(json["pagination"]["truncated"], serde_json::json!(true));
        assert_eq!(json["pagination"]["returned"], serde_json::json!(1));
        assert_eq!(json["pagination"]["limit"], serde_json::json!(1));
        assert!(
            json["pagination"].get("totalCount").is_none(),
            "totalCount 为 None 时必须整个省略，而非输出 null"
        );
    }

    #[test]
    fn test_should_derive_returned_from_item_count() {
        let paged = crate::paging::Paged {
            items: vec!["a", "b"],
            truncated: false,
            limit: 1000,
            total_count: None,
        };
        let (_, meta) = paged.into_parts();
        assert_eq!(meta.returned, 2);
        assert!(!meta.truncated);
        assert_eq!(meta.limit, 1000);
    }
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-core --lib output`
Expected: FAIL —— `cannot find function success_paged`、`no method named into_parts`。

- [ ] **Step 3: 写最小实现**

在 `crates/core/src/paging.rs` 的 `fetch_capped` 之后加入：

```rust
impl<T> Paged<T> {
    /// 拆成「纯条目」与「分页元数据」两半。
    ///
    /// CLI 层用它把条目放进输出的 `data`（保持数组形状），把元数据放进
    /// 输出信封的 `pagination` 字段。
    #[must_use]
    pub fn into_parts(self) -> (Vec<T>, crate::output::PaginationMeta) {
        let meta = crate::output::PaginationMeta {
            truncated: self.truncated,
            returned: self.items.len(),
            limit: self.limit,
            total_count: self.total_count,
        };
        (self.items, meta)
    }
}
```

在 `crates/core/src/output.rs` 中，`CliOutput` 结构体定义里 `pub error` 之后、`pub platform` 之前插入字段：

```rust
    /// 分页元数据，仅列表类命令出现。
    ///
    /// 非列表命令不序列化此字段，因此其输出与本特性引入前逐字节相同。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
```

在同文件中新增类型与构造函数：

```rust
/// 列表类命令的分页元数据。
///
/// 出现在输出信封上而非 `data` 内部，因此 `data` 仍是数组，既有的
/// `.data[]` 消费方（jq 脚本、skill）不受影响。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PaginationMeta {
    /// 是否因触顶而丢弃了更多条目。
    pub truncated: bool,
    /// 本次实际返回的条目数。
    pub returned: usize,
    /// 本次生效的上限。
    pub limit: u32,
    /// 平台原生便宜可得时的总数；GitHub 上恒为 `None`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u32>,
}
```

`CliOutput::success` 与 `CliOutput::failure` 的结构体字面量中补 `pagination: None`。
新增：

```rust
    /// 创建带分页元数据的成功输出。
    #[must_use]
    pub fn success_paged(
        data: T,
        pagination: PaginationMeta,
        platform: &str,
        command: &str,
    ) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: Some(pagination),
            platform: platform.into(),
            command: command.into(),
        }
    }
```

若 `crates/core/src/lib.rs` 第 51 行的 re-export 需要同步，改为：

```rust
pub use output::{CliError, CliOutput, PaginationMeta};
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-core --lib`
Expected: PASS。若其他 crate 因 `CliOutput` 字面量缺字段而编译失败，改用 `CliOutput::success(..)` 构造函数而非直接字面量。

Run: `cargo build --workspace`
Expected: 成功。

- [ ] **Step 5: 写 CLI 侧失败测试**

在 `apps/cli/src/commands/output.rs` 的 `#[cfg(test)] mod tests` 中追加（若无该模块则新建）：

```rust
    use gitflow_core::PaginationMeta;

    #[test]
    fn test_should_not_warn_when_not_truncated() {
        let meta = PaginationMeta {
            truncated: false,
            returned: 33,
            limit: 1000,
            total_count: None,
        };
        assert_eq!(truncation_warning(&meta), None);
    }

    #[test]
    fn test_should_warn_with_counts_when_truncated() {
        let meta = PaginationMeta {
            truncated: true,
            returned: 1000,
            limit: 1000,
            total_count: None,
        };
        let warning = truncation_warning(&meta).expect("truncated 必须产生警告");
        assert!(warning.contains("1000"), "警告必须含实际返回条数");
        assert!(warning.contains("--limit"), "警告必须告诉用户如何提高上限");
    }
```

- [ ] **Step 6: 运行确认失败**

Run: `cargo test -p gitflow-cli --lib commands::output`
Expected: FAIL —— `cannot find function truncation_warning`。

- [ ] **Step 7: 实现 CLI 侧**

在 `apps/cli/src/commands/output.rs` 中加入：

```rust
/// 构造截断警告文案；未截断时返回 `None`。
///
/// 抽成纯函数以便单元测试——实际写 stderr 的动作在
/// [`print_list_output`] 中，本身不含逻辑。
#[must_use]
pub fn truncation_warning(meta: &gitflow_core::PaginationMeta) -> Option<String> {
    if !meta.truncated {
        return None;
    }
    Some(format!(
        "⚠️  结果被截断：返回 {returned} 条，还有更多未取回。用 --limit <N> 提高上限。",
        returned = meta.returned
    ))
}

/// 打印列表类命令的输出：数据走 stdout，截断警告走 stderr。
///
/// 警告走 stderr 而非 stdout，以保证 stdout 仍可直接喂给 `jq`。
///
/// # Errors
///
/// 序列化或格式化失败时返回错误。
pub fn print_list_output<T: serde::Serialize>(
    items: T,
    meta: gitflow_core::PaginationMeta,
    platform: &str,
    command: &str,
    format: &OutputFormat,
) -> miette::Result<()> {
    let warning = truncation_warning(&meta);
    let output = gitflow_core::CliOutput::success_paged(items, meta, platform, command);
    print_output(&output, format)?;
    if let Some(warning) = warning {
        eprintln!("{warning}");
    }
    Ok(())
}
```

- [ ] **Step 8: 运行测试确认通过**

Run: `cargo test -p gitflow-cli --lib commands::output`
Expected: PASS。

- [ ] **Step 9: 静态检查**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警。

- [ ] **Step 10: 请示后提交**

⚠️ 先请求提交许可。获准后：

```bash
git add crates/core/src/output.rs crates/core/src/paging.rs crates/core/src/lib.rs apps/cli/src/commands/output.rs
git commit -m "feat(core,cli): add PaginationMeta envelope field and truncation warning"
```

---

### Task 3: `issue list` 三平台改造 + 修复 `--label` 静默失效

第一个 A 族纵切，确立后续 Task 4–6 复用的模式。trait 签名变更会同时打断三个平台的编译，
因此三平台必须在同一个 Task 内完成，否则工作区无法保持可构建。

**本 Task 附带修复两个核查中发现的缺陷**（用户已确认纳入）：

- `ListIssueArgs.labels` 在 github 与 gitlab 的 `list` 中**从未被读取**，`--label` 过滤静默失效。
  实测：`gf issue list --label "definitely-no-such-label-xyz"` 返回全部 33 条 Issue。
  受害方是 AC#4 点名的 `gf-label-stats`——它按标签逐个计数，在这两个平台上每个标签都会
  得到同一个数字（open 总数），整份报告失真。
- `ListIssueArgs.assignee` 三个平台均未使用，且 CLI 恒传 `None`，是死字段，予以删除。

**Files:**
- Modify: `crates/core/src/issue.rs`（`list` 返回类型；删除 `ListIssueArgs.assignee`）
- Modify: `crates/github/src/issue.rs:232-277`
- Modify: `crates/gitlab/src/issue.rs:405-450`
- Modify: `crates/gitcode/src/issue.rs:388-426`
- Create: `apps/cli/src/commands/list_args.rs`
- Modify: `apps/cli/src/commands/mod.rs`（挂载 `list_args` 模块）
- Modify: `apps/cli/src/commands/issue.rs:253-284`
- Test: 上述各适配器文件的 `#[cfg(test)] mod tests`，以及 `list_args.rs` 自带测试

**Interfaces:**
- Consumes: `gitflow_core::{DEFAULT_LIST_LIMIT, FetchStrategy, Paged, fetch_capped}`（Task 1）、
  `Paged::into_parts`、`apps/cli` 的 `print_list_output`（Task 2）
- Produces:
  - `IssueProvider::list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>>`
  - `pub fn validate_limit(limit: Option<u32>) -> Result<Option<u32>, UserInputError>`（apps/cli）
  - 各平台内部无新增公开项

- [ ] **Step 1: 写 `validate_limit` 的失败测试**

新建 `apps/cli/src/commands/list_args.rs`：

```rust
//! 列表类命令的共享参数校验。

use crate::errors::UserInputError;

/// 校验 `--limit`：必须大于 0。
///
/// `0` 会让分页器返回空集并报告截断，对用户毫无意义，因此在信任边界处直接拒绝，
/// 而不是让它穿透到适配器。`None` 原样透传，由适配器套用
/// [`gitflow_core::DEFAULT_LIST_LIMIT`]。
///
/// # Errors
///
/// `limit` 为 `Some(0)` 时返回 [`UserInputError`]。
pub fn validate_limit(limit: Option<u32>) -> Result<Option<u32>, UserInputError> {
    if limit == Some(0) {
        return Err(UserInputError::new(
            "Invalid --limit '0'. Expected a positive integer.".to_string(),
        ));
    }
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_accept_absent_limit() {
        assert_eq!(validate_limit(None), Ok(None));
    }

    #[test]
    fn test_should_accept_positive_limit() {
        assert_eq!(validate_limit(Some(1)), Ok(Some(1)));
        assert_eq!(validate_limit(Some(5000)), Ok(Some(5000)));
    }

    #[test]
    fn test_should_reject_zero_limit() {
        let err = validate_limit(Some(0)).expect_err("0 必须被拒绝");
        assert!(err.to_string().contains("--limit"));
    }
}
```

若 `UserInputError` 未实现 `PartialEq`，把前两个断言改为 `assert!(matches!(validate_limit(None), Ok(None)))` 形式。

在 `apps/cli/src/commands/mod.rs` 中按字母序加入：

```rust
pub mod list_args;
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test -p gitflow-cli --lib commands::list_args`
Expected: FAIL —— 模块尚未挂载或 `UserInputError` 导入路径不符时的编译错误。修正导入后应转为 PASS。

- [ ] **Step 3: 写三平台的失败测试（精确 argv 断言）**

在 `crates/github/src/issue.rs` 的 `#[cfg(test)] mod tests` 中追加：

```rust
    #[tokio::test]
    async fn test_should_request_default_cap_plus_one_when_limit_absent() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner.clone());
        let paged = provider
            .list(ListIssueArgs::default())
            .await
            .expect("list should succeed");
        assert!(paged.items.is_empty());
        assert!(!paged.truncated);
        let args = &runner.recorded_calls()[0].1;
        assert!(
            args.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"),
            "无 --limit 时必须向 gh 要 DEFAULT_LIST_LIMIT + 1 = 1001 条，实际 argv: {args:?}"
        );
    }

    #[tokio::test]
    async fn test_should_pass_user_limit_plus_one_to_gh() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner.clone());
        let args = ListIssueArgs {
            limit: Some(10),
            ..ListIssueArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "11"),
            "用户指定 --limit 10 时也要跑 N+1 探测，实际 argv: {recorded:?}"
        );
    }

    #[tokio::test]
    async fn test_should_report_truncation_when_gh_returns_more_than_cap() {
        // cap = 2 ⇒ 请求 3 条；返回 3 条 ⇒ 截到 2 条并置 truncated
        let stdout = r#"[
            {"number":1,"title":"a","state":"OPEN","body":"","labels":[],"assignees":[],"author":{"login":"u","id":"1"},"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","url":"https://example.com/1"},
            {"number":2,"title":"b","state":"OPEN","body":"","labels":[],"assignees":[],"author":{"login":"u","id":"1"},"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","url":"https://example.com/2"},
            {"number":3,"title":"c","state":"OPEN","body":"","labels":[],"assignees":[],"author":{"login":"u","id":"1"},"createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","url":"https://example.com/3"}
        ]"#;
        let runner = MockCommandRunner::success(stdout);
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner);
        let args = ListIssueArgs {
            limit: Some(2),
            ..ListIssueArgs::default()
        };
        let paged = provider.list(args).await.expect("list should succeed");
        assert_eq!(paged.items.len(), 2);
        assert!(paged.truncated);
        assert_eq!(paged.limit, 2);
    }

    #[tokio::test]
    async fn test_should_forward_label_filter_to_gh() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner.clone());
        let args = ListIssueArgs {
            labels: vec!["bug".to_string(), "help wanted".to_string()],
            ..ListIssueArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--label" && w[1] == "bug"),
            "--label 过滤此前被静默丢弃，必须真正传给 gh，实际 argv: {recorded:?}"
        );
        assert!(
            recorded.windows(2).any(|w| w[0] == "--label" && w[1] == "help wanted"),
            "多个标签必须各自重复 --label，实际 argv: {recorded:?}"
        );
    }
```

在 `crates/gitlab/src/issue.rs` 的测试模块中追加：

```rust
    #[tokio::test]
    async fn test_should_walk_pages_with_per_page_100_on_glab() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner.clone());
        provider
            .list(ListIssueArgs::default())
            .await
            .expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"),
            "glab 的 --per-page 受 API 限制上限 100，必须按页大小而非总数传，实际 argv: {recorded:?}"
        );
        assert!(
            recorded.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "必须显式指定页号，实际 argv: {recorded:?}"
        );
        assert_eq!(
            runner.recorded_calls().len(),
            1,
            "首页为空（短于 per_page）即已取尽，不得请求第 2 页"
        );
    }

    #[tokio::test]
    async fn test_should_forward_label_filter_to_glab() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner.clone());
        let args = ListIssueArgs {
            labels: vec!["bug".to_string()],
            ..ListIssueArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--label" && w[1] == "bug"),
            "--label 过滤此前被静默丢弃，必须真正传给 glab，实际 argv: {recorded:?}"
        );
    }
```

在 `crates/gitcode/src/issue.rs` 的测试模块中追加：

```rust
    #[tokio::test]
    async fn test_should_request_default_cap_plus_one_on_gitcode() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());
        provider
            .list(ListIssueArgs::default())
            .await
            .expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"),
            "实际 argv: {recorded:?}"
        );
    }

    #[tokio::test]
    async fn test_should_keep_forwarding_label_filter_on_gitcode() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());
        let args = ListIssueArgs {
            labels: vec!["bug".to_string()],
            ..ListIssueArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(recorded.windows(2).any(|w| w[0] == "--label" && w[1] == "bug"));
    }
```

若各 crate 的 provider 构造函数名或 `MockCommandRunner::success` 的签名与上述不符，
以该 crate 现有测试中的写法为准——不要新增构造函数。

- [ ] **Step 4: 运行确认失败**

Run: `cargo test -p gitflow-github -p gitflow-gitlab -p gitflow-gitcode --lib issue`
Expected: FAIL —— `Paged` 未知、`list` 返回类型不匹配、`--label` 断言失败。

- [ ] **Step 5: 改 core trait**

`crates/core/src/issue.rs`：删除 `ListIssueArgs` 的 `assignee` 字段（第 82-83 行的文档注释与字段），
并把 `list` 签名改为：

```rust
    /// 根据过滤条件列出 Issue 列表。
    ///
    /// 返回的 [`Paged`] 携带截断标志：平台侧的列表命令默认只返回首页，
    /// 本方法保证要么取满上限，要么诚实报告还有更多。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或过滤条件非法时返回错误。
    async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>>;
```

在该文件顶部导入 `use crate::paging::Paged;`。同文件第 286 行附近的
`assert!(args.assignee.is_none());` 一并删除。

- [ ] **Step 6: 改 github 实现**

把 `crates/github/src/issue.rs:232-277` 的整个 `list` 替换为：

```rust
    async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let state = args.state.map(|state| match state {
            State::Open => "open",
            State::Closed => "closed",
            State::All => "all",
        });

        debug!(repo = %self.repo, cap, "spawning `gh issue list`");

        fetch_capped(FetchStrategy::SingleShot, cap, async |_page, limit| {
            let limit_str = limit.to_string();
            let mut cmd_args: Vec<&str> = vec![
                "issue",
                "list",
                "--repo",
                &self.repo,
                "--json",
                ISSUE_FIELDS,
            ];

            if let Some(state) = state {
                cmd_args.push("--state");
                cmd_args.push(state);
            }

            if let Some(ref search) = args.search {
                cmd_args.push("--search");
                cmd_args.push(search);
            }

            for label in &args.labels {
                cmd_args.push("--label");
                cmd_args.push(label);
            }

            cmd_args.push("--limit");
            cmd_args.push(&limit_str);

            let output = self
                .runner
                .run("gh", &cmd_args)
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

            if !output.status.success() {
                return Err(parse_gh_error(&output.stderr).into());
            }

            let issues: Vec<IssueData> =
                serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
            Ok(issues)
        })
        .await
    }
```

在文件顶部的 `use gitflow_core::...` 中加入 `DEFAULT_LIST_LIMIT, FetchStrategy, Paged, fetch_capped`。

- [ ] **Step 7: 改 gitlab 实现**

把 `crates/gitlab/src/issue.rs:405-450` 的整个 `list` 替换为：

```rust
    async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning `glab issue list`");

        fetch_capped(
            FetchStrategy::Paged {
                per_page: GITLAB_MAX_PER_PAGE,
            },
            cap,
            async |page, per_page| {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let mut cmd_args: Vec<&str> = vec![
                    "issue",
                    "list",
                    "--repo",
                    &self.repo_target,
                    "--output",
                    "json",
                ];

                // glab 用 --closed 表示已关闭、--all 表示全部；默认（不加旗标）为 open。
                if let Some(state) = &args.state {
                    match state {
                        State::Closed => cmd_args.push("--closed"),
                        State::All => cmd_args.push("--all"),
                        State::Open => {}
                    }
                }

                if let Some(ref search) = args.search {
                    cmd_args.push("--search");
                    cmd_args.push(search);
                }

                for label in &args.labels {
                    cmd_args.push("--label");
                    cmd_args.push(label);
                }

                cmd_args.push("--per-page");
                cmd_args.push(&per_page_str);
                cmd_args.push("--page");
                cmd_args.push(&page_str);

                let output = self
                    .runner
                    .run("glab", &cmd_args)
                    .await
                    .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

                if !output.status.success() {
                    return Err(parse_glab_error(&output.stderr).into());
                }

                let issues: Vec<IssueData> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(issues)
            },
        )
        .await
    }
```

在 `crates/gitlab/src/lib.rs` 中新增常量（若已有同名常量则复用）：

```rust
/// GitLab API 对单页条数的硬上限。
///
/// 传入更大的值不会报错，服务端会静默按 100 处理——这正是此前
/// `--limit 1000` 在 GitLab 上只返回 100 条的原因。
pub(crate) const GITLAB_MAX_PER_PAGE: u32 = 100;
```

⚠️ **注意 `State::All` 与翻页的交互**：`--all` 在 glab 中表示「不限状态」，与
`--per-page`/`--page` 不冲突，可同时使用。切勿改用 `-A/--all`（那是「取全部条目」，
无上界，会让 cap 形同虚设）。

- [ ] **Step 8: 改 gitcode 实现**

把 `crates/gitcode/src/issue.rs:388-426` 的整个 `list` 替换为：

```rust
    async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        let binary = crate::gitcode_binary();
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning gitcode issue list");

        // gitcode CLI 在开发环境不可获得，其 `--limit` 语义未经实测验证，
        // 此处按与 gh 相同的「总条数上限」处理。若实际为页大小，N+1 探测会
        // 过度上报 truncated 而非静默丢数据——失效方向是安全的。
        fetch_capped(FetchStrategy::SingleShot, cap, async |_page, limit| {
            let limit_str = limit.to_string();
            let mut cmd_args: Vec<&str> = vec!["issue", "list", "-R", &self.repo, "--json"];

            if let Some(ref state) = args.state {
                cmd_args.push("--state");
                cmd_args.push(match state {
                    State::Open => "open",
                    State::Closed => "closed",
                    State::All => "all",
                });
            }
            if let Some(ref search) = args.search {
                cmd_args.push("--search");
                cmd_args.push(search);
            }
            for label in &args.labels {
                cmd_args.push("--label");
                cmd_args.push(label);
            }
            cmd_args.push("--limit");
            cmd_args.push(&limit_str);

            let output = self
                .runner
                .run(&binary, &cmd_args)
                .await
                .map_err(|e| CoreError::Platform(format!("{e}")))?;
            if !output.status.success() {
                return Err(parse_gitcode_error(&output.stderr).into());
            }
            let issues: Vec<IssueApiResponse> =
                serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
            Ok(issues.into_iter().map(IssueData::from).collect())
        })
        .await
    }
```

- [ ] **Step 9: 改 CLI 接线**

把 `apps/cli/src/commands/issue.rs:253-284` 的 `IssueCommand::List` 分支替换为：

```rust
        IssueCommand::List {
            state,
            search,
            label,
            limit,
        } => {
            let parsed_state = state
                .as_deref()
                .map(|s| match s {
                    "open" => Ok(State::Open),
                    "closed" => Ok(State::Closed),
                    "all" => Ok(State::All),
                    other => Err(crate::errors::UserInputError::new(format!(
                        "Invalid state '{other}'. Expected 'open', 'closed', or 'all'."
                    ))),
                })
                .transpose()?;

            let limit = crate::commands::list_args::validate_limit(limit)?;

            let args = ListIssueArgs {
                state: parsed_state,
                labels: label,
                search,
                limit,
            };
            let paged = provider
                .list(args)
                .await
                .map_err(|e| miette::miette!("Failed to list issues: {e}"))?;
            let (items, meta) = paged.into_parts();
            print_list_output(items, meta, platform, "issue list", &output_format)?;
        }
```

在该文件顶部加入本地转发（与既有 `print_output` 包装同风格）：

```rust
fn print_list_output<T: serde::Serialize>(
    items: T,
    meta: gitflow_core::PaginationMeta,
    platform: &str,
    command: &str,
    format: &OutputFormat,
) -> miette::Result<()> {
    crate::commands::output::print_list_output(items, meta, platform, command, format)
}
```

- [ ] **Step 10: 运行测试确认通过**

Run: `cargo test -p gitflow-core -p gitflow-github -p gitflow-gitlab -p gitflow-gitcode -p gitflow-cli --lib`
Expected: PASS。

Run: `cargo build --workspace`
Expected: 成功。

- [ ] **Step 11: 真实验证 `--label` 修复**

Run: `cargo run -p gitflow-cli -- issue list --state open --limit 1000 --label "definitely-no-such-label-xyz" --output json | jq '.data | length'`
Expected: `0`（修复前为 `33`）。

- [ ] **Step 12: 静态检查与提交**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`

⚠️ 先请求提交许可。获准后：

```bash
git add crates/core/src/issue.rs crates/github/src/issue.rs crates/gitlab/src/issue.rs crates/gitlab/src/lib.rs crates/gitcode/src/issue.rs apps/cli/src/commands/issue.rs apps/cli/src/commands/list_args.rs apps/cli/src/commands/mod.rs
git commit -m "fix(issue): paginate issue list to a cap and forward label filter

--label was silently dropped on GitHub and GitLab, returning unfiltered
results that looked filtered. Verified: a nonexistent label returned all
33 open issues."
```

---

### Task 4: `pr list` 三平台改造

与 Task 3 同构但更简单——`ListPrArgs` 只有 `state` 与 `limit` 两个字段，没有 label 过滤要修。

**Files:**
- Modify: `crates/core/src/pr.rs:115`（`list` 返回类型）、`:384`（trait 测试桩）
- Modify: `crates/github/src/pr.rs:157-191`
- Modify: `crates/gitlab/src/mr.rs:340-380`
- Modify: `crates/gitcode/src/pr.rs:275-310`
- Modify: `apps/cli/src/commands/pr.rs`（`PrCommand::List` 分支）

**Interfaces:**
- Consumes: `gitflow_core::{DEFAULT_LIST_LIMIT, FetchStrategy, Paged, fetch_capped}`、
  `crates/gitlab/src/lib.rs` 的 `GITLAB_MAX_PER_PAGE`（Task 3 已建）、
  `apps/cli` 的 `validate_limit` 与 `print_list_output`（Task 2/3 已建）
- Produces: `PrProvider::list(&self, args: ListPrArgs) -> Result<Paged<PrData>>`

- [ ] **Step 1: 写失败测试**

在 `crates/github/src/pr.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_request_default_cap_plus_one_for_pr_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner.clone());
        let paged = provider
            .list(ListPrArgs::default())
            .await
            .expect("list should succeed");
        assert!(!paged.truncated);
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"),
            "实际 argv: {recorded:?}"
        );
    }

    #[tokio::test]
    async fn test_should_still_forward_state_filter_for_pr_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubPrProvider::with_runner("owner/repo", runner.clone());
        let args = ListPrArgs {
            state: Some(State::Open),
            ..ListPrArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(recorded.windows(2).any(|w| w[0] == "--state" && w[1] == "open"));
    }
```

在 `crates/gitlab/src/mr.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_walk_pages_for_mr_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabMrProvider::with_runner("owner/repo", runner.clone());
        provider
            .list(ListPrArgs::default())
            .await
            .expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(recorded.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"));
        assert!(recorded.windows(2).any(|w| w[0] == "--page" && w[1] == "1"));
    }
```

在 `crates/gitcode/src/pr.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_request_default_cap_plus_one_for_gitcode_pr_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodePrProvider::with_runner("owner/repo", runner.clone());
        provider
            .list(ListPrArgs::default())
            .await
            .expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"));
    }
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test -p gitflow-github -p gitflow-gitlab -p gitflow-gitcode --lib pr`
Expected: FAIL（类型不匹配 / argv 断言失败）。

- [ ] **Step 3: 改 core trait**

`crates/core/src/pr.rs:115`：

```rust
    /// 根据过滤条件列出 PR 列表。
    ///
    /// 返回的 [`Paged`] 携带截断标志，语义见 [`crate::paging`]。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败或过滤条件非法时返回错误。
    async fn list(&self, args: ListPrArgs) -> Result<Paged<PrData>>;
```

顶部加 `use crate::paging::Paged;`。第 384 行的测试桩改为：

```rust
            async fn list(&self, _args: crate::pr::ListPrArgs) -> Result<crate::paging::Paged<crate::pr::PrData>> {
                Ok(crate::paging::Paged {
                    items: Vec::new(),
                    truncated: false,
                    limit: crate::paging::DEFAULT_LIST_LIMIT,
                    total_count: None,
                })
            }
```

- [ ] **Step 4: 改 github 实现**

替换 `crates/github/src/pr.rs` 的 `list`：

```rust
    async fn list(&self, args: ListPrArgs) -> Result<Paged<PrData>> {
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let state = args.state.map(|state| match state {
            State::Open => "open",
            State::Closed => "closed",
            State::All => "all",
        });

        debug!(repo = %self.repo, cap, "spawning `gh pr list`");

        fetch_capped(FetchStrategy::SingleShot, cap, async |_page, limit| {
            let limit_str = limit.to_string();
            let mut cmd_args: Vec<&str> =
                vec!["pr", "list", "--repo", &self.repo, "--json", PR_FIELDS];

            if let Some(state) = state {
                cmd_args.push("--state");
                cmd_args.push(state);
            }
            cmd_args.push("--limit");
            cmd_args.push(&limit_str);

            let output = self
                .runner
                .run("gh", &cmd_args)
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

            if !output.status.success() {
                return Err(parse_gh_error(&output.stderr).into());
            }

            let prs: Vec<PrData> =
                serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
            Ok(prs)
        })
        .await
    }
```

- [ ] **Step 5: 改 gitlab 实现**

替换 `crates/gitlab/src/mr.rs` 的 `list`（保留其原有的 `--closed`/`--all` 状态映射与
其余已有旗标，仅把 `--per-page` 的来源从 `args.limit` 改为翻页参数）：

```rust
    async fn list(&self, args: ListPrArgs) -> Result<Paged<PrData>> {
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning `glab mr list`");

        fetch_capped(
            FetchStrategy::Paged {
                per_page: crate::GITLAB_MAX_PER_PAGE,
            },
            cap,
            async |page, per_page| {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let mut cmd_args: Vec<&str> = vec![
                    "mr",
                    "list",
                    "--repo",
                    &self.repo_target,
                    "--output",
                    "json",
                ];

                if let Some(state) = &args.state {
                    match state {
                        State::Closed => cmd_args.push("--closed"),
                        State::All => cmd_args.push("--all"),
                        State::Open => {}
                    }
                }

                cmd_args.push("--per-page");
                cmd_args.push(&per_page_str);
                cmd_args.push("--page");
                cmd_args.push(&page_str);

                let output = self
                    .runner
                    .run("glab", &cmd_args)
                    .await
                    .map_err(|e| CoreError::Platform(format!("Failed to spawn glab: {e}")))?;

                if !output.status.success() {
                    return Err(parse_glab_error(&output.stderr).into());
                }

                let mrs: Vec<PrData> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(mrs)
            },
        )
        .await
    }
```

若该文件原实现在解析前还有中间响应类型（如 `MrApiResponse`），沿用它，只替换闭包内的解析行。

- [ ] **Step 6: 改 gitcode 实现**

替换 `crates/gitcode/src/pr.rs` 的 `list`，结构与 Task 3 的 gitcode issue 实现一致：
`FetchStrategy::SingleShot`，闭包内保留原有的 `-R`/`--json`/`--state` 旗标，
把 `--limit` 的值改为闭包参数 `limit`。

- [ ] **Step 7: 改 CLI 接线**

`apps/cli/src/commands/pr.rs` 的 `PrCommand::List` 分支：`limit` 先过
`crate::commands::list_args::validate_limit(limit)?`，结果 `paged.into_parts()` 后
用 `print_list_output(items, meta, platform, "pr list", &output_format)?`，
并在该文件加与 Task 3 相同的 `print_list_output` 本地转发函数。

- [ ] **Step 8: 验证与提交**

Run: `cargo test --workspace --lib` → PASS
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` → 无告警

⚠️ 请求提交许可后：

```bash
git add crates/core/src/pr.rs crates/github/src/pr.rs crates/gitlab/src/mr.rs crates/gitcode/src/pr.rs apps/cli/src/commands/pr.rs
git commit -m "fix(pr): paginate pr list to a cap instead of inheriting CLI default"
```

---

### Task 5: `release list` 三平台改造 + 修复被丢弃的 `--limit`

**本 Task 修复一个已确认的旗标失效**：`apps/cli/src/commands/release.rs:188` 的
`ReleaseCommand::List { .. }` 用 `..` 忽略了 `limit`，而 `provider.list()` 根本不收参数。
用户以为自己限定了条数，实际拿的是 `gh release list` 的默认 30 条（已实测该默认值）。
现有测试（`release.rs:502`）只断言 `limit` 被解析为 `Some(10)`，从不断言它生效——
这正是该 bug 存活至今的原因，因此本 Task 必须补上"抵达 provider"的断言。

**Files:**
- Modify: `crates/core/src/release.rs:93`
- Modify: `crates/github/src/release.rs:136-161`
- Modify: `crates/gitlab/src/release.rs:236-258`
- Modify: `crates/gitcode/src/release.rs:145-167`
- Modify: `apps/cli/src/commands/release.rs:188-195`、`:502` 附近的测试

**Interfaces:**
- Produces: `ReleaseProvider::list(&self, limit: Option<u32>) -> Result<Paged<ReleaseData>>`

签名用裸 `limit: Option<u32>` 而非新建 `ListReleaseArgs` 结构体：当前只有一个字段，
按 YAGNI 不引入新类型。

- [ ] **Step 1: 写失败测试**

在 `apps/cli/src/commands/release.rs` 的测试模块追加——这是本 Task 的核心回归护栏：

```rust
    #[test]
    fn test_should_not_discard_release_list_limit() {
        // 回归护栏：此前 `ReleaseCommand::List { .. }` 用 `..` 把 limit 整个丢弃，
        // 而既有测试只断言解析结果，因此该缺陷长期未被发现。
        let cli = crate::Cli::try_parse_from(["gitflow", "release", "list", "--limit", "10"])
            .expect("parse should succeed");
        match cli.command {
            crate::Commands::Release(ReleaseCommand::List { limit }) => {
                assert_eq!(limit, Some(10));
            }
            _ => panic!("Expected ReleaseCommand::List"),
        }
    }
```

在 `crates/github/src/release.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_request_default_cap_plus_one_for_release_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner.clone());
        provider.list(None).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"),
            "release list 此前完全没有 limit 概念，实际 argv: {recorded:?}"
        );
    }

    #[tokio::test]
    async fn test_should_honour_user_release_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubReleaseProvider::with_runner("owner/repo", runner.clone());
        provider.list(Some(10)).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "11"),
            "用户 limit 必须真正抵达底层 CLI，实际 argv: {recorded:?}"
        );
    }
```

在 `crates/gitlab/src/release.rs` 与 `crates/gitcode/src/release.rs` 的测试模块分别追加
与 Task 4 Step 1 同形的翻页/上限断言（gitlab 断言 `--per-page 100` + `--page 1`；
gitcode 断言 `--limit 1001`）。

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --workspace --lib release`
Expected: FAIL。

- [ ] **Step 3: 改 core trait**

`crates/core/src/release.rs:93`：

```rust
    /// 列出仓库的 Release。
    ///
    /// `limit` 为 `None` 时取至 [`crate::paging::DEFAULT_LIST_LIMIT`]。
    /// 返回的 [`Paged`] 携带截断标志。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败时返回错误。
    async fn list(&self, limit: Option<u32>) -> Result<Paged<ReleaseData>>;
```

- [ ] **Step 4: 改三平台实现**

github（`crates/github/src/release.rs`）：

```rust
    async fn list(&self, limit: Option<u32>) -> Result<Paged<ReleaseData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning `gh release list`");

        fetch_capped(FetchStrategy::SingleShot, cap, async |_page, want| {
            let want_str = want.to_string();
            let output = self
                .runner
                .run(
                    "gh",
                    &[
                        "release",
                        "list",
                        "--repo",
                        &self.repo,
                        "--json",
                        RELEASE_LIST_FIELDS,
                        "--limit",
                        &want_str,
                    ],
                )
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh: {e}")))?;

            if !output.status.success() {
                return Err(parse_gh_error(&output.stderr).into());
            }

            let releases: Vec<ReleaseData> =
                serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
            Ok(releases)
        })
        .await
    }
```

gitlab（`crates/gitlab/src/release.rs`）：同形，改用
`FetchStrategy::Paged { per_page: crate::GITLAB_MAX_PER_PAGE }`，闭包内在原有
`["release","list","--repo",&self.repo_target,"--output","json"]` 之后追加
`"--per-page", &per_page_str, "--page", &page_str`。

gitcode（`crates/gitcode/src/release.rs`）：同形，`FetchStrategy::SingleShot`，
闭包内在原有 `["release","list","-R",&self.repo,"--json",RELEASE_FIELDS]` 之后追加
`"--limit", &want_str`。

- [ ] **Step 5: 改 CLI 接线**

`apps/cli/src/commands/release.rs:188`：

```rust
        ReleaseCommand::List { limit } => {
            let limit = crate::commands::list_args::validate_limit(limit)?;
            let paged = provider
                .list(limit)
                .await
                .map_err(|e| miette::miette!("Failed to list releases: {e}"))?;
            let (items, meta) = paged.into_parts();
            print_list_output(items, meta, platform, "release list", &output_format)?;
        }
```

注意：模式必须从 `{ .. }` 改为 `{ limit }`，否则 bug 原样留存。

- [ ] **Step 6: 验证与提交**

Run: `cargo test --workspace --lib` → PASS
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`

⚠️ 请求提交许可后：

```bash
git add crates/core/src/release.rs crates/github/src/release.rs crates/gitlab/src/release.rs crates/gitcode/src/release.rs apps/cli/src/commands/release.rs
git commit -m "fix(release): honour --limit in release list instead of discarding it

The List arm destructured with `..`, so the parsed --limit never reached
the provider and users silently got gh's 30-item default."
```

---

### Task 6: `label list` / `milestone list` 改造 + 三处 runner 旁路修正

**本 Task 修复三个绕过 `runner` 的实现**：`crates/github/src/label.rs:366`（milestone）、
`crates/gitcode/src/label.rs:99`（label）与 `:310`（milestone）直接用
`tokio::process::Command`，因此无法被 `MockCommandRunner` 断言。不先修好这一点，
本 Task 的 argv 测试对它们完全失效。

**Files:**
- Modify: `crates/core/src/label.rs:102`（label list）、`:142`（milestone list）
- Modify: `crates/github/src/label.rs:128-150`（label）、`:366-382`（milestone，改走 runner + api 分页）
- Modify: `crates/gitlab/src/label.rs:119-140`（`list_api`）、`:479-495`（milestone）
- Modify: `crates/gitcode/src/label.rs:96-115`（label，改走 runner）、`:307-330`（milestone，改走 runner）
- Modify: `apps/cli/src/commands/label.rs`（两个 List 分支 + 新增 `--limit`）

**Interfaces:**
- Produces:
  - `LabelProvider::list(&self, limit: Option<u32>) -> Result<Paged<LabelData>>`
  - `MilestoneProvider::list(&self, limit: Option<u32>) -> Result<Paged<MilestoneData>>`

- [ ] **Step 1: 写失败测试**

在 `crates/github/src/label.rs` 的测试模块中，把既有的
`test_should_...` 里断言 `"--limit", "100"` 的用例（约在第 600-613 行）改为断言 `"1001"`，
并追加 milestone 的 runner 断言：

```rust
    #[tokio::test]
    async fn test_should_replace_hardcoded_label_limit_with_cap() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubLabelProvider::with_runner("owner/repo", runner.clone());
        provider.list(None).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded.windows(2).any(|w| w[0] == "--limit" && w[1] == "1001"),
            "硬编码的 --limit 100 必须被 cap 取代，实际 argv: {recorded:?}"
        );
    }

    #[tokio::test]
    async fn test_should_route_milestone_list_through_runner() {
        // 回归护栏：此前该实现直接 tokio::process::Command，完全绕过 runner，
        // 因此任何 argv 断言都对它无效。
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubMilestoneProvider::with_runner("owner/repo", runner.clone());
        provider.list(None).await.expect("list should succeed");
        assert_eq!(
            runner.recorded_calls().len(),
            1,
            "milestone list 必须经由 runner 发起，否则不可测"
        );
        let recorded = &runner.recorded_calls()[0].1;
        assert_eq!(recorded[0], "api");
        assert!(
            recorded[1].contains("per_page=100") && recorded[1].contains("page=1"),
            "api 路径必须带分页查询参数，实际: {recorded:?}"
        );
    }
```

在 `crates/gitcode/src/label.rs` 的测试模块追加两个同类断言，分别覆盖 label 与 milestone
经由 runner 发起（`runner.recorded_calls().len() == 1`）。

在 `crates/gitlab/src/label.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_page_glab_label_list() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner.clone());
        provider.list(None).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(recorded.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"));
        assert!(recorded.windows(2).any(|w| w[0] == "--page" && w[1] == "1"));
    }
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --workspace --lib label`
Expected: FAIL。

- [ ] **Step 3: 改 core trait**

`crates/core/src/label.rs`，两处签名：

```rust
    /// 列出仓库的全部标签。
    ///
    /// `limit` 为 `None` 时取至 [`crate::paging::DEFAULT_LIST_LIMIT`]。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败时返回错误。
    async fn list(&self, limit: Option<u32>) -> Result<Paged<LabelData>>;
```

```rust
    /// 列出仓库的全部里程碑。
    ///
    /// `limit` 为 `None` 时取至 [`crate::paging::DEFAULT_LIST_LIMIT`]。
    ///
    /// # Errors
    ///
    /// 当平台 API 调用失败时返回错误。
    async fn list(&self, limit: Option<u32>) -> Result<Paged<MilestoneData>>;
```

- [ ] **Step 4: 改 github label（去硬编码 100）**

```rust
    async fn list(&self, limit: Option<u32>) -> Result<Paged<LabelData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning `gh label list`");

        fetch_capped(FetchStrategy::SingleShot, cap, async |_page, want| {
            let want_str = want.to_string();
            let output = self
                .runner
                .run(
                    "gh",
                    &[
                        "label",
                        "list",
                        "--repo",
                        &self.repo,
                        "--json",
                        LABEL_FIELDS,
                        "--limit",
                        &want_str,
                    ],
                )
                .await
                .map_err(|e| CoreError::Platform(format!("Failed to spawn gh label list: {e}")))?;

            if !output.status.success() {
                return Err(parse_gh_error(&output.stderr).into());
            }

            let labels: Vec<LabelData> =
                serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
            Ok(labels)
        })
        .await
    }
```

- [ ] **Step 5: 改 github milestone（改走 runner + api 分页参数）**

```rust
    async fn list(&self, limit: Option<u32>) -> Result<Paged<MilestoneData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, cap, "spawning `gh api` GET milestones");

        fetch_capped(
            FetchStrategy::Paged { per_page: 100 },
            cap,
            async |page, per_page| {
                // gh api 直接接受 per_page / page 查询参数（已实测），
                // 因此无需 --paginate，且截断可被探测。
                let api_path = format!(
                    "repos/{repo}/milestones?per_page={per_page}&page={page}",
                    repo = self.repo
                );

                let output = self
                    .runner
                    .run("gh", &["api", &api_path])
                    .await
                    .map_err(|e| {
                        CoreError::Platform(format!("Failed to spawn gh api milestones: {e}"))
                    })?;

                if !output.status.success() {
                    return Err(parse_gh_error(&output.stderr).into());
                }

                let milestones: Vec<MilestoneApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(milestones.into_iter().map(MilestoneData::from).collect())
            },
        )
        .await
    }
```

该 provider 若尚无 `runner` 字段与 `with_runner` 构造函数，照 `GitHubLabelProvider`
的既有写法补上（同文件内已有范例），不要新造模式。

- [ ] **Step 6: 改 gitlab label 与 milestone**

`list_api` 改为接收 `(page, per_page)` 并在 argv 末尾追加
`"--per-page", &per_page_str, "--page", &page_str`；`list` 用
`FetchStrategy::Paged { per_page: crate::GITLAB_MAX_PER_PAGE }` 驱动它。
milestone（`:479`）同形处理，argv 基底保持
`["milestone","list","--project",&self.project_target,"--output","json"]`。

注意 `list_api` 还被同文件的 `delete`（按名查 id，约 `:215`）复用——那里需要的是
**全量**而非首页，改造后应传 `None`（即 cap 为 `DEFAULT_LIST_LIMIT`）并使用
`paged.items`；若 `paged.truncated` 为真，返回
`CoreError::Platform("label lookup truncated; too many labels to resolve by name")`
而不是在残缺集合里查找后误报"未找到"。

- [ ] **Step 7: 改 gitcode label 与 milestone（改走 runner）**

两处把 `tokio::process::Command::new(crate::gitcode_binary())` 换成
`self.runner.run(&binary, &cmd_args)`，argv 保持原样
（label: `["label","list","-R",&self.repo,"--json",LABEL_FIELDS]`；
milestone: `["milestone","list","-R",&self.repo,"--json"]`），
策略声明为 `FetchStrategy::SingleShot`，**且不追加 `--limit`**——该旗标在 gitcode 上
是否存在无从实测，乱传会直接让命令报错。

在两处各加一行注释：

```rust
            // gitcode CLI 在开发环境不可获得，其 label/milestone 列表是否有服务端
            // 默认上限无从实测，故不传 limit 旗标。后果：若确有默认上限，本实现
            // 无法探测也无法报告其截断。见设计文档 §4.3。
```

若 provider 无 `runner` 字段，照同 crate 内 `GitCodeIssueProvider` 的写法补上。

- [ ] **Step 8: 改 CLI 接线并新增 `--limit`**

`apps/cli/src/commands/label.rs`：给 `LabelCommand::List` 与 `MilestoneCommand::List`
（或等价的子命令变体）各加：

```rust
        /// 返回数量上限。
        #[arg(long)]
        limit: Option<u32>,
```

分支体改为 `validate_limit` → `provider.list(limit)` → `into_parts()` → `print_list_output`，
命令名分别为 `"label list"` 与 `"milestone list"`，并加本地 `print_list_output` 转发函数。

- [ ] **Step 9: 验证与提交**

Run: `cargo test --workspace --lib` → PASS
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`

⚠️ 请求提交许可后：

```bash
git add crates/core/src/label.rs crates/github/src/label.rs crates/gitlab/src/label.rs crates/gitcode/src/label.rs apps/cli/src/commands/label.rs
git commit -m "fix(label): paginate label and milestone lists, route them through runner

Three implementations spawned tokio::process::Command directly, bypassing
the injectable runner and making their argv untestable."
```

---

### Task 7: `issue comments` 三平台分页

最后一个命令。三平台都走 `api` 子命令，因此三处形态一致，都用
`FetchStrategy::Paged { per_page: 100 }` 配合查询参数。

**Files:**
- Modify: `crates/core/src/issue.rs`（`list_comments` 签名）
- Modify: `crates/github/src/issue.rs:433-456`
- Modify: `crates/gitlab/src/issue.rs:587-610`
- Modify: `crates/gitcode/src/issue.rs:556-580`
- Modify: `apps/cli/src/commands/issue.rs`（`IssueCommand::Comments` 分支）

**Interfaces:**
- Produces: `IssueProvider::list_comments(&self, number: u64, limit: Option<u32>) -> Result<Paged<CommentData>>`

- [ ] **Step 1: 写失败测试**

在 `crates/github/src/issue.rs` 的测试模块追加：

```rust
    #[tokio::test]
    async fn test_should_page_issue_comments_via_query_params() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitHubIssueProvider::with_runner("owner/repo", runner.clone());
        provider
            .list_comments(359, None)
            .await
            .expect("list_comments should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert_eq!(recorded[0], "api");
        assert!(
            recorded[1].contains("per_page=100") && recorded[1].contains("page=1"),
            "评论此前只取首页（30 条）且无信号，实际: {recorded:?}"
        );
    }
```

在 gitlab 与 gitcode 的测试模块追加同形断言（路径基底分别为
`/projects/{encoded}/issues/{n}/notes` 与 `/repos/{repo}/issues/{n}/comments`）。

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --workspace --lib comments`
Expected: FAIL。

- [ ] **Step 3: 改 core trait**

```rust
    /// 列出指定 Issue 的评论。
    ///
    /// `limit` 为 `None` 时取至 [`crate::paging::DEFAULT_LIST_LIMIT`]。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或平台 API 调用失败时返回错误。
    async fn list_comments(&self, number: u64, limit: Option<u32>) -> Result<Paged<CommentData>>;
```

- [ ] **Step 4: 改 github 实现**

```rust
    async fn list_comments(&self, number: u64, limit: Option<u32>) -> Result<Paged<CommentData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        debug!(repo = %self.repo, number, cap, "spawning `gh api` GET issue comments");

        fetch_capped(
            FetchStrategy::Paged { per_page: 100 },
            cap,
            async |page, per_page| {
                let api_path = format!(
                    "repos/{repo}/issues/{number}/comments?per_page={per_page}&page={page}",
                    repo = self.repo
                );

                let output = self
                    .runner
                    .run("gh", &["api", &api_path])
                    .await
                    .map_err(|e| CoreError::Platform(format!("Failed to spawn gh api: {e}")))?;

                if !output.status.success() {
                    return Err(parse_gh_error(&output.stderr).into());
                }

                let comments: Vec<GitHubCommentApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(comments.into_iter().map(CommentData::from).collect())
            },
        )
        .await
    }
```

- [ ] **Step 5: 改 gitlab / gitcode 实现**

gitlab：同形，路径为
`format!("/projects/{encoded_path}/issues/{number}/notes?per_page={per_page}&page={page}")`，
程序为 `"glab"`，保留原有的 `encode_project_path` 调用与响应类型。

gitcode：同形，路径为
`format!("/repos/{repo}/issues/{number}/comments?per_page={per_page}&page={page}")`，
程序为 `&binary`。加注释说明 GitCode 的 `per_page`/`page` 支持**未经实测**，
若该平台忽略这两个参数，翻页循环会在首页短于 `per_page` 时正常终止，
退化为当前行为而不会死循环或丢数据。

- [ ] **Step 6: 改 CLI 接线**

`apps/cli/src/commands/issue.rs` 的 `Comments` 子命令加 `--limit` 参数，分支体走
`validate_limit` → `list_comments(number, limit)` → `into_parts()` →
`print_list_output(items, meta, platform, "issue comments", &output_format)`。

- [ ] **Step 7: 验证与提交**

Run: `cargo test --workspace --lib` → PASS
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`

⚠️ 请求提交许可后：

```bash
git add crates/core/src/issue.rs crates/github/src/issue.rs crates/gitlab/src/issue.rs crates/gitcode/src/issue.rs apps/cli/src/commands/issue.rs
git commit -m "fix(issue): paginate issue comments instead of returning only page one"
```

---

### Task 8: 修正两份 Skill 文档（AC#4）

**Files:**
- Modify: `skills/gf-issue-triage/SKILL.md:49,57,72`
- Modify: `skills/gf-label-stats/SKILL.md:53,54,62,63,75,76,100`

⚠️ **只改 `skills/` 下的源文件，不要改 `.claude/skills/` 下的副本**（项目规定）。

- [ ] **Step 1: 修 `gf-label-stats` 的假兜底**

该文件第 100 行当前写：

```
| >1000 Issues | Paginate with `--limit` + `--page`. |
```

**`--page` 这个参数在 `gf issue list` 中根本不存在**（已核实 `apps/cli/src/commands/issue.rs:71-87`
的 `List` 变体只有 `state` / `search` / `label` / `limit` 四个参数）。该行是一个不可执行的
假兜底。替换为：

```
| >1000 Issues | `gf issue list` 默认取至 1000 条上限。触顶时输出 `pagination.truncated: true`，此时必须用 `--limit <N>` 提高上限重取，并在报告中声明本次覆盖范围。 |
```

同时删除第 53、54、62、63、75、76 行中的 `--limit 1000`——默认已是 1000，显式指定反而
在将来上限调整时产生不一致。即把

```
gf issue list --label "<l>" --state open --limit 1000
gf issue list --state open --limit 1000
```

改为

```
gf issue list --label "<l>" --state open
gf issue list --state open
```

- [ ] **Step 2: 给两份 skill 加截断检查条目**

在 `skills/gf-issue-triage/SKILL.md` 的 Step 1（第 72 行）之后插入：

```markdown
### Step 1b: Verify coverage — read `pagination.truncated` from the response.

If `truncated` is `true`, the fetch hit the limit and more Issues exist. Re-run with a
higher `--limit`, or state the partial coverage explicitly in the report. Never present
a truncated fetch as a complete classification.
```

在 `skills/gf-label-stats/SKILL.md` 的对应位置（第 75 行 Step 2 之后）插入等价条目。

- [ ] **Step 3: 记录此前报告可能失真**

在 `skills/gf-issue-triage/SKILL.md` 的 "When NOT to Use" 表之后插入：

```markdown
> **Historical note:** before the pagination fix (Issue #360), `gf issue list --state open`
> silently returned only the first 30 Issues. Triage reports generated before that fix may
> have classified an incomplete set. Re-run triage rather than trusting an older report.
```

- [ ] **Step 4: 同步校验**

Run: `make check-agent-sync`
Expected: 通过。若报告 `skills/` 与 `.claude/skills/` 不同步，按该 target 的提示同步，
**不要反向编辑** `.claude/skills/`。

- [ ] **Step 5: 请示后提交**

```bash
git add skills/gf-issue-triage/SKILL.md skills/gf-label-stats/SKILL.md
git commit -m "docs(skills): drop the nonexistent --page fallback, add truncation checks

gf-label-stats documented pagination via `--limit` + `--page`, but --page
was never a gf issue list flag. The fallback could never have run."
```

---

### Task 9: 端到端验证（AC#5）与验收对照

不写新代码，只跑验证并记录证据。若任何一项不符，回到对应 Task 修复。

- [ ] **Step 1: 安装本地构建**

Run: `cargo install --path apps/cli --force`

（`gf` 通过 PATH 解析，不重新安装则验证的是旧二进制。）

- [ ] **Step 2: 默认调用不再丢数据**

Run: `gf issue list --state open --output json | jq '{n: (.data|length), pagination}'`

Expected：`n` 为 33（而非修复前的 30），`pagination.truncated` 为 `false`，
`pagination.limit` 为 `1000`。

Run: `gf issue list --state open --output json | jq -r '.data[].number' | sort -n | grep -E '^(93|101|102)$'`
Expected：三个编号全部出现（修复前一个都没有）。

- [ ] **Step 3: 显式截断有信号**

Run: `gf issue list --state open --limit 10 --output json | jq '{n: (.data|length), pagination}'`
Expected：`n` 为 10，`pagination.truncated` 为 `true`。

Run: `gf issue list --state open --limit 10 --output json 2>/dev/null >/dev/null; echo "---"; gf issue list --state open --limit 10 --output json 2>&1 >/dev/null`
Expected：第二条命令在 stderr 上打印含「截断」与 `--limit` 的警告；stdout 被丢弃后仍有警告输出，
证明警告确实走 stderr 而非 stdout。

- [ ] **Step 4: 非列表命令逐字节兼容**

Run: `gf issue view 360 --output json | jq 'has("pagination")'`
Expected：`false`。

- [ ] **Step 5: `--label` 过滤真正生效**

Run: `gf issue list --state open --label "definitely-no-such-label-xyz" --output json | jq '.data | length'`
Expected：`0`（修复前为 `33`）。

- [ ] **Step 6: `release list --limit` 真正生效**

Run: `gf release list --limit 1 --output json | jq '{n: (.data|length), pagination}'`
Expected：`n` 至多为 1；若仓库 Release 多于 1 个，`pagination.truncated` 为 `true`。

- [ ] **Step 7: 评论分页**

Run: `gf issue comments 359 --output json | jq '{n: (.data|length), pagination}'`
Expected：返回全部评论（该 Issue 目前 2 条），`truncated` 为 `false`。

- [ ] **Step 8: 全量门禁**

Run: `make test`
Run: `cargo test --workspace`（含 doctest，nextest 不跑 doctest）
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
Run: `cargo +nightly fmt --check`

全部通过方可进入交付。

- [ ] **Step 9: 填写验收对照表**

把下表连同实际输出一并记入交付说明：

| AC | 验证步骤 | 结果 |
|---|---|---|
| ① 默认全量或显式截断信号 | Step 2 + Step 3 | |
| ② 「静默丢数据」不得保留 | Step 2 / 3 / 5 / 6 / 7 | |
| ③ pr/label/release list 一并核查 | Task 4/5/6 的单元测试 | |
| ④ 自称全量的 skill 显式用全量调用 | Task 8 + Step 5 | |
| ⑤ 可复现验证 | Step 2–7 | |

**必须如实记录的缺口**：gitcode 的 `label list` / `milestone list` 截断无法探测
（设计文档 §4.3），不得声称三平台全覆盖。

---

## Self-Review

**1. Spec coverage**

| 设计文档章节 | 落点 |
|---|---|
| §1 问题与发现 1–8 | 发现 1 → Task 3/4 的 `--per-page` 改造；发现 2 → Task 5；发现 3 → Task 6；发现 4 → Task 5；发现 5 → Task 7；发现 6 → Task 6；发现 7/8 → Task 8 |
| §2 底层 CLI 能力 | Task 3–7 各自的策略选择，旗标均已实测 |
| §3 选型与排除项 | Task 1 的 `fetch_capped`（N+1 探测 + cap） |
| §4.1 六命令策略矩阵 | Task 3（issue）/ 4（pr）/ 5（release）/ 6（label、milestone）/ 7（comments） |
| §4.2 pipeline 排除 | 无任务，属有意不改；已在本计划 File Structure 外记录 |
| §4.3 gitcode 缺口 | Task 6 Step 7 的注释 + Task 9 Step 9 的如实记录 |
| §5.0 两个类型两个边界 | Task 2 的 `Paged::into_parts` |
| §5.1 信封零破坏 | Task 2 Step 1 的 `test_should_omit_pagination_key_for_non_list_output` + Task 9 Step 4 |
| §5.2 `totalCount` 降级 | Task 1 的 `total_count: None` + Task 2 的 `skip_serializing_if` |
| §5.3 stderr 警告 | Task 2 的 `truncation_warning` + Task 9 Step 3 |
| §6 分页器 | Task 1（落位改为 core，理由见「与设计文档的两处偏离」） |
| §7 签名变更与 limit 语义 | Task 3–7 各自的 trait 改动 |
| §8 Skill 修正 | Task 8 |
| §9 测试策略 | Task 1–7 各自的测试步骤 |
| §11 可复现验证 | Task 9 |

无遗漏章节。

**2. Placeholder scan**

已检查：无 "TBD"、"TODO"、"稍后实现"、"添加适当的错误处理"、"为上述编写测试"。
每个代码步骤都带可直接粘贴的代码块。Task 4 Step 6、Task 5 Step 4（gitlab/gitcode）、
Task 6 Step 6、Task 7 Step 5 采用"以同 Task 内已给出的完整实现为模板 + 明确列出差异项"
的写法，差异项（argv 基底、路径模板、策略、响应类型）均已逐条写明，不是
"参照 Task N"式的空指。

**3. Type consistency**

- `Paged<T>` 的字段 `items` / `truncated` / `limit` / `total_count` 在 Task 1 定义，
  Task 2 的 `into_parts`、Task 3–7 的构造、Task 9 的 jq 断言用法一致。
- `PaginationMeta` 的字段 `truncated` / `returned` / `limit` / `total_count` 在 Task 2 定义，
  序列化为 camelCase（`totalCount`），Task 9 Step 2/3 的 jq 断言与之一致。
- `fetch_capped(strategy, cap, fetch)` 的参数顺序在 Task 1 定义，Task 3–7 全部按此顺序调用。
- `FetchStrategy::Paged { per_page }` 的字段名在 Task 1 定义，Task 3–7 一致。
- `validate_limit` 在 Task 3 定义，Task 4–7 引用路径均为
  `crate::commands::list_args::validate_limit`。
- `print_list_output(items, meta, platform, command, format)` 的参数顺序在 Task 2 定义，
  Task 3–7 一致。
- `GITLAB_MAX_PER_PAGE` 在 Task 3 Step 7 建于 `crates/gitlab/src/lib.rs`，
  Task 4/5/6 以 `crate::GITLAB_MAX_PER_PAGE` 引用。

**4. 执行顺序依赖**

Task 1 → 2 → 3 为强顺序（后者依赖前者产出的类型与辅助函数）。
Task 4、5、6、7 三者之间**无相互依赖**，可并行或任意顺序执行，但都依赖 Task 1–3。
Task 8 只改文档，无代码依赖。Task 9 依赖全部前序 Task。
