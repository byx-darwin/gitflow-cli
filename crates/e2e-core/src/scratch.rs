//! 临时 git 仓库构造模块
//!
//! 为需要"remote 指向真实目标仓库"的实测(GitLab/GitCode 的 issue/pr 场景)
//! 提供一次性的临时 git 检出目录。

use crate::fixture::FixtureError;

/// 创建一个临时 git 仓库,`origin` 指向 `remote_url`。
///
/// 用于让 [`crate::TtyRunner`] 在一个"remote 指向目标平台仓库"的工作目录中执行
/// `gf`,绕过 `gf` 仅从 `git remote get-url origin` 解析仓库路径、`list` 类命令
/// 无 `--repo` 覆盖的限制(详见设计文档
/// `docs/superpowers/specs/2026-09-03-e2e-gitlab-gitcode-coverage-design.md`)。
///
/// 返回的 [`tempfile::TempDir`] 在析构时自动清理临时目录。
///
/// # Errors
///
/// 当临时目录创建失败,或 `git init`/`git remote add` 命令执行失败
/// (非零退出码)时返回 `FixtureError::Io` 或 `FixtureError::Git`。
pub async fn scratch_repo_dir(remote_url: &str) -> Result<tempfile::TempDir, FixtureError> {
    let dir = tempfile::tempdir()?;

    let init = tokio::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(dir.path())
        .output()
        .await?;
    if !init.status.success() {
        return Err(FixtureError::Git(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&init.stderr)
        )));
    }

    let remote = tokio::process::Command::new("git")
        .args(["remote", "add", "origin", remote_url])
        .current_dir(dir.path())
        .output()
        .await?;
    if !remote.status.success() {
        return Err(FixtureError::Git(format!(
            "git remote add failed: {}",
            String::from_utf8_lossy(&remote.stderr)
        )));
    }

    Ok(dir)
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "Test code uses expect for simplicity")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_should_create_scratch_repo_with_origin_remote() {
        let dir = scratch_repo_dir("https://gitlab.com/example/project.git")
            .await
            .expect("scratch repo creation must succeed");

        let output = tokio::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(dir.path())
            .output()
            .await
            .expect("git remote get-url must run");
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "https://gitlab.com/example/project.git"
        );
    }

    #[tokio::test]
    async fn test_should_fail_when_remote_url_looks_like_a_flag() {
        // Empty strings are silently accepted by `git remote add` (verified locally:
        // `git remote add origin ""` exits 0 and stores an empty URL) — not a usable
        // failure case. A leading-`--` string, however, git parses as an unknown
        // option and rejects with a non-zero exit, which is what we need to exercise
        // the `FixtureError::Git` branch.
        let result = scratch_repo_dir("--bogus-flag").await;
        assert!(matches!(result, Err(FixtureError::Git(_))));
    }
}
