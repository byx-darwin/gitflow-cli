---
name: gitflow-precommit
description: |
  Use when the user wants to run pre-commit quality checks, configure or
  verify a Git pre-commit hook, or their commit is rejected by a quality
  gate. 当用户提交前运行质量检查、配置/验证 pre-commit hook、或提交被质量
  关卡拒绝时使用。
---

# gitflow-precommit — Language-Agnostic Pre-commit Checks

Run language-appropriate fmt/lint/test before commit. Report results. Optionally configure Git hook.

## Step 1: Language Detection

Same as `gitflow-quality`. Check marker files in project root:

| Marker | Language | Reference |
|--------|----------|-----------|
| `Cargo.toml` | Rust | `references/rust.md` |
| `go.mod` | Go | `references/go.md` |
| `pom.xml` / `build.gradle` | Java | `references/java.md` |
| `pyproject.toml` / `setup.py` | Python | `references/python.md` |
| `package.json` | Node.js | `references/node.md` |

After detection, load the matching `references/<lang>.md` for the specific commands.

## Step 2: Run Checks

Three checks in sequence:

| # | Check | What |
|---|-------|------|
| 1 | **format** | Code formatting check |
| 2 | **lint** | Static analysis / linting |
| 3 | **test** | Run test suite |

All commands come from the detected language reference. If no project detected, fall back to `pre-commit run --all-files`.

## Step 3: Report

```
| Check    | Status | Details |
|----------|--------|---------|
| format   | ✅/❌ | <files if diff> |
| lint     | ✅/❌ | <warnings if any> |
| test     | ✅/❌ | <failures if any> |

Result: ✅ all passed / ❌ <N> check(s) failed
```

## Fix vs Report Flow

| User request | Action |
|--------------|--------|
| "run the checks" | Run all three → summary table |
| "fmt failed" / "fix format" | Show diff → ask confirmation → auto-fix → re-check |
| "configure hook" | Write `.git/hooks/` or `pre-commit install` |
| "fix lint" | Show diff → ask confirmation → fix → re-check |

**Fix operations require explicit user confirmation. Never auto-fix without showing diff first.**

## Red Flags

- 🚩 "Auto-fix all lints" → show diff first, confirm before fixing
- 🚩 "Configure hook while running checks" → hook configuration is a side effect, requires authorization
- 🚩 "Run clean command to fix build" → never (cargo clean / mvn clean / go clean)

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting `--allow-dirty` for fix commands | Prompt before fixing |
| Hook missing executable permission | `chmod +x .git/hooks/pre-commit` |
| Auto-configuring hook without asking | Ask user first |

## Error Handling

| Error | Handling |
|-------|----------|
| No project detected | Fall back to `.pre-commit-config.yaml` only |
| Format check failed | Show diff → suggest fix command → wait for confirmation |
| Clean command used | Abort immediately (forbidden) |

## See Also

- `gitflow-quality` — full 6-gate quality check (includes pre-commit as Gate 6)
- `gitflow-commit` — bridges commit and pre-commit
