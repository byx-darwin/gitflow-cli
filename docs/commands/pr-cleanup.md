# gf pr cleanup

Safely clean up branches and worktrees after PR merge.

## Synopsis

```bash
gf pr cleanup <NUMBER>... [OPTIONS]
gf pr cleanup --merged [OPTIONS]
gf pr cleanup --closed [OPTIONS]
```

## Description

The `cleanup` command safely removes branches and worktrees associated with merged or closed pull requests. It performs safety checks to prevent accidental deletion of protected branches or the current branch.

This command is particularly useful in worktree-based workflows where `gh pr merge --delete-branch` fails because the branch is checked out in a worktree.

## Arguments

`<NUMBER>...`
: One or more PR numbers to clean up. Mutually exclusive with `--merged` and `--closed`.

## Options

`--worktree <PATH>`
: Remove the specified worktree path after cleanup.

`--remote`
: Delete remote branches (default: true).

`--local`
: Delete local branches (default: true).

`--force`
: Force cleanup of unmerged branches. Bypasses PR state check but not protected branch check.

`--dry-run`
: Show what would be done without actually deleting anything.

`--yes`, `-y`
: Skip interactive confirmation.

`--merged`
: Clean up all merged PRs. Mutually exclusive with `<NUMBER>...`.

`--closed`
: Clean up all closed PRs. Mutually exclusive with `<NUMBER>...`.

## Safety Checks

The command performs the following safety checks before deletion:

1. **PR State**: Only allows cleanup of merged or closed PRs (unless `--force` is used)
2. **Protected Branches**: Refuses to delete protected branches (main, master, develop, release/*)
3. **Current Branch**: Refuses to delete the currently checked-out branch

These checks cannot be bypassed except for the PR state check (using `--force`).

## Worktree Handling

When running in a worktree:

1. Automatically detects if currently in a worktree
2. Exits the worktree and returns to the main repository
3. If `--worktree <PATH>` is specified, removes the worktree directory
4. If `--worktree` is not specified, preserves the worktree directory and displays a message

## Examples

### Clean up a single PR

```bash
gf pr cleanup 172
```

Deletes both remote and local branches for PR #172 after confirming it's merged/closed.

### Clean up multiple PRs

```bash
gf pr cleanup 172 173 174
```

Cleans up branches for PRs #172, #173, and #174.

### Clean up with worktree removal

```bash
gf pr cleanup 172 --worktree .claude/worktrees/feat-172
```

Deletes branches and removes the specified worktree directory.

### Preview without deleting

```bash
gf pr cleanup 172 --dry-run
```

Shows what would be deleted without actually deleting anything.

### Force cleanup of unmerged branch

```bash
gf pr cleanup 172 --force
```

Deletes the branch even if the PR is not merged/closed. Still respects protected branch rules.

### Clean up all merged PRs

```bash
gf pr cleanup --merged
```

Finds all merged PRs and cleans up their branches.

### Clean up all closed PRs

```bash
gf pr cleanup --closed
```

Finds all closed PRs and cleans up their branches.

### Skip confirmation for scripting

```bash
gf pr cleanup 172 --yes
```

Skips the interactive confirmation prompt.

## Exit Status

Returns 0 on success, non-zero on failure.

## See Also

- `gf pr merge` - Merge a pull request
- `gf pr view` - View pull request details
- `git worktree` - Git worktree management

## Notes

- The command uses the platform's PR API (GitHub/GitLab/GitCode) to fetch PR state
- Remote branch deletion may fail if the branch doesn't exist; this is treated as a warning, not an error
- Batch operations continue on individual failures and report results at the end
- Protected branch detection is currently local only (main, master, develop, release/*); remote branch protection rules are not yet checked
