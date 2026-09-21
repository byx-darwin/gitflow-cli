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

    for line in lines:
        if line.startswith("Binary files") and line.endswith("differ"):
            entry["status"] = "binary"
            return entry

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
            header_match = DIFF_GIT_RE.match(first_line)
            error_path = header_match.group(2) if header_match else first_line
            entries.append({
                "path": error_path,
                "old_path": None,
                "status": "error",
                "lines": [],
                "error": str(exc),
                "pseudocode": None,
                "call_tree": None,
            })
    return entries


def scan_untracked_files():
    """Synthesize an all-added diff entry for each untracked file.

    `git diff` never includes untracked files, so this shells out to
    `git status --porcelain` separately and, for each `??`-prefixed
    entry, runs `git diff --no-index -- /dev/null <path>` and reuses
    `parse_diff_text` on that output (not a separate parsing path).
    """
    status_result = subprocess.run(
        ["git", "status", "--porcelain"],
        capture_output=True, text=True, check=True,
    )
    entries = []
    for line in status_result.stdout.splitlines():
        if not line.startswith("?? "):
            continue
        path = line[3:]
        diff_result = subprocess.run(
            ["git", "diff", "--no-index", "--", "/dev/null", path],
            capture_output=True, text=True,
        )
        # git diff --no-index exits 1 when a difference is found — expected here.
        for file_entry in parse_diff_text(diff_result.stdout):
            file_entry["status"] = "untracked"
            file_entry["path"] = path
            entries.append(file_entry)
    return entries


def build_annotations(diff_range, tracked_files, untracked_files):
    return {
        "diff_range": diff_range,
        "files": tracked_files + untracked_files,
    }


def _sanitize_range(diff_range):
    return diff_range.replace("/", "-")


def run_scan(diff_range, output_path):
    result = subprocess.run(
        ["git", "diff", diff_range],
        capture_output=True, text=True, check=True,
    )
    tracked_files = parse_diff_text(result.stdout)
    untracked_files = scan_untracked_files()
    annotations = build_annotations(diff_range, tracked_files, untracked_files)
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(annotations, f, indent=2, ensure_ascii=False)


BANNER_TEXT = (
    "⚠️ 派生视图 — 请勿手工编辑，由 make render-diff-review "
    "从 git diff 生成"
)

STYLE = """
body { font-family: -apple-system, sans-serif; margin: 0; background: #f4f4f6; }
.banner { background: #fff3cd; border-bottom: 1px solid #ffe08a; color: #7a5b00;
          padding: 10px 16px; font-weight: bold; }
.layout { display: flex; height: calc(100vh - 44px); }
#file-tree { width: 220px; overflow-y: auto; background: #fff; border-right: 1px solid #ddd; }
#file-tree .file-item { padding: 6px 10px; cursor: pointer; font-size: 13px; }
#diff-pane { flex: 1; overflow-y: auto; background: #fff; }
#annotation-pane { width: 280px; overflow-y: auto; background: #fafafa; border-left: 1px solid #ddd; }
.resize-handle { width: 4px; cursor: col-resize; background: #ddd; }
.diff-line { display: flex; font-family: monospace; font-size: 12px; white-space: pre; }
.diff-line.add { background: #e6ffed; }
.diff-line.remove { background: #ffeef0; }
.diff-line .lineno { width: 70px; color: #999; text-align: right; padding-right: 8px; user-select: none; }
.diff-line.hover-highlight { outline: 2px solid #7cb3f5; }
.annotation-card { padding: 8px; border-bottom: 1px solid #eee; font-size: 12px; cursor: pointer; }
.annotation-card.hover-highlight { background: #e0ecff; }
.annotation-card .empty { color: #999; font-style: italic; }
.file-header { background: #f0f0f2; font-weight: bold; padding: 6px 10px; font-size: 13px; }
.file-note { padding: 8px 10px; color: #666; font-style: italic; }
.flash { outline: 2px solid #ff9800; }
"""


def _anchor_id(path, line):
    side = "new" if line["new_line"] is not None else "old"
    num = line["new_line"] if line["new_line"] is not None else line["old_line"]
    return f"{path}:{side}:{num}"


def _render_line(file_path, line):
    old_no = line["old_line"] if line["old_line"] is not None else ""
    new_no = line["new_line"] if line["new_line"] is not None else ""
    css_class = {"add": "add", "remove": "remove", "context": ""}[line["type"]]
    anchor_id = html.escape(_anchor_id(file_path, line), quote=True)
    content_html = html.escape(line["content"])
    return (
        f'<div class="diff-line {css_class}" id="{anchor_id}" data-anchor="{anchor_id}">'
        f'<span class="lineno">{html.escape(str(old_no))}</span>'
        f'<span class="lineno">{html.escape(str(new_no))}</span>'
        f'<span class="content">{content_html}</span>'
        f'</div>'
    )


def _render_file_section(file_entry):
    path = html.escape(file_entry["path"])
    status = file_entry["status"]
    if status == "binary":
        body = '<div class="file-note">二进制文件，无法显示 diff</div>'
    elif status == "error":
        body = f'<div class="file-note">⚠️ 解析失败: {html.escape(file_entry["error"] or "")}</div>'
    else:
        body = "".join(_render_line(file_entry["path"], ln) for ln in file_entry["lines"])
    header_extra = ""
    if status == "renamed":
        header_extra = f' (重命名自 {html.escape(file_entry["old_path"] or "")})'
    elif status == "untracked":
        header_extra = " (未跟踪)"
    return (
        f'<div class="file-section" data-file="{path}">'
        f'<div class="file-header">{path}{header_extra}</div>'
        f'{body}'
        f'</div>'
    )


def _render_file_tree(files):
    items = "".join(
        f'<div class="file-item" data-file="{html.escape(f["path"])}">{html.escape(f["path"])}</div>'
        for f in files
    )
    return f'<div id="file-tree">{items}</div>'


def _render_annotation_cards(files):
    cards = []
    for f in files:
        for line in f.get("lines", []):
            anchor_id = html.escape(_anchor_id(f["path"], line), quote=True)
            extra_parts = []
            if f.get("pseudocode"):
                extra_parts.append(f'<details><summary>伪代码</summary>{html.escape(str(f["pseudocode"]))}</details>')
            if f.get("call_tree"):
                extra_parts.append(f'<details><summary>调用树</summary>{html.escape(str(f["call_tree"]))}</details>')
            extra = "".join(extra_parts) or '<span class="empty">暂无说明</span>'
            display_no = line["new_line"] if line["new_line"] is not None else line["old_line"]
            cards.append(
                f'<div class="annotation-card" data-anchor="{anchor_id}">'
                f'<div class="anchor-label">{html.escape(f["path"])}:{display_no}</div>'
                f'{extra}</div>'
            )
    return f'<div id="annotation-pane">{"".join(cards)}</div>'


def render_html(annotations):
    files = annotations.get("files", [])
    file_tree = _render_file_tree(files)
    diff_sections = "".join(_render_file_section(f) for f in files)
    annotation_pane = _render_annotation_cards(files)
    return (
        "<!DOCTYPE html>\n"
        '<html lang="zh-CN">\n<head>\n<meta charset="utf-8">\n'
        "<title>Diff Review</title>\n"
        f"<style>{STYLE}</style>\n</head>\n<body>\n"
        f'<div class="banner">{html.escape(BANNER_TEXT)}</div>\n'
        '<div class="layout">\n'
        f'{file_tree}\n'
        '<div class="resize-handle" id="resize-tree"></div>\n'
        f'<div id="diff-pane">{diff_sections}</div>\n'
        '<div class="resize-handle" id="resize-annotation"></div>\n'
        f'{annotation_pane}\n'
        '</div>\n'
        "</body>\n</html>\n"
    )


def run_render(annotations_path, output_path):
    with open(annotations_path, encoding="utf-8") as f:
        annotations = json.load(f)
    document = render_html(annotations)
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(document)


def main():
    import argparse
    parser = argparse.ArgumentParser(prog="render-diff-review")
    sub = parser.add_subparsers(dest="command", required=True)

    scan_p = sub.add_parser("scan", help="Parse a git diff range into annotations.json")
    scan_p.add_argument("diff_range", help="e.g. 'main..HEAD' or 'abc123..def456'")
    scan_p.add_argument("--output", default=None)

    render_p = sub.add_parser("render", help="Render annotations.json into an HTML review page")
    render_p.add_argument("annotations_path")
    render_p.add_argument("--output", default=None)

    args = parser.parse_args()

    if args.command == "scan":
        output_path = args.output or os.path.join(
            ".cache", "diff-review", f"{_sanitize_range(args.diff_range)}.json"
        )
        run_scan(args.diff_range, output_path)
        print(f"✓ 已生成 {output_path}")
    elif args.command == "render":
        with open(args.annotations_path, encoding="utf-8") as f:
            diff_range = json.load(f).get("diff_range", "review")
        output_path = args.output or os.path.join(
            ".cache", "diff-review", f"{_sanitize_range(diff_range)}.html"
        )
        run_render(args.annotations_path, output_path)
        print(f"✓ 已生成 {output_path}")


if __name__ == "__main__":
    main()
