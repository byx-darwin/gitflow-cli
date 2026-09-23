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

    def test_malformed_nested_field_renders_error_card_not_crash(self):
        # Regression test: a contract can pass _validate_contract() (has
        # workflow_id and a dict "phases") but still carry a malformed
        # nested field type — e.g. "status" as a list instead of a string.
        # This must not crash the whole run; it must isolate to one error
        # card, same as top-level JSON/schema failures.
        with tempfile.TemporaryDirectory() as tmp:
            active_dir = os.path.join(tmp, "active")
            os.makedirs(active_dir)
            self._write_contract(active_dir, "wf-bad-nested.json", {
                "workflow_id": "wf-bad-nested",
                "phases": {
                    "1": {"status": ["complete"]},  # wrong type: list, not str
                    "2": {"status": "pending"},
                    "3": {"status": "pending"},
                    "4": {"status": "pending"},
                },
            })
            self._write_contract(active_dir, "wf-ok.json", {
                "workflow_id": "wf-ok",
                "phases": {
                    "1": {"status": "complete"}, "2": {"status": "complete"},
                    "3": {"status": "complete"}, "4": {"status": "complete"},
                },
            })
            output_path = os.path.join(tmp, "dashboard.html")

            # Must not raise, and must not abort rendering of wf-ok.
            render_dashboard(active_dir, output_path)

            with open(output_path, encoding="utf-8") as f:
                html_content = f.read()

            self.assertIn("解析失败", html_content)
            self.assertIn("wf-bad-nested.json", html_content)
            self.assertIn("wf-ok", html_content)
            self.assertIn("status-complete", html_content)


if __name__ == "__main__":
    unittest.main()
