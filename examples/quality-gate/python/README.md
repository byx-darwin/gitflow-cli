# Python Quality Gate Example

Minimal Python project for validating `gf-quality` gates.

## Setup

```bash
cd examples/quality-gate/python
python -m venv .venv
source .venv/bin/activate  # Linux/macOS
pip install -e ".[dev]"
```

## Validate

```bash
gf quality
```

Expected: ALL CHECKS PASSED
