//! Environment diagnostic types and traits for `gf doctor`.

use serde::Serialize;

/// Status of a single health check item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Check passed.
    Pass,
    /// Check passed with warnings.
    Warn,
    /// Check failed.
    Fail,
}

/// A single diagnostic check result.
#[derive(Debug, Clone, Serialize)]
pub struct CheckItem {
    /// Check category (e.g., `platform_cli`, `agent`, `gf_self`, `agent_env`).
    pub category: String,
    /// Check item name (e.g., "gh CLI installed").
    pub name: String,
    /// Result status.
    pub status: CheckStatus,
    /// Human-readable description (Chinese-first).
    pub message: String,
    /// Fix suggestion (provided on Fail/Warn).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Additional detail (e.g., version string, path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Summary counts for a doctor report.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorSummary {
    /// Total number of checks.
    pub total: usize,
    /// Number of passing checks.
    pub passed: usize,
    /// Number of warnings.
    pub warned: usize,
    /// Number of failures.
    pub failed: usize,
}

impl DoctorSummary {
    /// Compute summary from a slice of check items.
    #[must_use]
    pub fn from_items(items: &[CheckItem]) -> Self {
        let total = items.len();
        let passed = items
            .iter()
            .filter(|i| i.status == CheckStatus::Pass)
            .count();
        let warned = items
            .iter()
            .filter(|i| i.status == CheckStatus::Warn)
            .count();
        let failed = items
            .iter()
            .filter(|i| i.status == CheckStatus::Fail)
            .count();
        Self {
            total,
            passed,
            warned,
            failed,
        }
    }
}

/// Complete diagnostic report.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    /// All check items.
    pub items: Vec<CheckItem>,
    /// Summary counts.
    pub summary: DoctorSummary,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

impl DoctorReport {
    /// Create a report from check items, computing summary and timestamp.
    #[must_use]
    pub fn from_items(items: Vec<CheckItem>) -> Self {
        let summary = DoctorSummary::from_items(&items);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or_else(|_| "unknown".to_string(), |d| format!("{}s", d.as_secs()));
        Self {
            items,
            summary,
            timestamp,
        }
    }
}

/// Trait for health check categories.
///
/// Each category implements this trait to provide a group of related checks.
/// The `gf doctor` command iterates over all registered categories and collects results.
pub trait HealthCheck: Send + Sync {
    /// Category name for grouping (e.g., `platform_cli`).
    fn category(&self) -> &str;

    /// Run all checks in this category, returning results.
    /// Must not fail fast — collect all results even if some checks fail.
    fn run(&self) -> Vec<CheckItem>;
}

impl CheckItem {
    /// Create a passing check item.
    #[must_use]
    pub fn pass(
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
            hint: None,
            detail: None,
        }
    }

    /// Create a warning check item.
    #[must_use]
    pub fn warn(
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Warn,
            message: message.into(),
            hint: Some(hint.into()),
            detail: None,
        }
    }

    /// Create a failing check item.
    #[must_use]
    pub fn fail(
        category: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
            hint: Some(hint.into()),
            detail: None,
        }
    }

    /// Attach a detail string to this check item (builder pattern).
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_calculate_summary_counts() {
        let items = vec![
            CheckItem::pass("cat", "a", "ok"),
            CheckItem::pass("cat", "b", "ok"),
            CheckItem::warn("cat", "c", "meh", "fix it"),
            CheckItem::fail("cat", "d", "bad", "fix now"),
        ];
        let summary = DoctorSummary::from_items(&items);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_should_create_report_from_items() {
        let items = vec![
            CheckItem::pass("cat", "a", "ok"),
            CheckItem::fail("cat", "b", "bad", "fix"),
        ];
        let report = DoctorReport::from_items(items);
        assert_eq!(report.summary.total, 2);
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.summary.failed, 1);
        assert!(!report.timestamp.is_empty());
    }

    #[test]
    fn test_should_serialize_report_to_json() {
        let items = vec![
            CheckItem::pass("platform_cli", "gh installed", "gh found").with_detail("v2.65.0"),
            CheckItem::fail(
                "platform_cli",
                "gc auth",
                "not authenticated",
                "run gc auth login",
            ),
        ];
        let report = DoctorReport::from_items(items);
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"status\":\"pass\""));
        assert!(json.contains("\"status\":\"fail\""));
        assert!(json.contains("\"detail\":\"v2.65.0\""));
        // hint is present on fail item
        assert!(json.contains("gc auth login"));
    }

    #[test]
    fn test_should_skip_none_fields_in_json() {
        let item = CheckItem::pass("cat", "name", "msg");
        let json = serde_json::to_string(&item).expect("serialize");
        assert!(!json.contains("\"hint\""));
        assert!(!json.contains("\"detail\""));
    }

    #[test]
    fn test_should_create_pass_item() {
        let item = CheckItem::pass("cat", "name", "message");
        assert_eq!(item.status, CheckStatus::Pass);
        assert!(item.hint.is_none());
    }

    #[test]
    fn test_should_create_warn_item_with_hint() {
        let item = CheckItem::warn("cat", "name", "message", "hint text");
        assert_eq!(item.status, CheckStatus::Warn);
        assert_eq!(item.hint.as_deref(), Some("hint text"));
    }

    #[test]
    fn test_should_create_fail_item_with_hint() {
        let item = CheckItem::fail("cat", "name", "message", "fix this");
        assert_eq!(item.status, CheckStatus::Fail);
        assert_eq!(item.hint.as_deref(), Some("fix this"));
    }

    #[test]
    fn test_should_attach_detail_via_builder() {
        let item = CheckItem::pass("cat", "name", "msg").with_detail("v1.0.0");
        assert_eq!(item.detail.as_deref(), Some("v1.0.0"));
    }
}
