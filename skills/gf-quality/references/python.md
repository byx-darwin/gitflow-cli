# Python Quality Toolchain

**Shared language profile:** `gf-quality/references/profiles/python.md`. Read it before running these gates.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `python -m compileall src/ -q` | exit 0 |
| 2 | test | `python -m pytest --tb=short` | all pass |
| 3 | coverage | `python -m pytest --cov=src/ --cov-report=term-missing --cov-fail-under=${COV_THRESHOLD:-80}` | exit 0 (total line coverage ≥ threshold); N/A if no `.py` in change set |
| 4 | format | `ruff format --check .` or `black --check .` | exit 0 |
| 5 | static | `ruff check .` or `pylint src/` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

## Gate Notes

- Prefer Ruff for format and lint; use the project's configured Black and Pylint fallback if Ruff is unavailable.
- Gate 1: for compiled Python checks; skip for pure script projects (mark N/A)
- Gate 4: report the `ruff format --check .` diff — never run `ruff format .` or `black .`
- Gate 5: check for TODO/FIXME/HACK residuals with `grep -rn "TODO\|FIXME\|HACK" --include="*.py" .`
- Respect project's existing tool config (`.ruff.toml`, `pyproject.toml [tool.ruff]`)

## Forbidden Actions

- ❌ Never auto-fix with `ruff format .` or `black .` — report only
- ❌ Never install packages into system Python — use venv or pipx

## Quality Gate Configuration

### Configuration Examples

#### pyproject.toml

```toml
[project]
name = "example"
version = "0.1.0"
requires-python = ">=3.10"

[tool.ruff]
line-length = 100
select = ["E", "F", "I"]

[tool.black]
line-length = 100
target-version = ["py310"]

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
```

#### .ruff.toml

```toml
line-length = 100
target-version = "py310"

[lint]
select = ["E", "F", "I", "N", "W"]
ignore = ["E501"]
```

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `pip: command not found` | pip not installed | `python -m ensurepip --upgrade` |
| `Permission denied` | System Python | Use virtual environment: `python -m venv .venv` |
| `ModuleNotFoundError` | Import error | Activate venv: `source .venv/bin/activate`, then report missing project dependencies |
| `ImportError: cannot import name` | Circular import | Restructure imports |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Test failure | Fix failing tests |
| 2 | Syntax error | Fix syntax errors |
| 127 | Command not found | Install missing tool |

### FAQ

**Q: ruff vs black vs pylint?**
A: ruff is fastest (covers format + lint). black is format-only. pylint is comprehensive but slow.

**Q: How to manage multiple Python versions?**
A: Use `pyenv` to manage versions. Set per-project with `pyenv local 3.10`.

**Q: pytest fixtures?**
A: Define in `conftest.py`. Use `@pytest.fixture` decorator. Share across tests.

### Performance Tips

- Use `pytest-xdist` for parallel test execution: `pytest -n auto`
- Use `--cov-report=term-missing` for faster coverage reports
- Use `pytest --cache-clear` to clear cache if tests behave unexpectedly
