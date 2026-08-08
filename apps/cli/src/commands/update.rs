//! `gf update` 子命令实现。
//!
//! 从 GitHub Releases 检查并更新 gf binary 到最新版本。
//! 版本选择逻辑独立为纯函数，便于单测覆盖。

use semver::Version;

/// GitHub 仓库 owner。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
pub(crate) const REPO_OWNER: &str = "byx-darwin";
/// GitHub 仓库名。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
pub(crate) const REPO_NAME: &str = "gitflow-cli";
/// binary 名称。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
pub(crate) const BIN_NAME: &str = "gf";

/// 解析 semver 版本字符串（容忍前导 `v`）。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
fn parse_version(s: &str) -> Option<Version> {
    Version::parse(s.trim_start_matches('v')).ok()
}

/// 是否为预发布版本（含 `-alpha`/`-beta`/`-rc` 等 pre 标识）。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
fn is_prerelease(v: &Version) -> bool {
    !v.pre.is_empty()
}

/// 从候选版本中选择目标版本：返回大于 `current` 的最高版本。
///
/// `include_prerelease` 为 `false` 时排除预发布版本（稳定版优先）。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
fn select_target_version<'a>(
    candidates: impl Iterator<Item = &'a str>,
    current: &Version,
    include_prerelease: bool,
) -> Option<String> {
    candidates
        .filter_map(parse_version)
        .filter(|v| *v > *current && (include_prerelease || !is_prerelease(v)))
        .max()
        .map(|v| v.to_string())
}

/// 当前安装的 gf 版本（编译期注入）。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
fn current_version() -> String {
    crate::built_info::PKG_VERSION.to_string()
}

/// 当前平台目标三元组（如 `x86_64-apple-darwin`）。
#[allow(dead_code, reason = "called by follow-up task 3 command handler")]
fn target_triple() -> String {
    self_update::get_target().to_string()
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "允许在测试中使用 expect/unwrap/panic"
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_stable() {
        assert_eq!(parse_version("1.0.0"), Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn test_parse_version_strips_leading_v() {
        assert_eq!(parse_version("v1.2.3"), Some(Version::new(1, 2, 3)));
    }

    #[test]
    fn test_parse_version_prerelease() {
        let v = parse_version("1.1.0-rc.1").expect("parse rc");
        assert!(v.pre.to_string().contains("rc"));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_is_prerelease_rc_is_prerelease() {
        assert!(is_prerelease(&Version::parse("1.1.0-rc.1").expect("rc")));
    }

    #[test]
    fn test_is_prerelease_stable_is_not() {
        assert!(!is_prerelease(&Version::new(1, 1, 0)));
    }

    #[test]
    fn test_select_target_version_ignores_prerelease_by_default() {
        let candidates = ["1.0.1", "1.1.0-rc.1", "1.1.0"];
        let current = Version::new(1, 0, 0);
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, false).as_deref(),
            Some("1.1.0")
        );
    }

    #[test]
    fn test_select_target_version_includes_prerelease_with_flag() {
        let current = Version::new(1, 0, 0);
        // 稳定版更高时仍选稳定版
        let candidates = ["1.1.0-rc.1", "1.1.0"];
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, true).as_deref(),
            Some("1.1.0")
        );
        // 仅预发布更高时，--pre 选中预发布
        let candidates = ["1.1.0-rc.1", "1.0.5"];
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, true).as_deref(),
            Some("1.1.0-rc.1")
        );
    }

    #[test]
    fn test_select_target_version_none_when_up_to_date() {
        let candidates = ["0.9.0", "1.0.0"];
        let current = Version::new(1, 0, 0);
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, false),
            None
        );
    }

    #[test]
    fn test_select_target_version_skips_invalid() {
        let candidates = ["not-a-version", "", "1.0.1"];
        let current = Version::new(1, 0, 0);
        assert_eq!(
            select_target_version(candidates.into_iter(), &current, false).as_deref(),
            Some("1.0.1")
        );
    }
}
