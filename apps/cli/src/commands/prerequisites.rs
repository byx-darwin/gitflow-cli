//! 原生 CLI 前置检查。
//!
//! 在执行任何 gitflow 命令之前，检查目标平台对应的原生 CLI
//! 是否已安装、版本满足最低要求，以及是否已登录。
//!
//! 错误消息包含 Agent 可解析的标记：
//! - `[[INSTALL_COMMAND]]` — Agent 可直接运行的单一安装命令
//! - `[[LOGIN_COMMAND]]` — Agent 提示用户在终端输入 token 的登录命令
//! - `[[LOGIN_WITH_TOKEN]]` — Agent 可通过 stdin 传入 token 的登录命令

#![allow(
    clippy::disallowed_types,
    reason = "Pre-runtime sync `Command` invocations for version probing"
)]

use std::process::Command;

/// 原生 CLI 版本要求。
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "Fields reserved for future use")]
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
    /// 非交互式登录命令（从 stdin 读取 token）。
    pub login_with_token: &'static str,
    /// 相关文档链接。
    pub doc_link: &'static str,
}

/// 平台 → CLI 要求映射。
#[must_use]
pub fn requirement_for(platform: &str) -> Option<CliRequirement> {
    match platform {
        "github" => Some(CliRequirement {
            binary: "gh",
            // gh 2.0+ provides `gh api`, `gh pr create/list/view/merge/close`,
            // `gh issue create/list/view`, and `gh release create/list/view`.
            // See docs/cli-compatibility.md for feature-level version breakdown.
            min_version: "2.0.0",
            install_url: "https://github.com/cli/cli#installation",
            install_hint: "brew install gh       # macOS/Linux\n\
                           choco install gh      # Windows\n\
                           sudo apt install gh  # Debian/Ubuntu",
            install_cmd: "brew install gh",
            login_cmd: "gh auth login",
            login_with_token: "echo TOKEN | gh auth login --with-token",
            doc_link: "https://cli.github.com/manual/",
        }),
        "gitlab" => Some(CliRequirement {
            binary: "glab",
            // glab 1.30+ provides `glab mr create/list/view/merge/close`,
            // `glab issue create/list/view`, `glab release create/list/view`,
            // and `glab ci list/trace`. See docs/cli-compatibility.md for details.
            min_version: "1.30.0",
            install_url: "https://gitlab.com/gitlab-org/cli#installation",
            install_hint: "brew install glab   # macOS/Linux\n\
                           sudo apt install glab  # Debian/Ubuntu",
            install_cmd: "brew install glab",
            login_cmd: "glab auth login",
            login_with_token: "glab auth login --token TOKEN",
            doc_link: "https://gitlab.com/gitlab-org/cli/-/blob/main/docs/",
        }),
        "gitcode" => Some(CliRequirement {
            // 优先使用 gc（Linux/macOS 原生名称），gitcode 作为回退。
            // gc 0.6+ provides `issue create/list/view/close/reopen`,
            // `pr create/list/view/merge/close/checkout`, `release create/list/view/edit/delete`,
            // `label create/list/edit/delete/view`, and `milestone create/list/edit/close/reopen`.
            // See docs/cli-compatibility.md for feature-level version breakdown.
            binary: "gc",
            min_version: "0.6.0",
            install_url: "https://gitcode.com/gitcode-cli/cli",
            install_hint: "# 方式 1 — Wheel 包（推荐，内置全平台二进制）:\n\
                           pip install https://gitcode.com/gitcode-cli/cli/releases/download/v0.6.1/gitcode_cli-0.6.1-py3-none-any.whl\n\n\
                           # 方式 2 — PyPI:\n\
                           pip install gitcode-cli\n\n\
                           # 方式 3 — Linux DEB:\n\
                           sudo dpkg -i gitcode_0.6.1_amd64.deb\n\n\
                           # 方式 4 — 源码构建（Go 1.22+）:\n\
                           git clone https://gitcode.com/gitcode-cli/cli.git && cd cli\n\
                           make build && mkdir -p ~/.local/bin && mv bin/gitcode ~/.local/bin/",
            install_cmd: "pip install gitcode-cli",
            login_cmd: "gc auth login",
            login_with_token: "echo TOKEN | gc auth login --with-token",
            doc_link: "https://gitcode.com/gitcode-cli/cli/blob/main/README.md",
        }),
        _ => None,
    }
}

/// 格式化平台名称用于显示。
fn format_platform(platform: &str) -> String {
    match platform {
        "github" => "[GitHub]".to_string(),
        "gitlab" => "[GitLab]".to_string(),
        "gitcode" => "[GitCode]".to_string(),
        other => format!("[{other}]"),
    }
}

/// 前置检查失败错误。
#[derive(Debug)]
pub enum PrerequisiteError {
    /// 底层 CLI 未安装。
    NotFound {
        /// CLI 可执行文件名。
        binary: String,
        /// 平台标识。
        platform: String,
        /// 安装选项。
        install_hint: String,
        /// 一键安装命令。
        install_cmd: String,
        /// 文档链接。
        doc_link: String,
    },

    /// 底层 CLI 版本过低。
    VersionTooLow {
        /// CLI 可执行文件名。
        binary: String,
        /// 平台标识。
        platform: String,
        /// 当前版本。
        found: String,
        /// 最低要求版本。
        required: String,
        /// 升级命令。
        install_cmd: String,
        /// 文档链接。
        doc_link: String,
    },

    /// 版本信息解析失败。
    VersionParseFailed {
        /// CLI 可执行文件名。
        binary: String,
        /// 平台标识。
        platform: String,
        /// 安装命令。
        install_cmd: String,
        /// 文档链接。
        doc_link: String,
    },

    /// 未认证。
    NotAuthenticated {
        /// CLI 可执行文件名。
        binary: String,
        /// 平台标识。
        platform: String,
        /// 失败原因。
        reason: String,
        /// 修复命令。
        hint: String,
    },

    /// 不支持的平台。
    UnsupportedPlatform {
        /// 平台标识。
        platform: String,
    },
}

impl std::fmt::Display for PrerequisiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound {
                binary,
                platform,
                install_cmd,
                doc_link,
                install_hint,
                ..
            } => write!(
                f,
                "{} 未检测到 {binary}。\n\n📦 安装：{install_cmd}\n📖 \
                 文档：{doc_link}\n\n其他安装方式：\n{install_hint}",
                format_platform(platform)
            ),
            Self::VersionTooLow {
                binary,
                platform,
                found,
                required,
                install_cmd,
                doc_link,
            } => write!(
                f,
                "{} {binary} 版本过低：当前 v{found}，需要 v{required}+。\n\n📦 \
                 升级：{install_cmd}\n📖 文档：{doc_link}",
                format_platform(platform)
            ),
            Self::VersionParseFailed {
                binary,
                platform,
                install_cmd,
                doc_link,
            } => write!(
                f,
                "{} {binary} 版本信息解析失败。\n\n📦 重新安装：{install_cmd}\n📖 文档：{doc_link}",
                format_platform(platform)
            ),
            Self::NotAuthenticated {
                binary,
                platform,
                reason,
                hint,
            } => write!(
                f,
                "{} {binary} 未认证。\n\n🔍 原因：{reason}\n🔧 修复：运行 `{hint}` 完成登录",
                format_platform(platform)
            ),
            Self::UnsupportedPlatform { platform } => {
                write!(
                    f,
                    "不支持的平台：{platform}。支持的平台：github、gitlab、gitcode"
                )
            }
        }
    }
}

impl std::error::Error for PrerequisiteError {}

/// 检查原生 CLI 是否可用、版本满足要求且已登录。
#[allow(
    clippy::result_large_err,
    reason = "PrerequisiteError carries structured install hints; boxing would lose ergonomic \
              matching"
)]
pub fn check(platform: &str) -> Result<(), PrerequisiteError> {
    let req = requirement_for(platform).ok_or_else(|| PrerequisiteError::UnsupportedPlatform {
        platform: platform.into(),
    })?;

    // 1. PATH 检查（gitcode 平台会搜索 pip 路径等非标准位置）
    let (binary, path, version) = if platform == "gitcode" {
        find_gitcode_cli(platform)?
    } else {
        let path = which::which(req.binary).map_err(|_| PrerequisiteError::NotFound {
            binary: req.binary.into(),
            platform: platform.into(),
            install_hint: req.install_hint.into(),
            install_cmd: req.install_cmd.into(),
            doc_link: req.doc_link.into(),
        })?;
        let version = get_version(req.binary, platform)?;
        (req.binary, path, version)
    };

    tracing::debug!(binary, path = %path.display(), "Found native CLI");

    // 2. 版本检查
    if !version_meets_minimum(&version, req.min_version) {
        return Err(PrerequisiteError::VersionTooLow {
            binary: binary.into(),
            platform: platform.into(),
            found: version,
            required: req.min_version.into(),
            install_cmd: req.install_cmd.into(),
            doc_link: req.doc_link.into(),
        });
    }

    tracing::debug!(
        binary,
        found = version,
        minimum = req.min_version,
        "Version OK"
    );

    // 3. 认证检查（使用 AuthChecker）
    let auth_checker = create_auth_checker(platform);
    if !auth_checker.is_authenticated() {
        let result = auth_checker.check_status();
        return Err(PrerequisiteError::NotAuthenticated {
            binary: binary.into(),
            platform: platform.into(),
            reason: result.reason.unwrap_or_else(|| "未知原因".into()),
            hint: result.hint.unwrap_or_else(|| req.login_cmd.into()),
        });
    }

    tracing::debug!(binary, "Authenticated");
    Ok(())
}

/// 创建平台特定的认证检查器。
fn create_auth_checker(platform: &str) -> Box<dyn gitflow_core::AuthChecker> {
    match platform {
        "github" => Box::new(gitflow_github::GitHubAuthProvider::new()),
        "gitlab" => Box::new(gitflow_gitlab::GitLabAuthProvider::new()),
        "gitcode" => Box::new(gitflow_gitcode::GitCodeAuthProvider::new()),
        _ => unreachable!("Platform already validated by requirement_for"),
    }
}

/// Try to locate and validate a `GitCode` CLI binary.
///
/// `GitCode` has two binary names (`gc` on Linux/macOS, `gitcode` cross-platform).
/// This function tries `gc` first, then `gitcode`, returning the first one that
/// passes version detection.
#[allow(
    clippy::disallowed_methods,
    reason = "binary discovery runs at startup before async runtime is ready"
)]
#[allow(
    clippy::result_large_err,
    reason = "Same PrerequisiteError size as check()"
)]
fn find_gitcode_cli(
    platform: &str,
) -> Result<(&'static str, std::path::PathBuf, String), PrerequisiteError> {
    let install_cmd = requirement_for(platform).map_or("", |r| r.install_cmd);

    for &binary in &["gc", "gitcode"] {
        // 1. 常规 PATH 搜索
        if let Ok(path) = which::which(binary)
            && let Ok(v) = get_version(binary, platform)
        {
            return Ok((binary, path, v));
        }

        // 2. pip 用户安装路径（macOS ~/Library/Python/X.Y/bin/）
        if let Ok(home) = std::env::var("HOME") {
            let lib = std::path::PathBuf::from(&home).join("Library/Python");
            if let Ok(entries) = std::fs::read_dir(&lib) {
                for entry in entries.flatten() {
                    let p = entry.path().join("bin").join(binary);
                    if p.exists()
                        && let Ok(v) = get_version(&p.to_string_lossy(), platform)
                    {
                        return Ok((binary, p, v));
                    }
                }
            }
        }
    }

    Err(PrerequisiteError::NotFound {
        binary: "gc".into(),
        platform: platform.into(),
        install_hint: requirement_for(platform)
            .map_or("", |r| r.install_hint)
            .into(),
        install_cmd: install_cmd.into(),
        doc_link: requirement_for(platform).map_or("", |r| r.doc_link).into(),
    })
}

#[allow(
    clippy::result_large_err,
    reason = "Same PrerequisiteError size as check()"
)]
fn get_version(binary: &str, platform: &str) -> Result<String, PrerequisiteError> {
    let install_cmd = requirement_for(platform).map_or("", |r| r.install_cmd);

    // 尝试两种版本命令：`--version` flag（gh/glab）和 `version` 子命令（gitcode）
    for version_arg in ["--version", "version"] {
        let output = match Command::new(binary).arg(version_arg).output() {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(v) = extract_semver(&stdout) {
            return Ok(v);
        }
    }

    Err(PrerequisiteError::VersionParseFailed {
        binary: binary.into(),
        platform: platform.into(),
        install_cmd: install_cmd.into(),
        doc_link: requirement_for(platform).map_or("", |r| r.doc_link).into(),
    })
}

#[must_use]
pub fn extract_semver(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d+\.\d+\.\d+").ok()?;
    re.find(s).map(|m| m.as_str().to_owned())
}

#[must_use]
pub fn version_meets_minimum(found: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };
    parse(found).cmp(&parse(minimum)) != std::cmp::Ordering::Less
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, reason = "test code")]
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
    fn test_should_return_requirement_for_gitcode() {
        let req = requirement_for("gitcode").expect("gitcode requirement");
        assert_eq!(req.binary, "gc");
        assert_eq!(req.min_version, "0.6.0");
        assert_eq!(req.install_cmd, "pip install gitcode-cli");
        assert_eq!(req.login_cmd, "gc auth login");
        assert_eq!(
            req.login_with_token,
            "echo TOKEN | gc auth login --with-token"
        );
    }

    #[test]
    fn test_should_extract_semver_from_gh_version_output() {
        assert_eq!(
            extract_semver("gh version 2.50.0 (2024-01-01)").as_deref(),
            Some("2.50.0")
        );
    }

    #[test]
    fn test_should_extract_semver_from_glab_version_output() {
        assert_eq!(
            extract_semver("glab version 1.35.0 (2024-01-01)").as_deref(),
            Some("1.35.0")
        );
    }

    #[test]
    fn test_should_version_meets_minimum() {
        assert!(version_meets_minimum("2.50.0", "2.0.0"));
        assert!(version_meets_minimum("2.0.0", "2.0.0"));
        assert!(!version_meets_minimum("1.9.0", "2.0.0"));
    }

    #[test]
    fn test_should_display_github_in_not_authenticated_error() {
        let err = PrerequisiteError::NotAuthenticated {
            binary: "gh".into(),
            platform: "github".into(),
            reason: "token expired".into(),
            hint: "gh auth login".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[GitHub]"), "Expected [GitHub] in: {msg}");
        assert!(
            !msg.contains("[[PLATFORM]]"),
            "Found literal [[PLATFORM]] in: {msg}"
        );
    }

    #[test]
    fn test_should_display_gitlab_in_not_found_error() {
        let err = PrerequisiteError::NotFound {
            binary: "glab".into(),
            platform: "gitlab".into(),
            install_hint: "brew install glab".into(),
            install_cmd: "brew install glab".into(),
            doc_link: "https://docs.gitlab.com".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[GitLab]"), "Expected [GitLab] in: {msg}");
        assert!(
            !msg.contains("[[PLATFORM]]"),
            "Found literal [[PLATFORM]] in: {msg}"
        );
    }

    #[test]
    fn test_should_display_gitcode_in_version_too_low_error() {
        let err = PrerequisiteError::VersionTooLow {
            binary: "gc".into(),
            platform: "gitcode".into(),
            found: "0.5.0".into(),
            required: "0.6.0".into(),
            install_cmd: "pip install gitcode-cli".into(),
            doc_link: "https://gitcode.com/cli".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[GitCode]"), "Expected [GitCode] in: {msg}");
        assert!(
            !msg.contains("[[PLATFORM]]"),
            "Found literal [[PLATFORM]] in: {msg}"
        );
    }
}
