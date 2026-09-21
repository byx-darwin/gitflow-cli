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


SINGLE_HUNK_DIFF = "\n".join([
    "diff --git a/src/foo.py b/src/foo.py",
    "index 1111111..2222222 100644",
    "--- a/src/foo.py",
    "+++ b/src/foo.py",
    "@@ -1,4 +1,5 @@",
    " def foo():",
    "-    return 1",
    "+    # comment",
    "+    return 2",
    " ",  # blank context line — kept as an explicit list item so a
          # trim-trailing-whitespace hook can't strip its leading space
    " def bar():",
]) + "\n"

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

    def test_rename_without_content_change(self):
        diff = """diff --git a/old_name.py b/new_name.py
similarity index 100%
rename from old_name.py
rename to new_name.py
"""
        files = parse_diff_text(diff)
        self.assertEqual(len(files), 1)
        f = files[0]
        self.assertEqual(f["status"], "renamed")
        self.assertEqual(f["old_path"], "old_name.py")
        self.assertEqual(f["path"], "new_name.py")
        self.assertEqual(f["lines"], [])

    def test_rename_with_content_hunk(self):
        diff = """diff --git a/old2.py b/new2.py
similarity index 87%
rename from old2.py
rename to new2.py
index 5555555..6666666 100644
--- a/old2.py
+++ b/new2.py
@@ -1,2 +1,2 @@
 def f():
-    return 1
+    return 2
"""
        files = parse_diff_text(diff)
        self.assertEqual(len(files), 1)
        f = files[0]
        self.assertEqual(f["status"], "renamed")
        self.assertEqual(f["old_path"], "old2.py")
        self.assertEqual(f["path"], "new2.py")
        self.assertEqual(len(f["lines"]), 3)

    def test_binary_file_marked_and_no_lines(self):
        diff = """diff --git a/image.png b/image.png
index 7777777..8888888 100644
Binary files a/image.png and b/image.png differ
"""
        files = parse_diff_text(diff)
        self.assertEqual(len(files), 1)
        f = files[0]
        self.assertEqual(f["status"], "binary")
        self.assertEqual(f["lines"], [])


import json
import subprocess
import tempfile

scan_untracked_files = render_diff_review.scan_untracked_files


class TestScanUntrackedFiles(unittest.TestCase):
    def test_untracked_file_produces_all_added_entry(self):
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.email", "t@t.com"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.name", "t"], cwd=tmp, check=True)
            with open(os.path.join(tmp, "committed.txt"), "w") as f:
                f.write("hello\n")
            subprocess.run(["git", "add", "committed.txt"], cwd=tmp, check=True)
            subprocess.run(["git", "commit", "-q", "-m", "init"], cwd=tmp, check=True)

            with open(os.path.join(tmp, "new_file.py"), "w") as f:
                f.write("def new():\n    return 1\n")

            cwd = os.getcwd()
            try:
                os.chdir(tmp)
                entries = scan_untracked_files()
            finally:
                os.chdir(cwd)

            self.assertEqual(len(entries), 1)
            self.assertEqual(entries[0]["path"], "new_file.py")
            self.assertEqual(entries[0]["status"], "untracked")
            self.assertTrue(any(l["type"] == "add" for l in entries[0]["lines"]))

    def test_no_untracked_files_returns_empty_list(self):
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.email", "t@t.com"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.name", "t"], cwd=tmp, check=True)
            with open(os.path.join(tmp, "committed.txt"), "w") as f:
                f.write("hello\n")
            subprocess.run(["git", "add", "committed.txt"], cwd=tmp, check=True)
            subprocess.run(["git", "commit", "-q", "-m", "init"], cwd=tmp, check=True)

            cwd = os.getcwd()
            try:
                os.chdir(tmp)
                entries = scan_untracked_files()
            finally:
                os.chdir(cwd)

            self.assertEqual(entries, [])


build_annotations = render_diff_review.build_annotations
run_scan = render_diff_review.run_scan


class TestBuildAnnotations(unittest.TestCase):
    def test_assembles_tracked_and_untracked_with_diff_range(self):
        tracked = [{"path": "a.py", "old_path": None, "status": "modified",
                    "lines": [], "error": None, "pseudocode": None, "call_tree": None}]
        untracked = [{"path": "b.py", "old_path": None, "status": "untracked",
                      "lines": [], "error": None, "pseudocode": None, "call_tree": None}]
        result = build_annotations("main..HEAD", tracked, untracked)
        self.assertEqual(result["diff_range"], "main..HEAD")
        self.assertEqual(result["files"], tracked + untracked)


class TestRunScan(unittest.TestCase):
    def test_malformed_fragment_isolated_valid_files_still_parse(self):
        # Directly exercise parse_diff_text's fault isolation (already covered
        # indirectly by parse_diff_text's own error-entry behavior) via
        # build_annotations + a hand-crafted mixed-validity diff text, to
        # confirm the assembled annotations still contain both the error
        # entry and the valid entry.
        mixed_diff = """diff --git a/good.py b/good.py
index 1111111..2222222 100644
--- a/good.py
+++ b/good.py
@@ -1,1 +1,1 @@
-old
+new
diff --git a/bad.py b/bad.py
this is not a valid diff --git header continuation
@@ not a real hunk header @@
"""
        tracked = render_diff_review.parse_diff_text(mixed_diff)
        annotations = build_annotations("x..y", tracked, [])
        statuses = {f["path"]: f["status"] for f in annotations["files"]}
        self.assertEqual(statuses["good.py"], "modified")
        # the second block's own diff --git header IS well-formed
        # ("diff --git a/bad.py b/bad.py"), but its hunk line is malformed —
        # the malformed hunk header must not crash the whole scan, and must
        # surface as an error entry for bad.py specifically.
        self.assertEqual(statuses["bad.py"], "error")
        self.assertIsNotNone(
            next(f for f in annotations["files"] if f["path"] == "bad.py")["error"]
        )

    def test_run_scan_writes_annotations_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            subprocess.run(["git", "init", "-q"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.email", "t@t.com"], cwd=tmp, check=True)
            subprocess.run(["git", "config", "user.name", "t"], cwd=tmp, check=True)
            with open(os.path.join(tmp, "f.txt"), "w") as f:
                f.write("line1\n")
            subprocess.run(["git", "add", "f.txt"], cwd=tmp, check=True)
            subprocess.run(["git", "commit", "-q", "-m", "init"], cwd=tmp, check=True)
            with open(os.path.join(tmp, "f.txt"), "w") as f:
                f.write("line1 changed\n")
            subprocess.run(["git", "add", "f.txt"], cwd=tmp, check=True)
            subprocess.run(["git", "commit", "-q", "-m", "change"], cwd=tmp, check=True)

            output_path = os.path.join(tmp, "out.json")
            cwd = os.getcwd()
            try:
                os.chdir(tmp)
                run_scan("HEAD~1..HEAD", output_path)
            finally:
                os.chdir(cwd)

            self.assertTrue(os.path.exists(output_path))
            with open(output_path, encoding="utf-8") as f:
                data = json.load(f)
            self.assertEqual(data["diff_range"], "HEAD~1..HEAD")
            self.assertEqual(len(data["files"]), 1)
            self.assertEqual(data["files"][0]["path"], "f.txt")


if __name__ == "__main__":
    unittest.main()
