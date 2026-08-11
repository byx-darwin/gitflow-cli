//! `gf doctor` — environment diagnostic command.
//!
//! Provides a comprehensive health check covering:
//! - Platform CLI status (`gh`, `glab`, `gc`)
//! - Agent platform and skills
//! - `gf` binary self-check
//! - Agent runtime environment

use gitflow_core::doctor::{CheckItem, HealthCheck};

/// Checks all three platform CLIs (`gh`, `glab`, `gc`) for installation, version, and auth.
///
/// Unlike `prerequisites::check()` which fast-fails on the target platform,
/// this check collects results for ALL platforms.
#[allow(dead_code, reason = "consumed by handle() added in subsequent commit")]
pub struct PlatformCliCheck;

#[allow(
    clippy::needless_lifetimes,
    reason = "Trait method signature uses elided lifetimes; impl must match"
)]
impl HealthCheck for PlatformCliCheck {
    fn category(&self) -> &'static str {
        "platform_cli"
    }

    fn run(&self) -> Vec<CheckItem> {
        let platforms = ["github", "gitlab", "gitcode"];
        let labels = ["GitHub", "GitLab", "GitCode"];
        let mut items = Vec::new();

        for (platform, label) in platforms.iter().zip(labels.iter()) {
            let Some(req) = super::prerequisites::requirement_for(platform) else {
                items.push(CheckItem::fail(
                    self.category(),
                    format!("{label} CLI"),
                    format!("{label} 平台配置缺失"),
                    "请报告此问题",
                ));
                continue;
            };

            // Check binary existence
            let binary_path = if *platform == "gitcode" {
                find_gitcode_binary()
            } else {
                which::which(req.binary)
                    .ok()
                    .map(|p| (p, req.binary.to_string()))
            };

            let Some((_, ref binary_name)) = binary_path else {
                items.push(CheckItem::fail(
                    self.category(),
                    format!("{label} CLI"),
                    format!("{label} {} 未安装", req.binary),
                    format!("安装：{}", req.install_cmd),
                ));
                continue;
            };

            // Check version
            let version = get_cli_version(binary_name);
            let version_str = version.as_deref().unwrap_or("unknown");

            if let Some(ref v) = version
                && !super::prerequisites::version_meets_minimum(v, req.min_version)
            {
                items.push(
                    CheckItem::warn(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} {} 版本过低", req.binary),
                        format!("升级：{}", req.install_cmd),
                    )
                    .with_detail(format!("当前 v{v}，需要 v{}", req.min_version)),
                );
                continue;
            }

            // Check auth
            let auth_checker = create_auth_checker_for(platform);
            if auth_checker.is_authenticated() {
                let check_result = auth_checker.check_status();
                let user_info = check_result.user.as_deref().unwrap_or("unknown");
                items.push(
                    CheckItem::pass(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} {} 已认证", req.binary),
                    )
                    .with_detail(format!("v{version_str} ({user_info})")),
                );
            } else {
                let check_result = auth_checker.check_status();
                let hint = check_result.hint.unwrap_or(req.login_cmd.to_string());
                items.push(
                    CheckItem::fail(
                        self.category(),
                        format!("{label} CLI"),
                        format!("{label} 未认证"),
                        format!("运行 `{hint}` 完成登录"),
                    )
                    .with_detail(format!("v{version_str}")),
                );
            }
        }

        items
    }
}

/// Find `GitCode` CLI binary (tries `gc` then `gitcode`, including pip paths).
#[allow(
    dead_code,
    clippy::disallowed_methods,
    reason = "consumed by handle() added in subsequent commit; sync binary discovery before async \
              runtime"
)]
fn find_gitcode_binary() -> Option<(std::path::PathBuf, String)> {
    for binary in &["gc", "gitcode"] {
        if let Ok(path) = which::which(binary) {
            return Some((path, (*binary).to_string()));
        }
    }
    // Check pip user install paths
    if let Ok(home) = std::env::var("HOME") {
        let lib = std::path::PathBuf::from(&home).join("Library/Python");
        if let Ok(entries) = std::fs::read_dir(&lib) {
            for entry in entries.flatten() {
                for binary in &["gc", "gitcode"] {
                    let p = entry.path().join("bin").join(binary);
                    if p.exists() {
                        return Some((p, (*binary).to_string()));
                    }
                }
            }
        }
    }
    None
}

/// Get CLI version string by running `<binary> --version` or `<binary> version`.
#[allow(
    dead_code,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "consumed by handle() added in subsequent commit; sync CLI version probe before \
              async runtime"
)]
fn get_cli_version(binary: &str) -> Option<String> {
    for arg in &["--version", "version"] {
        if let Ok(output) = std::process::Command::new(binary).arg(arg).output()
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(v) = super::prerequisites::extract_semver(&stdout) {
                return Some(v);
            }
        }
    }
    None
}

/// Create an auth checker for the given platform.
#[allow(dead_code, reason = "consumed by handle() added in subsequent commit")]
fn create_auth_checker_for(platform: &str) -> Box<dyn gitflow_core::AuthChecker> {
    match platform {
        "github" => Box::new(gitflow_github::GitHubAuthProvider::new()),
        "gitlab" => Box::new(gitflow_gitlab::GitLabAuthProvider::new()),
        "gitcode" => Box::new(gitflow_gitcode::GitCodeAuthProvider::new()),
        _ => unreachable!("platform already validated"),
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Tests legitimately need to unwrap fixture data"
)]
mod tests {
    use gitflow_core::doctor::CheckStatus;

    use super::*;

    #[test]
    fn test_should_return_platform_cli_category() {
        let check = PlatformCliCheck;
        assert_eq!(check.category(), "platform_cli");
    }

    #[test]
    fn test_should_collect_results_for_all_three_platforms() {
        let check = PlatformCliCheck;
        let items = check.run();
        assert_eq!(
            items.len(),
            3,
            "Expected 3 items (one per platform), got {}",
            items.len()
        );
    }

    #[test]
    fn test_should_not_fast_fail_on_missing_cli() {
        let check = PlatformCliCheck;
        let items = check.run();
        assert_eq!(items.len(), 3);
        for item in &items {
            assert!(
                matches!(
                    item.status,
                    CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Fail
                ),
                "Invalid status for {}: {:?}",
                item.name,
                item.status
            );
        }
    }

    #[test]
    fn test_should_include_install_hint_on_failure() {
        let check = PlatformCliCheck;
        let items = check.run();
        for item in &items {
            if item.status == CheckStatus::Fail && item.message.contains("未安装") {
                assert!(
                    item.hint.is_some(),
                    "Missing install hint for failed check: {}",
                    item.name
                );
            }
        }
    }

    #[test]
    fn test_should_get_cli_version_for_installed_binary() {
        let version = get_cli_version("git");
        assert!(version.is_some(), "git version should be detectable");
    }

    #[test]
    fn test_should_return_none_for_nonexistent_binary_version() {
        let version = get_cli_version("nonexistent-binary-xyz-12345");
        assert!(version.is_none());
    }
}
