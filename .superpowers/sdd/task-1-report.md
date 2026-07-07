# Task 1 Report: 添加 --repo 参数支持

## Status: DONE

## What Was Implemented

Added `--repo` CLI parameter to `gitflow-cli issue create` command, allowing users to specify a target repository (`owner/repo` format) different from the current working directory's remote.

### Behavior
- **Before**: `issue create` always used the repo extracted from `git remote get-url origin`
- **After**: `--repo owner/repo` overrides the auto-detected repo; without `--repo`, behavior is unchanged

### Files Changed

1. **`apps/cli/src/commands/issue.rs`**:
   - Added `repo: Option<String>` field to `IssueCommand::Create` variant with `#[arg(long)]` attribute
   - Modified `handle` function to compute `effective_repo` by checking for `--repo` override before creating the provider
   - Added 2 new unit tests:
     - `test_should_parse_issue_create_with_repo` -- verifies `--repo` is parsed correctly
     - `test_should_parse_issue_create_without_repo` -- verifies `repo` is `None` when not specified

2. **No changes to `apps/cli/src/main.rs`**: The `IssueCommand` enum is defined in `issue.rs` and used via `clap::Subcommand` derive, so CLI argument parsing is handled automatically.

## Test Results

### RED Phase
Tests failed with expected compilation errors:
```
error[E0026]: variant `commands::issue::IssueCommand::Create` does not have a field named `repo`
```

### GREEN Phase
After implementing the feature, all tests pass:
```
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 145 filtered out
```

### REFACTOR Phase
Refactored provider creation to avoid creating an unused provider for the `Create` case. The `effective_repo` is now computed once before provider construction.

### Validation
- `make clippy`: PASS (no warnings)
- `cargo test -p gitflow-cli --bin gitflow-cli -- issue`: 16/16 PASS
- `cargo test -p gitflow-cli --test issue_test`: 6/6 PASS
- `make test`: Pre-existing failures in `workflow_modes_test` (unrelated to this change)

## Usage Example

```bash
# Auto-detect repo from remote (existing behavior)
gitflow-cli issue create --title "Bug" --body "Description"

# Override repo (new behavior)
gitflow-cli issue create --title "Bug" --body "Description" --repo byx-darwin/gitflow-cli

# With platform override
gitflow-cli issue create --platform github --repo byx-darwin/gitflow-cli --title "Bug"
```

## Commit SHA
(To be added after commit)

## Concerns
- The pre-existing `workflow_modes_test` failures are unrelated to this change and should be addressed separately.
