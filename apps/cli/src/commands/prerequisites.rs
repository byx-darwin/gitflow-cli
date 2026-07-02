//! 原生 CLI 前置检查。
//!
//! 在执行任何 gitflow 命令之前，检查目标平台对应的原生 CLI
//! （`gh`、`glab`、`gc`）是否已安装、版本满足最低要求，以及是否已登录。
//! 检查失败将阻断执行并打印安装/登录指引。
//!
//! 错误消息包含 Agent 可解析的标记：
//! - `[[INSTALL_COMMAND]]` — Agent 可直接运行的单一安装命令
//! - `[[LOGIN_COMMAND]]` — Agent 提示用户在终端输入 token 的登录命令
//! - `[[LOGIN_WITH_TOKEN]]` — Agent 可通过 stdin 或 flag 传入 token 的登录命令

// The prerequisite check runs synchronously before the tokio runtime is
// created, so `std::process::Command` is appropriate here.
#![allow(
    clippy::disallowed_types,
    reason = "Pre-runtime sync `Command` invocations for version probing"
)]

use std::process::Command;

/// 原生 CLI 版本要求。
#[derive(Debug, Clone)]
pub struct CliRequirement {
    /// CLI 可执行文件名。
    pub binary: &'static str,
    /// 最低版本号（semver）。
    pub min_version: &'static str,
    /// 官方安装指引链接。
    pub install_url: &'static str,
    /// 常见包管理器安装命令。
    pub install_hint: &'static str,
    /// Agent 可直接执行的一键安装命令。
    pub install_cmd: &'static str,
    /// 交互式登录命令。
    pub login_cmd: &'static str,
    /// 非交互式登录命令（可传 token）。
    pub login_with_token: &'static str,
}

/// 平台 → CLI 要求映射。
#[must_use]
pub fn requirement_for(platform: &str) -> Option<CliRequirement> {
    match platform {
        "github" => Some(CliRequirement {
            binary: "gh",
            min_version: "2.0.0",
            install_url: "https://github.com/cli/cli#installation",
            install_hint: "brew install gh   # macOS/Linux\nchoco install gh  # Windows\nsudo apt \
                           install gh  # Debian/Ubuntu (via gh PPA)",
            install_cmd: "brew install gh",
            login_cmd: "gh auth login",
            login_with_token: "echo TOKEN | gh auth login --with-token",
        }),
        "gitlab" => Some(CliRequirement {
            binary: "glab",
            min_version: "1.30.0",
            install_url: "https://gitlab.com/gitlab-org/cli#installation",
            install_hint: "brew install glab   # macOS/Linux\nsudo apt install glab  # \
                           Debian/Ubuntu",
            install_cmd: "brew install glab",
            login_cmd: "glab auth login",
            login_with_token: "glab auth login --token TOKEN",
        }),
        "gitcode" => Some(CliRequirement {
            binary: "gc",
            min_version: "0.6.0",
            install_url: "https://gitcode.com/gitcode-cli/gitcode-cli/releases",
            install_hint: "brew install gitcode-cli  # macOS/Linux\ngo install \
                           gitcode.com/gitcode-cli/gc@latest  # Go",
            install_cmd: "brew install gitcode-cli",
            login_cmd: "gc auth login",
            login_with_token: "gc auth login --token TOKEN",
        }),
        _ => None,
    }
}

/// 前置检查失败错误。
#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    /// 原生 CLI 未在 PATH 中找到。
    #[error(
        "[[PLATFORM]] {binary} is not installed.\n\n📦 Install command: [[INSTALL_COMMAND]] \
         ({install_cmd})\n\nFull install options:\n{install_hint}\n\n🌐 Official docs: \
         {install_url}"
    )]
    NotFound {
        binary: String,
        platform: String,
        install_hint: String,
        install_url: String,
        install_cmd: String,
    },

    /// 原生 CLI 版本过低。
    #[error(
        "[[PLATFORM]] {binary} v{found} is too old (need v{required}+).\n\n📦 Upgrade command: \
         [[INSTALL_COMMAND]] ({install_cmd})"
    )]
    VersionTooLow {
        binary: String,
        platform: String,
        found: String,
        required: String,
        install_cmd: String,
    },

    /// 无法从 `--version` 输出中解析 semver。
    #[error(
        "[[PLATFORM]] `{binary}` was found but does not appear to be the correct CLI.\nThe \
         `{binary} --version` output did not contain a valid semver.\n\n📦 Reinstall: \
         [[INSTALL_COMMAND]] ({install_cmd})"
    )]
    VersionParseFailed {
        binary: String,
        platform: String,
        install_cmd: String,
    },

    /// 原生 CLI 未认证。
    #[error(
        "[[PLATFORM]] {binary} is not authenticated.\n\n🔐 Login (interactive): {login_cmd}\n   \
         (opens a browser or prompts for token)\n\n🔐 Login (paste token): {login_with_token}\n   \
         (get a token from your platform's settings)\n\n💡 Tip: Copy your Personal Access Token \
         from the platform's web UI and run the\nlogin-with-token command, replacing TOKEN with \
         your actual token."
    )]
    NotAuthenticated {
        binary: String,
        platform: String,
        login_cmd: String,
        login_with_token: String,
    },

    /// 不支持的平台。
    #[error("Unsupported platform: {platform}. Supported platforms: github, gitlab, gitcode")]
    UnsupportedPlatform { platform: String },
}

/// 检查原生 CLI 是否可用、版本满足要求且已登录。
///
/// 流程：
/// 1. `which` 检查 CLI 是否在 PATH → `NotFound`
/// 2. `--version` 检查版本 → `VersionTooLow` / `VersionParseFailed`
/// 3. `auth status` 检查认证 → `NotAuthenticated`
pub fn check(platform: &str) -> Result<(), PrerequisiteError> {
    let req = requirement_for(platform).ok_or_else(|| PrerequisiteError::UnsupportedPlatform {
        platform: platform.into(),
    })?;

    // 1. PATH 检查
    let path = which::which(req.binary).map_err(|_| PrerequisiteError::NotFound {
        binary: req.binary.into(),
        platform: platform.into(),
        install_hint: req.install_hint.into(),
        install_url: req.install_url.into(),
        install_cmd: req.install_cmd.into(),
    })?;

    tracing::debug!(binary = req.binary, path = %path.display(), "Found native CLI");

    // 2. 版本检查
    let version = get_version(req.binary, platform)?;
    if !version_meets_minimum(&version, req.min_version) {
        return Err(PrerequisiteError::VersionTooLow {
            binary: req.binary.into(),
            platform: platform.into(),
            found: version,
            required: req.min_version.into(),
            install_cmd: req.install_cmd.into(),
        });
    }

    tracing::debug!(
        binary = req.binary,
        found = version,
        minimum = req.min_version,
        "Version OK"
    );

    // 3. 认证检查
    if !is_authenticated(req.binary)? {
        return Err(PrerequisiteError::NotAuthenticated {
            binary: req.binary.into(),
            platform: platform.into(),
            login_cmd: req.login_cmd.into(),
            login_with_token: req.login_with_token.into(),
        });
    }

    tracing::debug!(binary = req.binary, "Authenticated");
    Ok(())
}

/// Execute `<binary> --version` and extract semver.
fn get_version(binary: &str, platform: &str) -> Result<String, PrerequisiteError> {
    let install_cmd = requirement_for(platform).map_or("", |r| r.install_cmd);

    let output = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|_| PrerequisiteError::VersionParseFailed {
            binary: binary.into(),
            platform: platform.into(),
            install_cmd: install_cmd.into(),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_semver(&stdout).ok_or_else(|| PrerequisiteError::VersionParseFailed {
        binary: binary.into(),
        platform: platform.into(),
        install_cmd: install_cmd.into(),
    })
}

/// Check whether the native CLI is authenticated.
fn is_authenticated(binary: &str) -> Result<bool, PrerequisiteError> {
    let output = match binary {
        "gh" => Command::new(binary).args(["auth", "status"]).output(),
        "glab" => Command::new(binary).args(["auth", "status"]).output(),
        "gc" => Command::new(binary).args(["auth", "status"]).output(),
        _ => return Ok(true),
    };
    match output {
        Ok(out) if out.status.success() => Ok(true),
        _ => Ok(false),
    }
}

/// Extract first semver (`X.Y.Z`) from a string.
#[must_use]
pub fn extract_semver(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").ok()?;
    re.find(s).map(|m| m.as_str().to_owned())
}

/// Check `found` version ≥ `minimum`.
#[must_use]
pub fn version_meets_minimum(found: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    let found_parts = parse(found);
    let min_parts = parse(minimum);
    found_parts.cmp(&min_parts) != std::cmp::Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_requirement_for_github() {
        let req = requirement_for("github").expect("github requirement");
        assert_eq!(req.binary, "gh");
        assert_eq!(req.min_version, "2.0.0");
        assert_eq!(req.install_cmd, "brew install gh");
        assert_eq!(req.login_cmd, "gh auth login");
        assert_eq!(
            req.login_with_token,
            "echo TOKEN | gh auth login --with-token"
        );
    }

    #[test]
    fn test_should_extract_semver_from_gh_version_output() {
        let output = "gh version 2.50.0 (2024-01-01)";
        let version = extract_semver(output);
        assert_eq!(version.as_deref(), Some("2.50.0"));
    }

    #[test]
    fn test_should_extract_semver_from_glab_version_output() {
        let output = "glab version 1.35.0 (2024-01-01)";
        let version = extract_semver(output);
        assert_eq!(version.as_deref(), Some("1.35.0"));
    }

    #[test]
    fn test_should_version_meets_minimum_pass() {
        assert!(version_meets_minimum("2.50.0", "2.0.0"));
    }

    #[test]
    fn test_should_version_meets_minimum_fail() {
        assert!(!version_meets_minimum("1.9.0", "2.0.0"));
    }

    #[test]
    fn test_should_version_meets_minimum_equal() {
        assert!(version_meets_minimum("2.0.0", "2.0.0"));
    }
}
