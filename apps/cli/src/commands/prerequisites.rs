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
}

/// 平台 → CLI 要求映射。
#[must_use]
pub fn requirement_for(platform: &str) -> Option<CliRequirement> {
    match platform {
        "github" => Some(CliRequirement {
            binary: "gh",
            min_version: "2.0.0",
            install_url: "https://github.com/cli/cli#installation",
            install_hint: "brew install gh       # macOS/Linux\n\
                           choco install gh      # Windows\n\
                           sudo apt install gh  # Debian/Ubuntu",
            install_cmd: "brew install gh",
            login_cmd: "gh auth login",
            login_with_token: "echo TOKEN | gh auth login --with-token",
        }),
        "gitlab" => Some(CliRequirement {
            binary: "glab",
            min_version: "1.30.0",
            install_url: "https://gitlab.com/gitlab-org/cli#installation",
            install_hint: "brew install glab   # macOS/Linux\n\
                           sudo apt install glab  # Debian/Ubuntu",
            install_cmd: "brew install glab",
            login_cmd: "glab auth login",
            login_with_token: "glab auth login --token TOKEN",
        }),
        "gitcode" => Some(CliRequirement {
            // 跨平台统一使用 gitcode（gc 在 Windows PowerShell 是 Get-Content 别名）
            binary: "gitcode",
            min_version: "0.5.9",
            install_url: "https://gitcode.com/gitcode-cli/cli",
            install_hint: "# 方式 1 — Wheel 包（推荐，内置全平台二进制）:\n\
                           pip install https://gitcode.com/gitcode-cli/cli/releases/download/v0.5.9/gitcode_cli-0.5.9-py3-none-any.whl\n\n\
                           # 方式 2 — PyPI:\n\
                           pip install gitcode-cli\n\n\
                           # 方式 3 — Linux DEB:\n\
                           sudo dpkg -i gc_0.5.9_amd64.deb\n\n\
                           # 方式 4 — 源码构建（Go 1.22+）:\n\
                           git clone https://gitcode.com/gitcode-cli/cli.git && cd cli\n\
                           make build && mkdir -p ~/.local/bin && mv bin/gc ~/.local/bin/",
            install_cmd: "pip install gitcode-cli",
            login_cmd: "gitcode auth login",
            login_with_token: "echo TOKEN | gitcode auth login --with-token",
        }),
        _ => None,
    }
}

/// 前置检查失败错误。
#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    #[error(
        "[[PLATFORM]] {binary} is not installed.\n\n📦 Install: {install_cmd}\n\nFull \
         options:\n{install_hint}\n\n🌐 Official: {install_url}"
    )]
    NotFound {
        binary: String,
        platform: String,
        install_hint: String,
        install_url: String,
        install_cmd: String,
    },

    #[error(
        "[[PLATFORM]] {binary} v{found} is too old (need v{required}+).\n\n📦 Upgrade: \
         {install_cmd}"
    )]
    VersionTooLow {
        binary: String,
        platform: String,
        found: String,
        required: String,
        install_cmd: String,
    },

    #[error(
        "[[PLATFORM]] `{binary}` was found but ` --version` output was invalid.\n\n📦 Reinstall: \
         {install_cmd}"
    )]
    VersionParseFailed {
        binary: String,
        platform: String,
        install_cmd: String,
    },

    #[error(
        "[[PLATFORM]] {binary} is not authenticated.\n\n🔐 Interactive: {login_cmd}\n\n🔐 Paste \
         token: {login_with_token}\n\n💡 Or set env var: export GITCODE_TOKEN=your_token \
         (gitcode) / export GH_TOKEN=your_token (github)"
    )]
    NotAuthenticated {
        binary: String,
        platform: String,
        login_cmd: String,
        login_with_token: String,
    },

    #[error("Unsupported platform: {platform}. Supported: github, gitlab, gitcode")]
    UnsupportedPlatform { platform: String },
}

/// 检查原生 CLI 是否可用、版本满足要求且已登录。
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
            install_url: req.install_url.into(),
            install_cmd: req.install_cmd.into(),
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
        });
    }

    tracing::debug!(
        binary,
        found = version,
        minimum = req.min_version,
        "Version OK"
    );

    // 3. 认证检查
    if !is_authenticated(binary, platform) {
        return Err(PrerequisiteError::NotAuthenticated {
            binary: binary.into(),
            platform: platform.into(),
            login_cmd: req.login_cmd.into(),
            login_with_token: req.login_with_token.into(),
        });
    }

    tracing::debug!(binary, "Authenticated");
    Ok(())
}

/// 检测 CLI 认证状态。
fn is_authenticated(binary: &str, platform: &str) -> bool {
    // GitCode 优先检查环境变量
    if platform == "gitcode" && std::env::var("GITCODE_TOKEN").is_ok() {
        return true;
    }
    // GitHub 检查 GH_TOKEN
    if platform == "github" && std::env::var("GH_TOKEN").is_ok() {
        return true;
    }

    let args: &[&str] = match binary {
        "gh" | "glab" | "gitcode" => &["auth", "status"],
        _ => return true,
    };

    match Command::new(binary).args(args).output() {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// Try to locate and validate a GitCode CLI binary.
///
/// GitCode has two binary names (`gc` on Linux/macOS, `gitcode` cross-platform),
/// and uses `version` subcommand instead of `--version`. This function tries
/// `gc` first, then `gitcode`, returning the first one that passes version detection.
fn find_gitcode_cli(
    platform: &str,
) -> Result<(&'static str, std::path::PathBuf, String), PrerequisiteError> {
    let install_cmd = requirement_for(platform).map_or("", |r| r.install_cmd);

    for &binary in &["gitcode"] {
        // 1. 常规 PATH 搜索
        if let Ok(path) = which::which(binary) {
            if let Ok(v) = get_version(binary, platform) {
                return Ok((binary, path, v));
            }
        }

        // 2. pip 用户安装路径（macOS ~/Library/Python/X.Y/bin/）
        if let Ok(home) = std::env::var("HOME") {
            let lib = std::path::PathBuf::from(&home).join("Library/Python");
            if let Ok(entries) = std::fs::read_dir(&lib) {
                for entry in entries.flatten() {
                    let p = entry.path().join("bin").join(binary);
                    if p.exists() {
                        if let Ok(v) = get_version(&p.to_string_lossy(), platform) {
                            return Ok((binary, p, v));
                        }
                    }
                }
            }
        }
    }

    Err(PrerequisiteError::NotFound {
        binary: "gitcode".into(),
        platform: platform.into(),
        install_hint: requirement_for(platform)
            .map_or("", |r| r.install_hint)
            .into(),
        install_url: requirement_for(platform)
            .map_or("", |r| r.install_url)
            .into(),
        install_cmd: install_cmd.into(),
    })
}

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
        assert_eq!(req.binary, "gitcode");
        assert_eq!(req.min_version, "0.5.9");
        assert_eq!(req.install_cmd, "pip install gitcode-cli");
        assert_eq!(req.login_cmd, "gitcode auth login");
        assert_eq!(
            req.login_with_token,
            "echo TOKEN | gitcode auth login --with-token"
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
}
