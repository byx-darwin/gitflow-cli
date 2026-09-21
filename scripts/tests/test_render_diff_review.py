#!/usr/bin/env python3
"""Standalone test script for render-diff-review.py.

Not run by `make test` (Rust/nextest only) — run directly:
    python3 scripts/tests/test_render_diff_review.py
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import importlib.util

_spec = importlib.util.spec_from_file_location(
    "render_diff_review",
    os.path.join(os.path.dirname(__file__), "..", "render-diff-review.py"),
)
render_diff_review = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(render_diff_review)

parse_diff_text = render_diff_review.parse_diff_text


SINGLE_HUNK_DIFF = """diff --git a/src/foo.py b/src/foo.py
index 1111111..2222222 100644
--- a/src/foo.py
+++ b/src/foo.py
@@ -1,4 +1,5 @@
 def foo():
-    return 1
+    # comment
+    return 2

 def bar():
"""

MULTI_HUNK_DIFF = """diff --git a/src/multi.py b/src/multi.py
index 3333333..4444444 100644
--- a/src/multi.py
+++ b/src/multi.py
@@ -1,3 +1,3 @@
 def a():
-    return 1
+    return 10
@@ -10,3 +10,3 @@
 def b():
-    return 2
+    return 20
"""

MULTI_FILE_DIFF = SINGLE_HUNK_DIFF + MULTI_HUNK_DIFF


class TestParseDiffText(unittest.TestCase):
    def test_single_hunk_dual_line_numbers(self):
        files = parse_diff_text(SINGLE_HUNK_DIFF)
        self.assertEqual(len(files), 1)
        f = files[0]
        self.assertEqual(f["path"], "src/foo.py")
        self.assertEqual(f["status"], "modified")
        self.assertIsNone(f["old_path"])
        self.assertIsNone(f["error"])

        lines = f["lines"]
        # context line "def foo():" -> old_line=1, new_line=1
        self.assertEqual(lines[0], {
            "old_line": 1, "new_line": 1, "type": "context", "content": "def foo():",
        })
        # removed line "    return 1" -> old_line=2, new_line=None
        self.assertEqual(lines[1], {
            "old_line": 2, "new_line": None, "type": "remove", "content": "    return 1",
        })
        # added line "    # comment" -> old_line=None, new_line=2
        self.assertEqual(lines[2], {
            "old_line": None, "new_line": 2, "type": "add", "content": "    # comment",
        })
        # added line "    return 2" -> old_line=None, new_line=3
        self.assertEqual(lines[3], {
            "old_line": None, "new_line": 3, "type": "add", "content": "    return 2",
        })

    def test_multi_hunk_per_file_resets_counters_correctly(self):
        files = parse_diff_text(MULTI_HUNK_DIFF)
        self.assertEqual(len(files), 1)
        lines = files[0]["lines"]
        # first hunk starts at old=1/new=1
        self.assertEqual(lines[0]["old_line"], 1)
        # second hunk header says old=10/new=10 — must reset, not continue counting
        # find the context line "def b():" which should be old=10, new=10
        def_b = next(l for l in lines if l["content"] == "def b():")
        self.assertEqual(def_b["old_line"], 10)
        self.assertEqual(def_b["new_line"], 10)

    def test_multi_file_diff_produces_two_entries(self):
        files = parse_diff_text(MULTI_FILE_DIFF)
        self.assertEqual(len(files), 2)
        self.assertEqual(files[0]["path"], "src/foo.py")
        self.assertEqual(files[1]["path"], "src/multi.py")

    def test_empty_diff_text_returns_empty_list(self):
        self.assertEqual(parse_diff_text(""), [])
        self.assertEqual(parse_diff_text("   \n  "), [])


if __name__ == "__main__":
    unittest.main()
