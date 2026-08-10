//! Git operations for branch and worktree management.
//!
//! Provides low-level git operations used by [`CleanupService`](crate::cleanup::CleanupService).
//! All operations are local git commands — no platform API calls.

use tokio::process::Command;

use crate::Result;

/// Delete a local git branch.
///
/// Uses `git branch -d <branch>` for merged branches, or `git branch -D <branch>`
/// if `force` is true.
///
/// # Errors
///
/// Returns an error if:
/// - The git command fails
/// - The branch does not exist
/// - The branch is not fully merged (unless `force` is true)
/// - The branch is currently checked out
pub async fn delete_local_branch(branch: &str, force: bool) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    let output = Command::new("git")
        .args(["branch", flag, branch])
        .output()
        .await
        .map_err(|e| crate::CoreError::App(format!("Failed to execute git branch: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::CoreError::App(format!(
            "Failed to delete local branch '{branch}': {stderr}"
        )));
    }

    Ok(())
}

/// Delete a remote git branch.
///
/// Uses `git push origin --delete <branch>`.
///
/// # Errors
///
/// Returns an error if:
/// - The git command fails
/// - The remote branch does not exist
/// - Authentication fails
pub async fn delete_remote_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "origin", "--delete", branch])
        .output()
        .await
        .map_err(|e| crate::CoreError::App(format!("Failed to execute git push: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::CoreError::App(format!(
            "Failed to delete remote branch '{branch}': {stderr}"
        )));
    }

    Ok(())
}

/// Check if the current directory is inside a git worktree.
///
/// A worktree is detected by checking if `.git` is a file (not a directory).
/// In a regular repo, `.git` is a directory; in a worktree, it's a file
/// containing a pointer to the main repo's git directory.
///
/// # Errors
///
/// Returns an error if the `.git` path cannot be accessed.
#[allow(
    clippy::unused_async,
    reason = "Consistent async API with other git_ops functions"
)]
pub async fn is_in_worktree() -> Result<bool> {
    let git_path = std::path::Path::new(".git");
    if !git_path.exists() {
        return Err(crate::CoreError::App("Not in a git repository".to_string()));
    }
    Ok(git_path.is_file())
}

/// Get the path to the main repository from a worktree.
///
/// Uses `git rev-parse --git-common-dir` to find the main repo's git directory,
/// then returns its parent (the main repo root).
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git worktree
/// - The git command fails
pub async fn get_main_repo_path() -> Result<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .await
        .map_err(|e| crate::CoreError::App(format!("Failed to execute git: {e}")))?;

    if !output.status.success() {
        return Err(crate::CoreError::App(
            "Failed to get main repo path".to_string(),
        ));
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let git_path = std::path::PathBuf::from(&git_dir);

    // git-common-dir returns the .git directory; parent is the repo root
    git_path
        .parent()
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| {
            crate::CoreError::App("Cannot determine parent of git directory".to_string())
        })
}

/// Get the path to the current worktree.
///
/// Uses `git rev-parse --show-toplevel` to get the worktree root.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git worktree
/// - The git command fails
pub async fn get_current_worktree_path() -> Result<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .await
        .map_err(|e| crate::CoreError::App(format!("Failed to execute git: {e}")))?;

    if !output.status.success() {
        return Err(crate::CoreError::App(
            "Failed to get current worktree path".to_string(),
        ));
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(path))
}

/// Remove a git worktree.
///
/// Uses `git worktree remove <path>`.
///
/// # Errors
///
/// Returns an error if:
/// - The worktree does not exist
/// - The worktree contains uncommitted changes
/// - The git command fails
pub async fn remove_worktree(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "remove", path])
        .output()
        .await
        .map_err(|e| {
            crate::CoreError::App(format!("Failed to execute git worktree remove: {e}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::CoreError::App(format!(
            "Failed to remove worktree '{path}': {stderr}"
        )));
    }

    Ok(())
}

/// Exit the current worktree and return to the main repository.
///
/// Changes the current directory to the main repository root.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a worktree
/// - Cannot determine main repo path
/// - Cannot change directory
pub async fn exit_worktree() -> Result<std::path::PathBuf> {
    if !is_in_worktree().await? {
        return Err(crate::CoreError::App("Not in a worktree".to_string()));
    }

    let main_repo = get_main_repo_path().await?;
    std::env::set_current_dir(&main_repo).map_err(|e| {
        crate::CoreError::App(format!(
            "Failed to change directory to {}: {e}",
            main_repo.display()
        ))
    })?;

    Ok(main_repo)
}

#[cfg(test)]
mod tests {
    // Branch deletion operations are tested via integration tests
    // since they require a real git repository and remote.
    // Unit tests would require mocking Command execution, which
    // adds complexity disproportionate to the simple git wrapper logic.
}
