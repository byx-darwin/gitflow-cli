//! 原生 CLI 前置检查。
//!
//! 在执行任何 gitflow 命令之前，检查目标平台对应的原生 CLI
//! （`gh`、`glab`、`gc`）是否已安装且版本满足最低要求。
//! 检查失败将阻断执行并打印安装指引。

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
    /// 常见包管理器安装命令（brew / apt / choco 等）。
    pub install_hint: &'static str,
}

/// 平台 → CLI 要求映射。
///
/// 返回 `None` 表示该平台目前不受支持（仅 github / gitlab / gitcode）。
#[must_use]
pub fn requirement_for(platform: &str) -> Option<CliRequirement> {
    match platform {
        "github" => Some(CliRequirement {
            binary: "gh",
            min_version: "2.0.0",
            install_url: "https://github.com/cli/cli#installation",
            install_hint: "brew install gh   # macOS\napt install gh    # Ubuntu\nchoco install \
                           gh  # Windows",
        }),
        "gitlab" => Some(CliRequirement {
            binary: "glab",
            min_version: "1.30.0",
            install_url: "https://gitlab.com/gitlab-org/cli#installation",
            install_hint: "brew install glab   # macOS\napt install glab    # Ubuntu",
        }),
        "gitcode" => Some(CliRequirement {
            binary: "gc",
            min_version: "0.6.0",
            install_url: "https://gitcode.com/gitcode-cli/gitcode-cli/releases",
            install_hint: "brew install gitcode-cli    # macOS (if available)\n\
                           go install gitcode.com/gitcode-cli/gc@latest   # Go toolchain\n\
                           # Or download from: https://gitcode.com/gitcode-cli/gitcode-cli/releases",
        }),
        _ => None,
    }
}

/// 前置检查失败错误。
#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    /// 原生 CLI 未在 PATH 中找到。
    #[error(
        "{binary} not found in PATH.\n\n📦 Install from: {install_url}\n\n💡 Quick \
         install:\n{install_hint}"
    )]
    NotFound {
        /// CLI 可执行文件名。
        binary: String,
        /// 快速安装命令。
        install_hint: String,
        /// 官方安装链接。
        install_url: String,
    },

    /// 原生 CLI 版本过低。
    #[error("{binary} v{required}+ required, found v{found}.\n\n💡 Upgrade:\n{install_hint}")]
    VersionTooLow {
        /// CLI 可执行文件名。
        binary: String,
        /// 检测到的版本号。
        found: String,
        /// 要求的最低版本号。
        required: String,
        /// 升级命令提示。
        install_hint: String,
    },

    /// 无法从 `--version` 输出中解析 semver。
    #[error(
        "Failed to parse `{binary} --version` output — {binary} may not be the expected \
         {platform} CLI.\n\n📦 Install the correct CLI from: {install_url}\n\n💡 Quick \
         install:\n{install_hint}"
    )]
    VersionParseFailed {
        /// CLI 可执行文件名。
        binary: String,
        /// 对应的平台名称。
        platform: String,
        /// 快速安装命令。
        install_hint: String,
        /// 官方安装链接。
        install_url: String,
    },

    /// 不支持的平台。
    #[error("Unsupported platform: {platform}. Supported platforms: github, gitlab, gitcode")]
    UnsupportedPlatform {
        /// 不支持的平台标识符。
        platform: String,
    },
}

/// 检查原生 CLI 是否可用且满足版本要求。
///
/// 流程：
/// 1. 通过 `which` 检查 CLI 是否在 PATH 上。
/// 2. 执行 `<cli> --version` 获取版本号。
/// 3. 比较版本是否 ≥ 最低要求。
///
/// # Errors
///
/// - CLI 不在 PATH 上 → `PrerequisiteError::NotFound`
/// - CLI 版本过低 → `PrerequisiteError::VersionTooLow`
/// - 版本解析失败 → `PrerequisiteError::VersionParseFailed`
/// - 平台不受支持 → `PrerequisiteError::UnsupportedPlatform`
pub fn check(platform: &str) -> Result<(), PrerequisiteError> {
    let req = requirement_for(platform).ok_or_else(|| PrerequisiteError::UnsupportedPlatform {
        platform: platform.into(),
    })?;

    // 1. 检查 CLI 是否在 PATH 上
    let path = which::which(req.binary).map_err(|_| PrerequisiteError::NotFound {
        binary: req.binary.into(),
        install_hint: req.install_hint.into(),
        install_url: req.install_url.into(),
    })?;

    tracing::debug!(binary = req.binary, path = %path.display(), "Found native CLI binary");

    // 2. 检查版本
    let version = get_version(req.binary)?;
    if !version_meets_minimum(&version, req.min_version) {
        return Err(PrerequisiteError::VersionTooLow {
            binary: req.binary.into(),
            found: version,
            required: req.min_version.into(),
            install_hint: req.install_hint.into(),
        });
    }

    tracing::debug!(
        binary = req.binary,
        found = version,
        minimum = req.min_version,
        "Native CLI version meets minimum requirement"
    );
    Ok(())
}

/// 执行 `<binary> --version` 并解析其中的 semver 字符串。
///
/// # Errors
///
/// 执行失败或输出中无 semver 模式时返回 `PrerequisiteError::VersionParseFailed`。
fn get_version(binary: &str) -> Result<String, PrerequisiteError> {
    let find_hint = || -> (&str, &str) {
        requirement_for(binary_to_platform(binary))
            .map(|r| (r.install_hint, r.install_url))
            .unwrap_or(("", ""))
    };

    let output = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|_| {
            let (hint, url) = find_hint();
            PrerequisiteError::VersionParseFailed {
                binary: binary.into(),
                platform: binary_to_platform(binary).into(),
                install_hint: hint.into(),
                install_url: url.into(),
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_semver(&stdout).ok_or_else(|| {
        let (hint, url) = find_hint();
        PrerequisiteError::VersionParseFailed {
            binary: binary.into(),
            platform: binary_to_platform(binary).into(),
            install_hint: hint.into(),
            install_url: url.into(),
        }
    })
}

/// Map a binary name to its platform name for error messages.
fn binary_to_platform(binary: &str) -> &'static str {
    match binary {
        "gh" => "github",
        "glab" => "gitlab",
        "gc" => "gitcode",
        _ => "unknown",
    }
}

/// 从字符串中提取第一个 semver 模式（`X.Y.Z`）。
///
/// 返回找到的版本号字符串，若未找到则返回 `None`。
#[must_use]
pub fn extract_semver(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").ok()?;
    re.find(s).map(|m| m.as_str().to_owned())
}

/// 检查 `found` 版本是否 ≥ `minimum` 版本。
///
/// 将版本号按 `.` 分割为 `u32` 段后进行字典序比较，
/// 与标准 semver 语义一致（major → minor → patch）。
#[must_use]
pub fn version_meets_minimum(found: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    let found_parts = parse(found);
    let min_parts = parse(minimum);
    // 字典序比较对 semver 语义正确：major 优先，其次 minor，最后 patch。
    found_parts.cmp(&min_parts) != std::cmp::Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_requirement_for_github() {
        let req = requirement_for("github");
        assert!(req.is_some());
        let req = req.unwrap_or_else(|| unreachable!("already checked is_some"));
        assert_eq!(req.binary, "gh");
        assert_eq!(req.min_version, "2.0.0");
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
