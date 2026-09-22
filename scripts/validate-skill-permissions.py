#!/usr/bin/env python3
"""Reject write-tool grants in skills that describe themselves as read-only."""

import argparse
import re
from pathlib import Path


READ_ONLY_CLAIM = re.compile(
    r"\bread-only\b|\bnever\s+(?:writes?|modifies?|mutates?)\b|\bdetection only\b",
    re.IGNORECASE,
)
WRITE_TOOL = re.compile(r"(?<![\w-])(?:Write|Edit)(?![\w-])")
RESTRICTED_SKILLS = {
    "gf-security-check",
    "gf-label-stats",
    "gf-pipeline-analyzer",
    "gf-repo-onboarding",
}


def frontmatter_field(lines: list[str], field: str) -> str:
    """Return a simple inline or YAML-list frontmatter field without dependencies."""
    values: list[str] = []
    collecting = False
    for line in lines:
        if line.startswith(f"{field}:"):
            collecting = True
            values.append(line.split(":", 1)[1])
        elif collecting and (line.startswith(" ") or line.startswith("\t")):
            values.append(line)
        elif collecting:
            break
    return " ".join(values)


def check_skill(path: Path) -> list[str]:
    """Return any grant violations for one SKILL.md."""
    content = path.read_text(encoding="utf-8")
    parts = content.split("---", 2)
    if len(parts) != 3 or parts[0].strip():
        return [f"{path}: missing YAML frontmatter"]
    header = parts[1].splitlines()
    introduction = parts[2].split("\n## ", 1)[0]
    restricted = path.parent.name in RESTRICTED_SKILLS
    read_only = READ_ONLY_CLAIM.search(introduction) is not None
    if not read_only and not restricted:
        return []
    grants = frontmatter_field(header, "allowed-tools")
    errors: list[str] = []
    if restricted and not read_only:
        errors.append(f"{path}: restricted skill must state its read-only scope")
    if WRITE_TOOL.search(grants):
        errors.append(f"{path}: read-only skill grants Write or Edit via allowed-tools")
    if restricted:
        denials = frontmatter_field(header, "disallowed-tools")
        if not {"Write", "Edit"}.issubset(set(re.findall(r"\b(?:Write|Edit)\b", denials))):
            errors.append(f"{path}: restricted skill must deny Write and Edit via disallowed-tools")
        if re.search(r"\bBash\b", grants):
            errors.append(f"{path}: restricted skill must not pre-approve Bash")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "skills")
    args = parser.parse_args()
    paths = sorted(args.root.glob("gf-*/SKILL.md"))
    if not paths:
        parser.error(f"no gf skills found under {args.root}")
    errors = [error for path in paths for error in check_skill(path)]
    if errors:
        for error in errors:
            print(f"✗ {error}")
        return 1
    print(f"✓ {len(paths)} skill permission grants checked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
