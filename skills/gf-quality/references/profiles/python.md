# Python Language Profile

Shared facts for the Python language layers in `gf-quality`, `gf-precommit`, and `gf-smell`.

## Version Source

- Read `requires-python` from `pyproject.toml` when present and compare it with `python --version`; do not assume a fixed target version.

## Tools and Installation

| Tool | Installation guidance | Used by |
|---|---|---|
| ruff | Recommend `pip install ruff` in the project virtual environment | format and lint |
| black | Recommend `pip install black` in the project virtual environment | format fallback |
| pylint | Recommend `pip install pylint` in the project virtual environment | lint fallback |
| pytest-cov | Recommend `pip install pytest-cov` in the project virtual environment | coverage |
| radon | Recommend `pip install radon` in the project virtual environment | smell detection |

Never install packages automatically or into system Python. `PYTHONPATH`, `PYTHONDONTWRITEBYTECODE`, and `VIRTUAL_ENV` may affect checks; inspect values only when needed.
