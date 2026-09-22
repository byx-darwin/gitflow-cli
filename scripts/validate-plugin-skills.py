#!/usr/bin/env python3
"""Check the Claude Code plugin catalog against the canonical skills tree."""

import json
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    plugin = json.loads((root / ".claude-plugin/plugin.json").read_text(encoding="utf-8"))
    marketplace = json.loads(
        (root / ".claude-plugin/marketplace.json").read_text(encoding="utf-8")
    )

    actual = {
        path.name
        for path in (root / "skills").iterdir()
        if path.is_dir() and (path / "SKILL.md").is_file()
    }
    paths = plugin.get("skills", [])
    expected_paths = {f"./skills/{name}" for name in actual}
    listed_paths = set(paths)
    errors = []
    if len(paths) != len(listed_paths):
        errors.append("plugin.json contains duplicate skill paths")
    if listed_paths != expected_paths:
        errors.append(
            "plugin.json skills mismatch: "
            f"missing={sorted(expected_paths - listed_paths)}, "
            f"extra={sorted(listed_paths - expected_paths)}"
        )
    entries = marketplace.get("plugins", [])
    if len(entries) != 1 or entries[0].get("name") != plugin.get("name"):
        errors.append("marketplace.json must list the plugin exactly once by manifest name")
    elif entries[0].get("source") not in (".", "./"):
        errors.append("marketplace source must be the repository root")

    if errors:
        for error in errors:
            print(f"✗ {error}", file=sys.stderr)
        return 1
    print(f"✓ Claude Code plugin lists exactly {len(actual)} canonical skills")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
