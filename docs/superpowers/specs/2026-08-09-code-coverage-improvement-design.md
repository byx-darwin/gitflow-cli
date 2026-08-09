# Code Coverage Improvement Design

**Issue**: #158 - Improve code coverage to 80% threshold  
**Date**: 2026-08-09  
**Status**: Approved

## Context

Current code coverage is 36.26% (2011/5546 lines), measured by `cargo tarpaulin --workspace`. This is below the recommended 80% threshold for production Rust projects. The codebase has 54 source files but only 6 test files, indicating a significant testing gap.

## Goals

- Improve workspace-wide code coverage from 36.26% to at least 80%
- Critical modules (auth, release, signing) achieve ≥90% coverage
- All public functions have unit tests
- Error paths are tested (not just happy paths)
- No decrease in existing test pass rate

## Approach: Module Priority-Based Progression

### Strategy

Hybrid approach combining critical path prioritization with incremental gap filling:
1. **Phase 1**: Critical paths (auth, release, signing) reach 90%+ coverage
2. **Phase 2**: Other modules fill remaining gaps to achieve 80% overall

### Testing Architecture

Each module uses inline unit tests with `#[cfg(test)] mod tests`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    
    // Happy path
    #[test]
    fn test_should_<expected_behavior>() {
        // Arrange - Act - Assert
    }
    
    // Error path
    #[test]
    fn test_should_return_error_when_<condition>() {
        let result = function_call(invalid_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExpectedError::Variant));
    }
    
    // Parameterized tests
    #[rstest]
    #[case(input1, expected1)]
    #[case(input2, expected2)]
    fn test_should_handle_<scenario>(
        #[case] input: InputType,
        #[case] expected: ExpectedType,
    ) {
        // ...
    }
    
    // Boundary conditions
    #[test]
    fn test_should_handle_empty_input() { }
    
    #[test]
    fn test_should_handle_max_values() { }
}
```

### TDD Workflow (Strict Mode)

For each uncovered function/method:

1. **RED** - Write failing test
   - Define expected behavior
   - Test must fail (compile error or runtime failure)
   
2. **GREEN** - Minimal implementation
   - Only make the test pass
   - No extra logic
   
3. **REFACTOR** - Clean up code
   - Keep tests passing
   - Improve code quality
   - Run `make lint`

### Coverage Scope

Every function must cover:
- ✅ Normal execution paths
- ✅ All error paths (use `matches!()` to verify error types)
- ✅ Boundary conditions (empty values, max values, special characters, overflow, etc.)
- ✅ Parameter combinations (use `rstest` parameterized tests)

## Module Priority & Phases

### Phase 1: Critical Paths to 90%+

**Priority 1: Core Types & Error Handling**
- `crates/core/src/types.rs`
- `crates/core/src/cli_error.rs`
- `crates/core/src/error.rs` (if exists)
- Each crate's `error.rs`

**Priority 2: Auth Modules (All Platforms)**
- `crates/core/src/auth.rs`
- `crates/github/src/auth.rs`
- `crates/gitlab/src/auth.rs`
- `crates/gitcode/src/auth.rs`

**Priority 3: Release/Signing Modules**
- `crates/core/src/release.rs`
- `crates/github/src/release.rs`
- `crates/gitlab/src/release.rs`
- `crates/gitcode/src/release.rs`
- `crates/release-signer/src/main.rs`

**Target**: These modules ≥ 90% coverage

### Phase 2: Overall to 80%

**Priority 4: Platform Adapter Layer (by coverage, lowest first)**
- `crates/gitlab/src/` - Issue explicitly states low coverage
- `crates/gitcode/src/` - Extensive untested code
- `crates/github/src/` - Fill remaining coverage

**Priority 5: CLI Command Layer**
- `apps/cli/src/commands/` - Sort by file coverage

**Priority 6: Auxiliary Modules**
- `crates/e2e-core/src/`
- `crates/e2e-github/src/`
- Other low-coverage files

**Target**: Overall coverage ≥ 80%

### Progress Tracking

After each phase:
- Run `cargo tarpaulin --workspace` to verify coverage
- Update Issue #158 progress
- Record completed modules and coverage

## Testing Infrastructure & Tools

### Required Tools

- `rstest` - Parameterized tests (`#[case]`, `#[rstest]`)
- `mockall` - Mock generation (use only when necessary)
- `assert_matches` - Error type matching (`assert_matches!`)
- `proptest` - Property-based testing (for complex invariant verification)

### Optional Tools

- `wiremock` - HTTP mocking (if auth/release needs external APIs)
- `tempfile` - Temporary file/directory testing

### Test Naming Convention

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    
    // Happy path
    #[test]
    fn test_should_<expected_behavior>() {
        // Arrange - Act - Assert
    }
    
    // Error path
    #[test]
    fn test_should_return_error_when_<condition>() {
        let result = function_call(invalid_input);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExpectedError::Variant));
    }
    
    // Parameterized tests
    #[rstest]
    #[case(input1, expected1)]
    #[case(input2, expected2)]
    fn test_should_handle_<scenario>(
        #[case] input: InputType,
        #[case] expected: ExpectedType,
    ) {
        // ...
    }
    
    // Boundary conditions
    #[test]
    fn test_should_handle_empty_input() { }
    
    #[test]
    fn test_should_handle_max_values() { }
}
```

### Coverage Verification

**After each phase**:
```bash
# Overall coverage
cargo tarpaulin --workspace

# Specific module coverage
cargo tarpaulin -p <crate-name>

# Generate HTML report
cargo tarpaulin --workspace --out Html
```

**Quality Gates**:
- Critical path modules: ≥ 90%
- Overall workspace: ≥ 80%
- All tests must pass: `cargo test --workspace`
- No lint warnings: `cargo clippy -- -D warnings`

### Refactoring Strategy

When existing code is difficult to test:

1. **Extract pure functions** - Separate business logic from I/O
2. **Dependency injection** - Use traits to abstract external dependencies
3. **Complete error types** - Use `thiserror` to define clear error enums
4. **Avoid panics** - Change `unwrap()`/`expect()` to `Result<T>`

**Principle**: Minimize code changes, refactor only when necessary

## Acceptance Criteria

### Functional Acceptance

- ✅ `cargo tarpaulin --workspace` reports overall coverage ≥ 80%
- ✅ Critical path modules (auth, release, signing) coverage ≥ 90%
- ✅ All public functions have unit tests
- ✅ Error paths are tested (not just happy paths)
- ✅ Existing test pass rate 100% (no regressions)

### Quality Acceptance

- ✅ `cargo test --workspace` all tests pass
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` no warnings
- ✅ `cargo fmt --check` format correct
- ✅ No `TODO`, `unwrap()`, `expect()` in production code

### Documentation Acceptance

- ✅ All test functions have doc comments explaining test purpose
- ✅ Complex test scenarios have comments explaining boundary conditions
- ✅ Issue #158 updated with progress and final results

## Deliverables

1. **Test Code**
   - `#[cfg(test)] mod tests` for each module
   - Parameterized tests covering multiple scenarios
   - Error path and boundary condition tests

2. **Coverage Reports**
   - Final `cargo tarpaulin --workspace` output
   - Critical module coverage details

3. **Issue Updates**
   - Issue #158 comment updates with progress
   - Mark completed acceptance criteria

## Risk Management

**Risk 1: Existing code difficult to test**
- **Mitigation**: Prioritize refactoring to pure functions, use dependency injection
- **Fallback**: Introduce mocks when necessary, but minimize usage

**Risk 2: Slow coverage improvement**
- **Mitigation**: Check progress weekly, adjust priorities
- **Fallback**: If 80% is unreachable, set realistic target (e.g., 75%) and explain reasons

**Risk 3: Tests introduce regressions**
- **Mitigation**: Run full test suite after each phase
- **Fallback**: Fix regressions immediately, don't proceed to next phase

## Time Estimation

Based on current state (36.26% → 80%):
- **Phase 1** (critical paths to 90%+): ~3-5 days
- **Phase 2** (overall to 80%): ~5-7 days
- **Total**: ~8-12 days

**Note**: This is an estimate; actual progress depends on code complexity and refactoring difficulty.

## Related

- Issue: https://github.com/byx-darwin/gitflow-cli/issues/158
- Quality Gate Report (2026-08-09) — Coverage: 36.26% (FAIL)
