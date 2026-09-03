//! 临时 git 仓库构造模块
//!
//! 为需要"remote 指向真实目标仓库"的实测(GitLab/GitCode 的 issue/pr 场景)
//! 提供一次性的临时 git 检出目录。

use crate::fixture::FixtureError;

/// 会重定向 git 仓库发现的环境变量,必须在生成子进程前清除。
///
/// 这些变量若从父进程继承(例如本仓库自身的 `pre-push` hook 执行 `cargo nextest`
/// 时,git 会为 hook 进程设置 `GIT_DIR`/`GIT_WORK_TREE` 指向被 push 的仓库),会让
/// `git init`/`git remote add` 忽略 `current_dir()`,转而操作被继承的那个仓库——
/// 这正是本模块存在的意义(隔离出一个"干净"的仓库)要防止的情况。
const GIT_ENV_VARS_TO_CLEAR: [&str; 4] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_CEILING_DIRECTORIES",
];

/// 构造一个不受父进程 git 环境变量污染的 `git` 命令。
fn git_command(dir: &std::path::Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("git");
    cmd.current_dir(dir);
    for var in GIT_ENV_VARS_TO_CLEAR {
        cmd.env_remove(var);
    }
    cmd
}

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

    let init = git_command(dir.path())
        .args(["init", "--quiet"])
        .output()
        .await?;
    if !init.status.success() {
        return Err(FixtureError::Git(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&init.stderr)
        )));
    }

    let remote = git_command(dir.path())
        .args(["remote", "add", "origin", remote_url])
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

        let output = git_command(dir.path())
            .args(["remote", "get-url", "origin"])
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

    #[tokio::test]
    async fn test_should_isolate_from_inherited_git_dir_env_vars() {
        // Regression test for a real bug caught by this repo's own `pre-push` hook:
        // git sets `GIT_DIR`/`GIT_WORK_TREE` in the hook process's environment,
        // pointing at the repo being pushed. Those vars are inherited by
        // `tokio::process::Command` unless explicitly cleared, which makes
        // `git init`/`git remote add` operate on the INHERITED repo (which already
        // has an `origin` remote) instead of the fresh scratch directory — surfacing
        // as `error: remote origin already exists.`
        //
        // This crate forbids `unsafe`, so it can't use `std::env::set_var` (unsound
        // in multi-threaded Rust 2024) to pollute *this test process's* environment
        // the way a real hook would. Instead, `.env(...)` is applied to the
        // `Command` builder first, standing in for parent-process inheritance:
        // `env_remove()` always overrides whatever came before it on the same
        // builder, whether that "before" state is a prior explicit `.env()` call or
        // true inheritance from the parent process — the two are indistinguishable
        // to the builder, so this is a faithful simulation of the real hook
        // environment without touching global process state.
        let repo_root = tokio::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .await
            .expect("git rev-parse must run");
        let repo_root = String::from_utf8_lossy(&repo_root.stdout)
            .trim()
            .to_string();
        let git_dir = tokio::process::Command::new("git")
            .args(["rev-parse", "--absolute-git-dir"])
            .output()
            .await
            .expect("git rev-parse must run");
        let git_dir = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();

        let dir = tempfile::tempdir().expect("tempdir must be created");

        let mut cmd = tokio::process::Command::new("git");
        cmd.env("GIT_DIR", &git_dir)
            .env("GIT_WORK_TREE", &repo_root);
        for var in GIT_ENV_VARS_TO_CLEAR {
            cmd.env_remove(var);
        }
        cmd.current_dir(dir.path());
        let init = cmd
            .args(["init", "--quiet"])
            .output()
            .await
            .expect("git init must run");
        assert!(init.status.success());

        // Without the `GIT_ENV_VARS_TO_CLEAR` fix, this call would fail with
        // "remote origin already exists" because it would incorrectly operate on the repo
        // at `git_dir`/`repo_root`, which already has an `origin` remote.
        let remote = git_command(dir.path())
            .args([
                "remote",
                "add",
                "origin",
                "https://gitcode.com/example/project.git",
            ])
            .output()
            .await
            .expect("git remote add must run");
        assert!(
            remote.status.success(),
            "git_command() must clear inherited GIT_DIR/GIT_WORK_TREE: {}",
            String::from_utf8_lossy(&remote.stderr)
        );
    }
}
