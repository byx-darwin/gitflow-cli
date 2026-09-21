# Workflow Dashboard Renderer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增一个零依赖的 Python3 脚本，把 `.cache/workflows/active/*.json` 派生为单文件、自包含、离线可打开的 HTML 进度看板，并通过 Makefile 目标触发。

**Architecture:** 核心渲染逻辑写成可导入的纯函数 `render_dashboard(input_dir, output_path)`，脚本入口只是用固定常量调用它；这样测试脚本可以直接 import 并用临时目录跑，不需要改动"无 CLI 参数"的设计约束。Makefile 新增一个单行目标调用脚本入口。

**Tech Stack:** Python3 标准库（`json`、`html`、`glob`、`os`、`sys`），不引入任何第三方包。

**Spec:** `docs/superpowers/specs/2026-09-21-gf-workflow-dashboard-design.md`

## Global Constraints

- 零第三方 Python 依赖——全程标准库，不出现任何 `pip install`。
- 脚本对 `.cache/workflows/active/` 只读，不修改/移动/删除该目录下任何文件。
- `.cache/workflows/dashboard.html` 绝不提交进仓库——`.cache/` 已被根 `.gitignore` 覆盖，只需验证，不重新编辑 `.gitignore`。
- 不做 `--watch` 或常驻服务模式——设计 §3 已明确排除；`<meta refresh=5>` 是唯一的"自动更新"机制，且是被动的（依赖外部重跑生成命令，脚本自己不触发重跑）。
- 恰好 2 个 Task（脚本+测试合并为 Task 1；Makefile 目标为 Task 2）——测试文件和它测试的脚本属于同一个测试周期，不拆成单独 Task。

---

## Task 1: `scripts/render-workflow-dashboard.py` + 测试

**Files:**
- Create: `scripts/render-workflow-dashboard.py`
- Create: `scripts/tests/test_render_workflow_dashboard.py`

**Interfaces:**
- Produces: `render_dashboard(input_dir: str, output_path: str) -> None`（模块级函数，读 `input_dir` 下所有 `*.json`，写渲染结果到 `output_path`）。供 Task 1 自己的测试脚本 import 使用；不供 Task 2 使用（Task 2 只调用整个脚本作为子进程，不 import 它的函数）。

- [ ] **Step 1: 编写测试脚本（先写测试，后写实现）**

创建 `scripts/tests/test_render_workflow_dashboard.py`：

```python
#!/usr/bin/env python3
"""Standalone test script for render-workflow-dashboard.py.

Not run by `make test` (Rust/nextest only) — run directly:
    python3 scripts/tests/test_render_workflow_dashboard.py
"""
import json
import os
import sys
import tempfile
import unittest

# Import the module under test by path, since scripts/ is not a package.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import importlib.util

_spec = importlib.util.spec_from_file_location(
    "render_workflow_dashboard",
    os.path.join(os.path.dirname(__file__), "..", "render-workflow-dashboard.py"),
)
render_workflow_dashboard = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(render_workflow_dashboard)
render_dashboard = render_workflow_dashboard.render_dashboard


class TestRenderDashboard(unittest.TestCase):
    def _write_contract(self, active_dir, filename, content):
        path = os.path.join(active_dir, filename)
        with open(path, "w", encoding="utf-8") as f:
            if isinstance(content, str):
                f.write(content)
            else:
                json.dump(content, f)
        return path

    def test_valid_contract_renders_status_classes_and_fields(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-valid.json", {
                "workflow_id": "wf-valid",
                "title": "Valid <Workflow> & Title",
                "mode": "full",
                "current_phase": 2,
                "phases": {
                    "1": {"name": "Clarification", "status": "complete",
                          "started_at": "2026-09-21T00:00:00Z",
                          "completed_at": "2026-09-21T01:00:00Z",
                          "executor": "main-agent"},
                    "2": {"name": "Planning", "status": "in_progress",
                          "started_at": "2026-09-21T01:00:00Z",
                          "completed_at": None, "executor": None},
                    "3": {"name": "Execution", "status": "pending"},
                    "4": {"name": "Delivery", "status": "skipped"},
                },
            })
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            self.assertTrue(os.path.exists(output_path))
            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            # Status classes for all 4 distinct statuses used above.
            self.assertIn("status-complete", html_content)
            self.assertIn("status-in-progress", html_content)
            self.assertIn("status-pending", html_content)
            self.assertIn("status-skipped", html_content)
            # Title must be HTML-escaped (raw <Workflow> must not appear unescaped).
            self.assertNotIn("<Workflow>", html_content)
            self.assertIn("&lt;Workflow&gt;", html_content)
            # Banner text present verbatim.
            self.assertIn(
                "⚠️ 派生视图 — 请勿手工编辑，由 make render-workflow-dashboard "
                "从 .cache/workflows/active/*.json 生成",
                html_content,
            )
            # Auto-refresh meta tag present.
            self.assertIn('http-equiv="refresh"', html_content)
            self.assertIn('content="5"', html_content)

    def test_missing_optional_fields_fall_back_to_em_dash(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-sparse.json", {
                "workflow_id": "wf-sparse",
                "phases": {
                    "1": {"status": "pending"},
                    "2": {"status": "pending"},
                    "3": {"status": "pending"},
                    "4": {"status": "pending"},
                },
            })
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("—", html_content)
            # Missing title falls back to workflow_id.
            self.assertIn("wf-sparse", html_content)

    def test_malformed_json_renders_error_card_not_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-broken.json", "{not valid json")
            output_path = os.path.join(tmp, "dashboard.html")

            # Must not raise.
            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("解析失败", html_content)
            self.assertIn("wf-broken.json", html_content)

    def test_missing_required_key_renders_error_card(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            # Valid JSON, but missing the required "phases" key.
            self._write_contract(active_dir, "wf-no-phases.json", {
                "workflow_id": "wf-no-phases",
            })
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("解析失败", html_content)
            self.assertIn("wf-no-phases.json", html_content)

    def test_one_bad_file_does_not_block_other_valid_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-broken.json", "{not valid json")
            self._write_contract(active_dir, "wf-ok.json", {
                "workflow_id": "wf-ok",
                "phases": {
                    "1": {"status": "complete"}, "2": {"status": "complete"},
                    "3": {"status": "complete"}, "4": {"status": "complete"},
                },
            })
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("解析失败", html_content)
            self.assertIn("wf-ok", html_content)
            self.assertIn("status-complete", html_content)

    def test_empty_active_dir_renders_friendly_message(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("当前没有活跃工作流", html_content)

    def test_nonexistent_active_dir_renders_friendly_message(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")  # never created
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("当前没有活跃工作流", html_content)

    def test_unknown_status_gets_unknown_class(self):
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-weird.json", {
                "workflow_id": "wf-weird",
                "phases": {
                    "1": {"status": "some_future_status"},
                    "2": {"status": "pending"},
                    "3": {"status": "pending"},
                    "4": {"status": "pending"},
                },
            })
            output_path = os.path.join(tmp, "dashboard.html")

            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("status-unknown", html_content)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: 运行测试，确认失败（模块尚不存在）**

Run: `python3 scripts/tests/test_render_workflow_dashboard.py`
Expected: `ModuleNotFoundError` 或 `FileNotFoundError`（`render-workflow-dashboard.py` 还不存在），测试收集阶段就失败。

- [ ] **Step 3: 编写 `scripts/render-workflow-dashboard.py`**

```python
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
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_workflow_dashboard.py`
Expected: `OK`（全部 8 个测试通过），退出码 0。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-workflow-dashboard.py scripts/tests/test_render_workflow_dashboard.py
git commit -m "feat(scripts): add workflow contract dashboard renderer"
```

---

## Task 2: Makefile 目标

**Files:**
- Modify: `Makefile`（在 `check-agent-sync` 目标的三行 `@bash scripts/...` 之后插入新目标；`.PHONY` 列表追加 `render-workflow-dashboard`）

**Interfaces:**
- Consumes：Task 1 产出的 `scripts/render-workflow-dashboard.py`（作为子进程调用，不 import）。

- [ ] **Step 1: 在 `check-agent-sync` 目标之后插入新目标**

当前（`Makefile` 里 `check-agent-sync` 目标结尾）：

```makefile
check-agent-sync: ## Verify agent instructions exist and skill docs stay consistent
	@test -f CLAUDE.md || { \
		echo "✗ CLAUDE.md is required for project-level agent instructions."; \
		exit 1; \
	}
	@echo "✓ CLAUDE.md 存在"
	@bash scripts/verify-skills-when-not-to-use.sh
	@bash scripts/validate-skill-commands.sh
	@bash scripts/validate-skill-links.sh
```

改为（追加空行 + 新目标）：

```makefile
check-agent-sync: ## Verify agent instructions exist and skill docs stay consistent
	@test -f CLAUDE.md || { \
		echo "✗ CLAUDE.md is required for project-level agent instructions."; \
		exit 1; \
	}
	@echo "✓ CLAUDE.md 存在"
	@bash scripts/verify-skills-when-not-to-use.sh
	@bash scripts/validate-skill-commands.sh
	@bash scripts/validate-skill-links.sh

render-workflow-dashboard: ## 从 .cache/workflows/active/*.json 生成派生的 HTML 进度看板（勿手编产物）
	@python3 scripts/render-workflow-dashboard.py
	@echo "✓ 已生成 .cache/workflows/dashboard.html"
```

- [ ] **Step 2: `.PHONY` 列表追加目标名**

当前（`Makefile` 末尾附近）：

```makefile
.PHONY: help build build-release local-install check run test test-watch fmt clippy lint audit sbom install-tools install-skills install-hooks install \
        list-skills uninstall-skills completions completions-install completions-uninstall \
        watch bench bench-cli coverage docs release-dry-run \
        update-submodule check-agent-sync check-smell-skill check-refactor-skill check-architecture-diagram-skill check-walkthrough-skill check-quality-review-evidence-skill check-decompose-skill check-skills-drift release release-quick release-rehearse \
        smoke-test smoke-test-github smoke-test-gitlab smoke-test-gitcode smoke-test-write completions-install completions-uninstall changelog release-push release-publish package
```

改为（在 `check-skills-drift` 之后追加 `render-workflow-dashboard`）：

```makefile
.PHONY: help build build-release local-install check run test test-watch fmt clippy lint audit sbom install-tools install-skills install-hooks install \
        list-skills uninstall-skills completions completions-install completions-uninstall \
        watch bench bench-cli coverage docs release-dry-run \
        update-submodule check-agent-sync check-smell-skill check-refactor-skill check-architecture-diagram-skill check-walkthrough-skill check-quality-review-evidence-skill check-decompose-skill check-skills-drift render-workflow-dashboard release release-quick release-rehearse \
        smoke-test smoke-test-github smoke-test-gitlab smoke-test-gitcode smoke-test-write completions-install completions-uninstall changelog release-push release-publish package
```

- [ ] **Step 3: 验证——对本仓库真实的 `.cache/workflows/active/` 跑一次**

```bash
make render-workflow-dashboard
```

预期：退出码 0，输出 `✓ 已生成 .cache/workflows/dashboard.html`，文件 `.cache/workflows/dashboard.html` 被创建（会包含本工作流自身 `wf-2026-09-21-005` 的进行中卡片，这是预期行为，不是 bug）。

```bash
make check-agent-sync
```

预期：与本次改动前完全一致的基线（Commands=78 Files=29 Refs=194 Mismatches=0）——本 Task 不触碰任何 skill 文件。

```bash
git check-ignore .cache/workflows/dashboard.html
```

预期：命令成功（退出码 0）并原样打印路径，证明该产物确实被 `.gitignore` 覆盖，不会被误提交。

- [ ] **Step 4: Commit**

```bash
git add Makefile
git commit -m "feat(makefile): add render-workflow-dashboard target"
```

---

## Final Validation

```bash
make render-workflow-dashboard && make check-agent-sync
python3 scripts/tests/test_render_workflow_dashboard.py
```

预期：三条命令全部成功；`make check-agent-sync` 基线不变；测试脚本 8 项全过。人工用浏览器打开 `.cache/workflows/dashboard.html`（`file://` 协议本地打开）确认离线渲染正常、无控制台报错、无外部资源加载尝试。
