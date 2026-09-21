#!/usr/bin/env python3
"""Render an interactive, offline-readable HTML review page from a git diff.

This is a DERIVED VIEW — never hand-edit the generated HTML. Re-run
`make render-diff-review` to refresh it after the diff changes.

Standard-library only, no third-party dependencies. No third-party
JS/CSS either — the rendered page has no CDN links and no bundled
libraries (in particular, no Prism.js).
"""
import html
import json
import os
import re
import subprocess
import sys

HUNK_HEADER_RE = re.compile(r'^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@')
DIFF_GIT_RE = re.compile(r'^diff --git a/(.+) b/(.+)$')


def _new_file_entry(path):
    return {
        "path": path,
        "old_path": None,
        "status": "modified",
        "lines": [],
        "error": None,
        "pseudocode": None,
        "call_tree": None,
    }


def _parse_hunks(lines, start_idx):
    """Parse consecutive hunks starting at lines[start_idx] (a '@@' line).

    Returns the list of line entries. Stops at the next 'diff --git ' line
    or end of the block.
    """
    entries = []
    i = start_idx
    while i < len(lines) and lines[i].startswith('@@'):
        m = HUNK_HEADER_RE.match(lines[i])
        if not m:
            raise ValueError(f"malformed hunk header: {lines[i]!r}")
        old_line = int(m.group(1))
        new_line = int(m.group(3))
        i += 1
        while i < len(lines) and not lines[i].startswith('@@') and not lines[i].startswith('diff --git '):
            line = lines[i]
            if line.startswith('+'):
                entries.append({"old_line": None, "new_line": new_line, "type": "add", "content": line[1:]})
                new_line += 1
            elif line.startswith('-'):
                entries.append({"old_line": old_line, "new_line": None, "type": "remove", "content": line[1:]})
                old_line += 1
            elif line.startswith(' '):
                entries.append({"old_line": old_line, "new_line": new_line, "type": "context", "content": line[1:]})
                old_line += 1
                new_line += 1
            elif line.startswith('\\'):
                pass  # "\ No newline at end of file"
            else:
                raise ValueError(f"malformed diff line: {line!r}")
            i += 1
    return entries


def _parse_file_block(block_text):
    lines = block_text.splitlines()
    if not lines:
        raise ValueError("empty diff block")
    m = DIFF_GIT_RE.match(lines[0])
    if not m:
        raise ValueError(f"malformed diff --git header: {lines[0]!r}")
    _, b_path = m.group(1), m.group(2)

    entry = _new_file_entry(b_path)

    for line in lines[1:6]:
        if line.startswith("rename from "):
            entry["old_path"] = line[len("rename from "):]
            entry["status"] = "renamed"
        elif line.startswith("rename to "):
            entry["path"] = line[len("rename to "):]

    hunk_start = None
    for idx, line in enumerate(lines):
        if line.startswith("@@"):
            hunk_start = idx
            break
    if hunk_start is not None:
        entry["lines"] = _parse_hunks(lines, hunk_start)

    return entry


def parse_diff_text(diff_text):
    if not diff_text.strip():
        return []
    blocks = re.split(r'(?=^diff --git )', diff_text, flags=re.MULTILINE)
    blocks = [b for b in blocks if b.strip()]
    entries = []
    for block in blocks:
        try:
            entries.append(_parse_file_block(block))
        except ValueError as exc:
            first_line = block.splitlines()[0] if block.splitlines() else "<empty>"
            entries.append({
                "path": first_line,
                "old_path": None,
                "status": "error",
                "lines": [],
                "error": str(exc),
                "pseudocode": None,
                "call_tree": None,
            })
    return entries


if __name__ == "__main__":
    print("render-diff-review.py: CLI wiring added in a later task", file=sys.stderr)
    sys.exit(1)
