# Python Pre-commit Checks

**Detection:** `pyproject.toml`, `setup.py`, or `setup.cfg` in project root.

## Check Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `ruff format --check .` or `black --check .` | `ruff format .` or `black .` |
| lint | `ruff check .` or `pylint src/` | `ruff check --fix .` |
| test | `python -m pytest --tb=short` | — |

## Notes

- Prefer `ruff` (covers format + lint). Fall back to `black` + `pylint` if not configured.
- Fix commands require user confirmation before execution
- Use project venv if available; never install into system Python
