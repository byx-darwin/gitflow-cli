# Python Quality Toolchain

**Detection:** `pyproject.toml`, `setup.py`, or `setup.cfg` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `python -m compileall src/ -q` | exit 0 |
| 2 | test | `python -m pytest --tb=short` | all pass |
| 3 | coverage | `python -m pytest --cov=src/ --cov-report=term-missing` | incremental ≥ 80% |
| 4 | format | `ruff format --check .` or `black --check .` | exit 0 |
| 5 | static | `ruff check .` or `pylint src/` | exit 0 |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A) |

## Tool Installation

| Tool | Install Command | Required By |
|------|----------------|-------------|
| ruff | `pip install ruff` | Gate 4, 5 |
| black | `pip install black` | Gate 4 (fallback) |
| pylint | `pip install pylint` | Gate 5 (fallback) |
| pytest-cov | `pip install pytest-cov` | Gate 3 |

Prefer `ruff` (fast, covers format + lint). Fall back to `black` + `pylint` if ruff not configured.

## Notes

- Gate 1: for compiled Python checks; skip for pure script projects (mark N/A)
- Gate 4: auto-fix with `ruff format .` or `black .` only after user confirmation
- Gate 5: check for TODO/FIXME/HACK residuals with `grep -rn "TODO\|FIXME\|HACK" --include="*.py" .`
- Respect project's existing tool config (`.ruff.toml`, `pyproject.toml [tool.ruff]`)

## Forbidden Actions

- ❌ Never auto-fix without showing diff first
- ❌ Never install packages into system Python — use venv or pipx
