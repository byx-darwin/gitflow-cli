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

#[cfg(test)]
mod tests {
    // Branch deletion operations are tested via integration tests
    // since they require a real git repository and remote.
    // Unit tests would require mocking Command execution, which
    // adds complexity disproportionate to the simple git wrapper logic.
}
