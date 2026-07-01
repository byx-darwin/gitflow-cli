# TDD (Test-Driven Development)

This template is designed for TDD. Every code change should follow this cycle:

```
RED → GREEN → REFACTOR
```

## The Cycle

1. **RED** — Write a failing test that describes the behavior you want
2. **GREEN** — Write the minimal code to make the test pass
3. **REFACTOR** — Clean up the code while keeping tests green

## Tooling

| Command | Purpose |
|---|---|
| `make test` | Run all tests once |
| `make test-watch` | Watch for file changes and re-run tests automatically |
| `make coverage` | Generate test coverage report |

## Test Patterns

### Unit tests (`#[test]`)

```rust
#[test]
fn test_should_reject_empty_input() {
    let result = parse("");
    assert!(result.is_err());
}
```

### Parameterized tests (`rstest`)

```rust
use rstest::rstest;

#[rstest]
#[case("valid", true)]
#[case("", false)]
fn test_should_validate(#[case] input: &str, #[case] expected: bool) {
    assert_eq!(validate(input), expected);
}
```

### Property-based tests (`proptest`)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_should_always_reject_absolute_paths(
        rest in "[a-zA-Z0-9/._-]{0,32}"
    ) {
        let path = format!("/{rest}");
        assert!(SafePath::new(&path).is_err());
    }
}
```

## CI Integration

Tests run on every push and PR. Property-based tests sample 256 cases by default and complete in under 1 second.

## TDD Workflow

```bash
# Terminal 1: start test watcher
make test-watch

# Terminal 2: write code
# 1. Write a failing test
# 2. Watch it fail in terminal 1
# 3. Write the implementation
# 4. Watch it pass
# 5. Refactor
```

## For AI Agents

When implementing features, agents should:

1. Write tests first that describe the expected behavior
2. Run `make test` to confirm tests fail (RED)
3. Implement the minimal code to pass tests
4. Run `make test` to confirm tests pass (GREEN)
5. Run `make lint` to verify code quality
6. Refactor if needed while keeping tests green
