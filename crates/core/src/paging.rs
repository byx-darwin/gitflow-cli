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

/// 按 [`FetchStrategy::Paged`] 逐页抓取，直到取够 `want` 条或页面取尽。
///
/// 只负责翻页循环本身：不做上限截断（由调用方 `fetch_capped` 统一处理），
/// 输入输出与被提炼前的 `match` 分支完全一致（`per_page`/`want`/`fetch` →
/// 收集到的条目）。
///
/// # Errors
///
/// - `per_page` 为 0 时返回 [`CoreError::Platform`]：该取值会导致翻页永不前进。
/// - `fetch` 返回的错误原样透传。
/// - 页号溢出 `u32` 时返回 [`CoreError::Platform`]。
async fn fetch_all_pages<T, F, Fut>(per_page: u32, want: usize, fetch: &F) -> Result<Vec<T>>
where
    F: Fn(u32, u32) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>>>,
{
    if per_page == 0 {
        return Err(CoreError::Platform(
            "pagination per_page must be greater than zero".to_string(),
        ));
    }
    let mut items: Vec<T> = Vec::new();
    let mut page: u32 = 1;
    loop {
        let batch = fetch(page, per_page).await?;
        let batch_len = batch.len();
        items.extend(batch);

        // Lossless on all supported 32/64-bit platforms: u32::MAX always fits in usize.
        let per_page_usize = usize::try_from(per_page).unwrap_or(usize::MAX);
        if batch_len < per_page_usize {
            break; // 短页 ⇒ 已取尽
        }
        if items.len() >= want {
            break; // 已够 N+1 探测所需
        }
        page = page
            .checked_add(1)
            .ok_or_else(|| CoreError::Platform("pagination page number overflowed".to_string()))?;
    }
    Ok(items)
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
///
/// `fetch` 是返回 `Future` 的普通闭包（`Fn(u32, u32) -> Fut`），而非原生
/// `AsyncFn`：在 `#[async_trait]` 装箱的方法体内调用一个捕获引用的
/// `async move |..| {..}` 闭包会触发 rustc 的 HRTB `Send` 检查缺陷
/// （`implementation of Send is not general enough`）。返回具名 `Future` 的
/// 普通闭包不受此限制影响。
///
/// # Examples
///
/// 从一个自身借用了 `self` 字段的方法体里调用时，**借用**要捕获的字段
/// （而非 `.clone()` 后 `move` 进闭包），再让每次调用返回的 `async move`
/// 块去移动那个借用——借用是 `Copy`，因此闭包仍满足 `Fn`：
///
/// ```
/// use gitflow_core::{FetchStrategy, Result, fetch_capped};
///
/// struct Source {
///     repo: String,
/// }
///
/// impl Source {
///     async fn list(&self, cap: u32) -> Result<Vec<String>> {
///         let repo = &self.repo;
///         let paged = fetch_capped(FetchStrategy::SingleShot, cap, |_page, limit| async move {
///             // `repo` 是 `&String`（`Copy`），可在每次调用中重新借出，
///             // 无需 `.clone()`；此处仅返回不超过 `limit` 条的假数据。
///             let all: Vec<String> = (0..3).map(|i| format!("{repo}#{i}")).collect();
///             Ok(all.into_iter().take(limit as usize).collect())
///         })
///         .await?;
///         Ok(paged.items)
///     }
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// let source = Source { repo: "octocat/hello-world".to_string() };
/// let items = source.list(10).await.expect("fetch should succeed");
/// assert_eq!(items.len(), 3);
/// # }
/// ```
pub async fn fetch_capped<T, F, Fut>(
    strategy: FetchStrategy,
    cap: u32,
    fetch: F,
) -> Result<Paged<T>>
where
    F: Fn(u32, u32) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>>>,
{
    // Lossless on all supported 32/64-bit platforms: u32::MAX always fits in usize there.
    let want = usize::try_from(cap.saturating_add(1)).unwrap_or(usize::MAX);

    let mut items: Vec<T> = match strategy {
        FetchStrategy::SingleShot => {
            let limit = cap.saturating_add(1);
            fetch(1, limit).await?
        }
        FetchStrategy::Paged { per_page } => fetch_all_pages(per_page, want, &fetch).await?,
    };

    // Lossless on all supported 32/64-bit platforms: u32::MAX always fits in usize there.
    let cap_usize = usize::try_from(cap).unwrap_or(usize::MAX);
    let truncated = items.len() > cap_usize;
    if truncated {
        items.truncate(cap_usize);
    }

    Ok(Paged {
        items,
        truncated,
        limit: cap,
    })
}

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
        };
        (self.items, meta)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// 记录每次 fetch 调用的 (page, limit)，并按预设总量返回数据。
    #[derive(Debug)]
    struct Source {
        total: usize,
        calls: Mutex<Vec<(u32, u32)>>,
    }

    impl Source {
        fn new(total: usize) -> Self {
            Self {
                total,
                calls: Mutex::new(Vec::new()),
            }
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
        let paged = fetch_capped(FetchStrategy::SingleShot, cap, |page, limit| {
            let src = &src;
            async move { Ok(src.page(page, limit)) }
        })
        .await
        .expect("fetch_capped should succeed");
        let calls = src.recorded();
        (paged, calls)
    }

    async fn run_paged(total: usize, cap: u32, per_page: u32) -> (Paged<usize>, Vec<(u32, u32)>) {
        let src = Source::new(total);
        let paged = fetch_capped(FetchStrategy::Paged { per_page }, cap, |page, limit| {
            let src = &src;
            async move { Ok(src.page(page, limit)) }
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
    async fn test_should_probe_one_past_cap_when_cap_is_a_multiple_of_per_page() {
        // Discriminates a correct N+1 probe from an N probe. With `want = cap`
        // (the off-by-one bug), the loop would stop after page 2 holding exactly
        // 20 items and report truncated = false, even though 30 exist.
        let (paged, calls) = run_paged(30, 20, 10).await;
        assert_eq!(
            calls,
            vec![(1, 10), (2, 10), (3, 10)],
            "必须探到第 3 页才能发现还有更多"
        );
        assert_eq!(paged.items.len(), 20);
        assert!(paged.truncated, "30 > cap=20，必须报告截断");
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
        let result = fetch_capped(FetchStrategy::Paged { per_page: 0 }, 10, |page, limit| {
            let src = &src;
            async move { Ok(src.page(page, limit)) }
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
