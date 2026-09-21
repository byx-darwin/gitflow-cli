#!/usr/bin/env python3
"""Derive a single-file HTML dashboard from .cache/workflows/active/*.json.

This is a DERIVED VIEW — never hand-edit the generated HTML. Re-run
`make render-workflow-dashboard` to refresh it after a contract changes.

Standard-library only, no third-party dependencies.
"""
import glob
import html
import json
import os
import sys

INPUT_DIR = os.path.join(".cache", "workflows", "active")
OUTPUT_PATH = os.path.join(".cache", "workflows", "dashboard.html")

STATUS_CLASSES = {
    "complete": "status-complete",
    "in_progress": "status-in-progress",
    "blocked": "status-blocked",
    "skipped": "status-skipped",
    "pending": "status-pending",
}

BANNER_TEXT = (
    "⚠️ 派生视图 — 请勿手工编辑，由 make render-workflow-dashboard "
    "从 .cache/workflows/active/*.json 生成"
)

STYLE = """
body { font-family: -apple-system, sans-serif; background: #f4f4f6; margin: 0; padding: 24px; }
.banner { background: #fff3cd; border: 1px solid #ffe08a; color: #7a5b00;
          padding: 12px 16px; border-radius: 6px; margin-bottom: 20px; font-weight: bold; }
.card { background: #fff; border: 1px solid #ddd; border-radius: 8px;
        padding: 16px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.card h2 { margin: 0 0 4px 0; font-size: 18px; }
.card .meta { color: #666; font-size: 13px; margin-bottom: 12px; }
.phases { display: flex; gap: 12px; flex-wrap: wrap; }
.phase { flex: 1; min-width: 160px; border-radius: 6px; padding: 10px; }
.phase .name { font-weight: bold; }
.phase .fields { font-size: 12px; margin-top: 6px; color: #333; }
.status-complete { background: #d4edda; border: 1px solid #86c98f; }
.status-in-progress { background: #cce5ff; border: 1px solid #7cb3f5; }
.status-blocked { background: #f8d7da; border: 1px solid #ea9aa2; }
.status-skipped { background: #e9ecef; border: 1px solid #ced4da; font-style: italic; }
.status-pending { background: #f1f1f3; border: 1px solid #ddd; }
.status-unknown { background: #ffe8cc; border: 1px solid #ffbb66; }
.error-card { background: #fff0f0; border: 1px solid #e08a8a; }
.empty { color: #666; font-style: italic; }
"""


def _phase_html(phase_num, phase):
    name = html.escape(str(phase.get("name") or f"Phase {phase_num}"))
    status = phase.get("status") or "pending"
    css_class = STATUS_CLASSES.get(status, "status-unknown")
    started = html.escape(str(phase.get("started_at") or "—"))
    completed = html.escape(str(phase.get("completed_at") or "—"))
    executor = html.escape(str(phase.get("executor") or "—"))
    status_escaped = html.escape(str(status))
    return (
        f'<div class="phase {css_class}">'
        f'<div class="name">{name} — {status_escaped}</div>'
        f'<div class="fields">start: {started}<br>end: {completed}<br>'
        f'executor: {executor}</div>'
        f"</div>"
    )


def _card_html(contract):
    title = html.escape(str(contract.get("title") or contract.get("workflow_id") or "unknown"))
    mode = html.escape(str(contract.get("mode") or "unknown"))
    current_phase = html.escape(str(contract.get("current_phase") if
                                     contract.get("current_phase") is not None else "?"))
    phases = contract.get("phases") or {}
    phase_cells = "".join(
        _phase_html(n, phases.get(str(n)) or {}) for n in (1, 2, 3, 4)
    )
    return (
        '<div class="card">'
        f"<h2>{title}</h2>"
        f'<div class="meta">mode: {mode} · current_phase: {current_phase}</div>'
        f'<div class="phases">{phase_cells}</div>'
        "</div>"
    )


def _error_card_html(filename, message):
    return (
        '<div class="card error-card">'
        f"<h2>⚠️ 解析失败: {html.escape(filename)}</h2>"
        f"<div class=\"meta\">{html.escape(message)}</div>"
        "</div>"
    )


def _validate_contract(data):
    if not isinstance(data, dict):
        raise ValueError("top-level JSON is not an object")
    if "workflow_id" not in data:
        raise ValueError("missing required key: workflow_id")
    if "phases" not in data or not isinstance(data["phases"], dict):
        raise ValueError("missing or invalid required key: phases")


def render_dashboard(input_dir, output_path):
    cards = []
    pattern = os.path.join(input_dir, "*.json")
    files = sorted(glob.glob(pattern))

    for path in files:
        filename = os.path.basename(path)
        try:
            with open(path, encoding="utf-8") as f:
                data = json.load(f)
            _validate_contract(data)
        except (json.JSONDecodeError, ValueError, OSError) as exc:
            cards.append(_error_card_html(filename, str(exc)))
            continue
        cards.append(_card_html(data))

    if cards:
        body = "".join(cards)
    else:
        body = '<p class="empty">当前没有活跃工作流</p>'

    document = (
        "<!DOCTYPE html>\n"
        '<html lang="zh-CN">\n<head>\n<meta charset="utf-8">\n'
        '<meta http-equiv="refresh" content="5">\n'
        "<title>gf-workflow Dashboard</title>\n"
        f"<style>{STYLE}</style>\n</head>\n<body>\n"
        f'<div class="banner">{html.escape(BANNER_TEXT)}</div>\n'
        f"{body}\n</body>\n</html>\n"
    )

    try:
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(document)
    except OSError as exc:
        print(f"render-workflow-dashboard: failed to write {output_path}: {exc}",
              file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    render_dashboard(INPUT_DIR, OUTPUT_PATH)
