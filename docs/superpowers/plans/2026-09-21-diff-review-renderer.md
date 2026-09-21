# diff 交互式审阅网页渲染 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 `scripts/render-diff-review.py`（scan/render 两个子命令），把 `git diff` 渲染成一个自包含、可离线打开、支持行级锚定/hover联动/折叠/拖拽记忆宽度的交互式审阅 HTML 页面。

**Architecture:** 单文件 Python3 标准库脚本，`scan` 手写 unified diff 解析器产出 `annotations.json`（含未跟踪文件、重命名、二进制识别，逐文件容错），`render` 把该 JSON 字符串模板注入内嵌的 HTML/CSS/自研语法高亮/原生 JS，产出单文件 HTML。测试用 standalone `unittest`（同 #345 的 `scripts/tests/` 惯例），不进 `make test`。

**Tech Stack:** Python3 标准库（`argparse`、`json`、`html`、`subprocess`、`re`、`os`、`sys`），零第三方依赖；纯原生 JS/CSS，无框架、无构建步骤、无 CDN。

**Spec:** `docs/superpowers/specs/2026-09-21-diff-review-renderer-design.md`

## Global Constraints

- 零第三方 Python 依赖——全程标准库，不出现 `pip install`。
- 零第三方 JS/CSS——不引 CDN、不打包第三方库（明确不用 Prism），无构建工具（无 npm/webpack/vite），纯原生 JS 和手写 CSS，全部内联在产出的 HTML 里。
- 所有源自 diff/文件系统的字符串数据在插入模板前必须经 `html.escape()` 转义——与 #345 Phase 4 审查验证过的安全基线一致。
- `.cache/diff-review/` 产物绝不提交——已被现有 `.gitignore` 里的 `.cache/` 行覆盖，只需验证不需要改 `.gitignore`。
- 对仓库只读：`scan`/`render` 绝不修改被跟踪文件，绝不在 `.cache/diff-review/` 之外写任何东西（中间产物 `annotations.json` 与最终 HTML 都落在 `.cache/diff-review/` 下）。
- 产物中不得出现任何"接受/拒绝/写回"入口——单向只读展示。
- 只产出设计 §9 列出的文件：`scripts/render-diff-review.py`、`scripts/tests/test_render_diff_review.py`、`Makefile`（改动）。
- pseudocode/call_tree 字段挂在**文件级**（不是行级）——因为本 Issue 交付时这两个字段永远是 `None`（内容生成拆到 #398），行级挂载对当前空值场景没有额外价值，文件级更符合 schema 的自然粒度。

---

## Task 1: Unified diff 解析器核心（标准 hunk）

**Files:**
- Create: `scripts/render-diff-review.py`
- Create: `scripts/tests/test_render_diff_review.py`

**Interfaces:**
- Produces: `parse_diff_text(diff_text: str) -> list[dict]`。每个 file entry 的形状（后续所有 Task 复用，字段名固定）：
  ```python
  {
      "path": str,           # 当前路径（b 侧）
      "old_path": str|None,  # 重命名时的旧路径，其余情况为 None
      "status": str,         # "modified" | "added" | "deleted" | "renamed" | "binary" | "untracked" | "error"
      "lines": list[dict],   # line entry 列表，binary/error 状态为空列表
      "error": str|None,     # status=="error" 时的报错信息
      "pseudocode": None,    # 本计划全程为 None（内容生成拆到 #398）
      "call_tree": None,     # 同上
  }
  ```
  line entry 形状：
  ```python
  {
      "old_line": int|None,  # 删除/上下文行有值，新增行为 None
      "new_line": int|None,  # 新增/上下文行有值，删除行为 None
      "type": str,           # "add" | "remove" | "context"
      "content": str,        # 该行文本（不含 diff 前缀的 +/-/空格）
  }
  ```
  Task 1 本身只处理"标准修改文件"（无重命名、无二进制）的 block；Task 2/3 会扩展同一个 `_parse_file_block` 函数覆盖更多形态。

- [ ] **Step 1: 写测试脚本骨架 + 标准 hunk 测试**

创建 `scripts/tests/test_render_diff_review.py`：

```python
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
```

- [ ] **Step 2: 运行测试，确认失败（模块尚不存在）**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `FileNotFoundError`（`render-diff-review.py` 还不存在）。

- [ ] **Step 3: 实现解析器核心**

创建 `scripts/render-diff-review.py`：

```python
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
```

- [ ] **Step 4: 运行测试，确认通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（4 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): add unified diff parser core for diff review renderer"
```

---

## Task 2: 重命名识别

**Files:**
- Modify: `scripts/render-diff-review.py`（`_parse_file_block` 函数）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：Task 1 的 `_new_file_entry`、`_parse_hunks`、file entry 形状（`old_path`/`status` 字段本任务开始真正被赋非默认值）。
- Produces：`_parse_file_block` 扩展后能正确识别 `status="renamed"` 并填充 `old_path`。不新增函数名。

- [ ] **Step 1: 追加重命名测试**

在 `scripts/tests/test_render_diff_review.py` 的 `TestParseDiffText` 类中追加（`if __name__` 块之前）：

```python
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
```

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: 前 4 项通过，新增 2 项 `FAIL`（`status` 仍是 `"modified"`，`old_path` 仍是 `None`）。

- [ ] **Step 3: 扩展 `_parse_file_block` 识别重命名**

当前：

```python
def _parse_file_block(block_text):
    lines = block_text.splitlines()
    if not lines:
        raise ValueError("empty diff block")
    m = DIFF_GIT_RE.match(lines[0])
    if not m:
        raise ValueError(f"malformed diff --git header: {lines[0]!r}")
    _, b_path = m.group(1), m.group(2)

    entry = _new_file_entry(b_path)

    hunk_start = None
    for idx, line in enumerate(lines):
        if line.startswith("@@"):
            hunk_start = idx
            break
    if hunk_start is not None:
        entry["lines"] = _parse_hunks(lines, hunk_start)

    return entry
```

改为：

```python
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
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（6 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): detect renamed files in diff parser"
```

---

## Task 3: 二进制文件识别 + 未跟踪文件

**Files:**
- Modify: `scripts/render-diff-review.py`（`_parse_file_block` 函数 + 新增 `scan_untracked_files`）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：Task 1 的 `parse_diff_text`（未跟踪文件复用它解析 `git diff --no-index` 的输出，不是单独代码路径）。
- Produces：`scan_untracked_files() -> list[dict]`（形状同 file entry，`status` 固定为 `"untracked"`）；`_parse_file_block` 扩展后能识别 `status="binary"`。

- [ ] **Step 1: 追加二进制文件测试**

在 `scripts/tests/test_render_diff_review.py` 的 `TestParseDiffText` 类中追加：

```python
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
```

再新增一个独立的测试类（同文件内，`TestParseDiffText` 之后、`if __name__` 之前）覆盖 `scan_untracked_files`：这个函数依赖真实的 git 仓库状态，用 `tempfile` 建一个真实的 scratch git repo来测试，不 mock subprocess：

```python
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
```

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `AttributeError: module 'render_diff_review' has no attribute 'scan_untracked_files'`（二进制测试也会 FAIL，因为 status 仍是 `"modified"`）。

- [ ] **Step 3: 扩展解析器识别二进制文件 + 实现未跟踪文件扫描**

当前 `_parse_file_block`（在 Task 2 的重命名识别之后）：

```python
    for line in lines[1:6]:
        if line.startswith("rename from "):
            entry["old_path"] = line[len("rename from "):]
            entry["status"] = "renamed"
        elif line.startswith("rename to "):
            entry["path"] = line[len("rename to "):]

    hunk_start = None
```

改为（在重命名检测和 hunk 检测之间插入二进制检测）：

```python
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
```

在 `parse_diff_text` 函数之后（`if __name__` 之前）新增：

```python
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
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（9 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): detect binary files and scan untracked files"
```

---

## Task 4: annotations.json 组装 + scan CLI

**Files:**
- Modify: `scripts/render-diff-review.py`（新增 `build_annotations`、`run_scan`、`main` 的 `scan` 分支）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：Task 1-3 的 `parse_diff_text`、`scan_untracked_files`。
- Produces：`build_annotations(diff_range: str, tracked_files: list[dict], untracked_files: list[dict]) -> dict`，返回形状：
  ```python
  {
      "diff_range": str,     # 原样保留用户传入的 range 字符串，如 "abc123..def456"
      "files": list[dict],   # tracked_files + untracked_files 拼接
  }
  ```
  `run_scan(diff_range: str, output_path: str) -> None`：跑 `git diff <diff_range>`，解析，组装，写 JSON 到 `output_path`。供 Task 8 的 CLI 调用，也供 Task 5 的测试直接调用而不经过 CLI。

- [ ] **Step 1: 追加测试**

在 `scripts/tests/test_render_diff_review.py` 追加（`TestScanUntrackedFiles` 类之后）：

```python
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
```

在文件顶部 import 区域追加 `import json`（如果之前测试代码没引入过，检查一下当前文件顶部是否已有该 import，没有则加在 `import subprocess` 旁边）。

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `AttributeError`（`build_annotations`/`run_scan` 尚不存在）。

- [ ] **Step 3: 实现 `build_annotations`、`run_scan`，并接上 CLI**

在 `scripts/render-diff-review.py` 的 `scan_untracked_files` 函数之后（`if __name__` 之前）新增：

```python
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
```

把文件末尾的 `if __name__ == "__main__":` 块替换为：

```python
def main():
    import argparse
    parser = argparse.ArgumentParser(prog="render-diff-review")
    sub = parser.add_subparsers(dest="command", required=True)

    scan_p = sub.add_parser("scan", help="Parse a git diff range into annotations.json")
    scan_p.add_argument("diff_range", help="e.g. 'main..HEAD' or 'abc123..def456'")
    scan_p.add_argument("--output", default=None)

    args = parser.parse_args()

    if args.command == "scan":
        output_path = args.output or os.path.join(
            ".cache", "diff-review", f"{_sanitize_range(args.diff_range)}.json"
        )
        run_scan(args.diff_range, output_path)
        print(f"✓ 已生成 {output_path}")


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（12 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): assemble annotations.json and wire up scan CLI"
```

---

## Task 5: HTML 模板结构（三栏布局 + 行渲染，暂不含 JS 交互）

**Files:**
- Modify: `scripts/render-diff-review.py`（新增 `BANNER_TEXT`、`STYLE`、`_render_line`、`_render_file_section`、`_render_file_tree`、`_render_annotation_cards`、`render_html`、`run_render`、`main` 的 `render` 分支）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：Task 4 的 annotations 形状（`{"diff_range": str, "files": [file entry, ...]}`）。
- Produces：`render_html(annotations: dict) -> str`（返回完整 HTML 文本）；`run_render(annotations_path: str, output_path: str) -> None`。Task 6 会在 `_render_line` 内部接入语法高亮（本 Task 先用纯转义、不高亮的占位实现，Task 6 替换掉）；Task 7 会在 `render_html` 末尾追加 `<script>` 块。

- [ ] **Step 1: 追加测试**

在 `scripts/tests/test_render_diff_review.py` 追加：

```python
render_html = render_diff_review.render_html
run_render = render_diff_review.run_render


SAMPLE_ANNOTATIONS = {
    "diff_range": "main..HEAD",
    "files": [
        {
            "path": "src/foo.py",
            "old_path": None,
            "status": "modified",
            "lines": [
                {"old_line": 1, "new_line": 1, "type": "context", "content": "def foo():"},
                {"old_line": 2, "new_line": None, "type": "remove", "content": "    return 1"},
                {"old_line": None, "new_line": 2, "type": "add", "content": "    return 2"},
            ],
            "error": None, "pseudocode": None, "call_tree": None,
        },
        {
            "path": "assets/logo.png",
            "old_path": None,
            "status": "binary",
            "lines": [], "error": None, "pseudocode": None, "call_tree": None,
        },
        {
            "path": "broken.py",
            "old_path": None,
            "status": "error",
            "lines": [], "error": "malformed hunk header", "pseudocode": None, "call_tree": None,
        },
    ],
}


class TestRenderHtml(unittest.TestCase):
    def test_produces_three_pane_structure(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn('id="file-tree"', out)
        self.assertIn('id="diff-pane"', out)
        self.assertIn('id="annotation-pane"', out)

    def test_diff_line_anchors_match_line_numbers(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn('src/foo.py:new:1', out)
        self.assertIn('src/foo.py:old:2', out)
        self.assertIn('src/foo.py:new:2', out)

    def test_add_remove_css_classes_present(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn('diff-line add', out)
        self.assertIn('diff-line remove', out)

    def test_binary_file_shows_note_not_lines(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn("二进制文件", out)

    def test_error_file_shows_error_message(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn("解析失败", out)
        self.assertIn("malformed hunk header", out)

    def test_banner_present_verbatim(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn(
            "⚠️ 派生视图 — 请勿手工编辑，由 make render-diff-review 从 git diff 生成",
            out,
        )

    def test_no_cdn_script_tags(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertNotIn('src="http://', out)
        self.assertNotIn('src="https://', out)
        self.assertNotIn("cdn.", out.lower())
        self.assertNotIn("prism", out.lower())

    def test_html_injection_is_escaped(self):
        malicious = {
            "diff_range": "x..y",
            "files": [{
                "path": "<script>alert(1)</script>.py",
                "old_path": None, "status": "modified",
                "lines": [{"old_line": 1, "new_line": 1, "type": "context",
                           "content": "<img src=x onerror=alert(2)>"}],
                "error": None, "pseudocode": None, "call_tree": None,
            }],
        }
        out = render_html(malicious)
        self.assertNotIn("<script>alert(1)</script>", out)
        self.assertNotIn("<img src=x onerror=alert(2)>", out)
        self.assertIn("&lt;script&gt;", out)


class TestRunRender(unittest.TestCase):
    def test_run_render_writes_html_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            annotations_path = os.path.join(tmp, "annotations.json")
            with open(annotations_path, "w", encoding="utf-8") as f:
                json.dump(SAMPLE_ANNOTATIONS, f)
            output_path = os.path.join(tmp, "out.html")

            run_render(annotations_path, output_path)

            self.assertTrue(os.path.exists(output_path))
            with open(output_path, encoding="utf-8") as f:
                content = f.read()
            self.assertIn('id="file-tree"', content)
```

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `AttributeError`（`render_html`/`run_render` 尚不存在）。

- [ ] **Step 3: 实现模板渲染**

在 `scripts/render-diff-review.py` 顶部的 `import` 区域追加 `import os`（如尚未有）；在 `build_annotations`/`_sanitize_range` 定义之后（`run_scan` 之前或之后均可，放在 `run_scan` 之后、`main` 函数之前）新增：

```python
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
```

把 `main()` 函数替换为（新增 `render` 子命令分支）：

```python
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
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（21 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): render three-pane HTML structure from annotations"
```

---

## Task 6: 自研轻量语法高亮

**Files:**
- Modify: `scripts/render-diff-review.py`（新增 `LANGUAGE_BY_EXT`、`TOKEN_RULES`、`_language_for_path`、`_highlight_line`；修改 `_render_line` 接入高亮）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：无新依赖（纯函数，独立于 Task 1-5 的解析/渲染管线）。
- Produces：`_highlight_line(language: str|None, text: str) -> str`（返回已转义、已包裹 `<span class="tok-*">` 的 HTML 片段）；`_language_for_path(path: str) -> str|None`。Task 5 的 `_render_line` 改为调用这两个函数而不是裸 `html.escape()`。

- [ ] **Step 1: 追加测试**

在 `scripts/tests/test_render_diff_review.py` 追加：

```python
_highlight_line = render_diff_review._highlight_line
_language_for_path = render_diff_review._language_for_path


class TestLanguageDetection(unittest.TestCase):
    def test_known_extensions_map_to_language(self):
        self.assertEqual(_language_for_path("src/foo.rs"), "rust")
        self.assertEqual(_language_for_path("src/foo.py"), "python")
        self.assertEqual(_language_for_path("scripts/foo.sh"), "shell")
        self.assertEqual(_language_for_path("README.md"), "markdown")
        self.assertEqual(_language_for_path("config.yaml"), "yaml")
        self.assertEqual(_language_for_path("Cargo.toml"), "toml")
        self.assertEqual(_language_for_path("data.json"), "json")

    def test_unknown_extension_returns_none(self):
        self.assertIsNone(_language_for_path("binary.exe"))


class TestHighlightLine(unittest.TestCase):
    def test_rust_keyword_string_comment(self):
        out = _highlight_line("rust", 'fn foo() { "hi" } // comment')
        self.assertIn('<span class="tok-keyword">fn</span>', out)
        self.assertIn('<span class="tok-string">&quot;hi&quot;</span>', out)
        self.assertIn('<span class="tok-comment">// comment</span>', out)

    def test_python_keyword_string_comment(self):
        out = _highlight_line("python", 'def foo(): return "hi"  # comment')
        self.assertIn('<span class="tok-keyword">def</span>', out)
        self.assertIn('<span class="tok-string">&quot;hi&quot;</span>', out)
        self.assertIn('<span class="tok-comment"># comment</span>', out)

    def test_shell_keyword_string_comment(self):
        out = _highlight_line("shell", 'if [ -f x ]; then echo "hi"; fi # comment')
        self.assertIn('<span class="tok-keyword">if</span>', out)
        self.assertIn('<span class="tok-string">&quot;hi&quot;</span>', out)
        self.assertIn('<span class="tok-comment"># comment</span>', out)

    def test_json_string(self):
        out = _highlight_line("json", '{"key": "value"}')
        self.assertIn('<span class="tok-string">&quot;key&quot;</span>', out)

    def test_toml_comment_and_string(self):
        out = _highlight_line("toml", 'name = "gf" # comment')
        self.assertIn('<span class="tok-string">&quot;gf&quot;</span>', out)
        self.assertIn('<span class="tok-comment"># comment</span>', out)

    def test_yaml_comment_and_string(self):
        out = _highlight_line("yaml", 'key: "value" # comment')
        self.assertIn('<span class="tok-string">&quot;value&quot;</span>', out)
        self.assertIn('<span class="tok-comment"># comment</span>', out)

    def test_markdown_heading(self):
        out = _highlight_line("markdown", '# Heading')
        self.assertIn('<span class="tok-keyword"># Heading</span>', out)

    def test_none_language_only_escapes(self):
        out = _highlight_line(None, '<script>alert(1)</script>')
        self.assertEqual(out, "&lt;script&gt;alert(1)&lt;/script&gt;")

    def test_html_in_string_is_escaped_not_double_escaped(self):
        out = _highlight_line("python", '"<b>"')
        self.assertIn("&lt;b&gt;", out)
        self.assertNotIn("<b>", out)
```

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `AttributeError`（`_highlight_line`/`_language_for_path` 尚不存在）。

- [ ] **Step 3: 实现语法高亮器**

在 `scripts/render-diff-review.py` 的 `STYLE` 常量定义之后（`_anchor_id` 之前）新增：

```python
LANGUAGE_BY_EXT = {
    ".rs": "rust",
    ".py": "python",
    ".sh": "shell", ".bash": "shell",
    ".md": "markdown",
    ".yml": "yaml", ".yaml": "yaml",
    ".toml": "toml",
    ".json": "json",
}


def _language_for_path(path):
    _, ext = os.path.splitext(path)
    return LANGUAGE_BY_EXT.get(ext)


TOKEN_RULES = {
    "rust": [
        (re.compile(r'//.*$'), "tok-comment"),
        (re.compile(r'"(?:[^"\\]|\\.)*"'), "tok-string"),
        (re.compile(r'\b(fn|let|mut|pub|struct|enum|impl|trait|use|mod|match|if|else|for|while|loop|return|self|Self)\b'), "tok-keyword"),
    ],
    "python": [
        (re.compile(r'#.*$'), "tok-comment"),
        (re.compile(r'"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)*\''), "tok-string"),
        (re.compile(r'\b(def|class|import|from|return|if|elif|else|for|while|with|as|try|except|raise|pass|self)\b'), "tok-keyword"),
    ],
    "shell": [
        (re.compile(r'#.*$'), "tok-comment"),
        (re.compile(r'"(?:[^"\\]|\\.)*"|\'[^\']*\''), "tok-string"),
        (re.compile(r'\b(if|then|else|fi|for|do|done|while|function|echo|local|return)\b'), "tok-keyword"),
    ],
    "markdown": [
        (re.compile(r'^#+\s.*$'), "tok-keyword"),
        (re.compile(r'`[^`]*`'), "tok-string"),
    ],
    "yaml": [
        (re.compile(r'#.*$'), "tok-comment"),
        (re.compile(r'"[^"]*"|\'[^\']*\''), "tok-string"),
    ],
    "toml": [
        (re.compile(r'#.*$'), "tok-comment"),
        (re.compile(r'"[^"]*"'), "tok-string"),
    ],
    "json": [
        (re.compile(r'"(?:[^"\\]|\\.)*"'), "tok-string"),
    ],
}


def _highlight_line(language, text):
    if language not in TOKEN_RULES:
        return html.escape(text)

    spans = []
    for pattern, css_class in TOKEN_RULES[language]:
        for m in pattern.finditer(text):
            spans.append((m.start(), m.end(), css_class))
    if not spans:
        return html.escape(text)

    spans.sort(key=lambda s: s[0])
    merged = []
    last_end = -1
    for start, end, css_class in spans:
        if start >= last_end:
            merged.append((start, end, css_class))
            last_end = end

    out = []
    cursor = 0
    for start, end, css_class in merged:
        out.append(html.escape(text[cursor:start]))
        out.append(f'<span class="{css_class}">{html.escape(text[start:end])}</span>')
        cursor = end
    out.append(html.escape(text[cursor:]))
    return "".join(out)
```

在 `STYLE` 常量的 CSS 文本里追加高亮用的 token 颜色规则（找到 `.flash { outline: 2px solid #ff9800; }` 这一行，在它之后追加）：

当前：

```
.flash { outline: 2px solid #ff9800; }
"""
```

改为：

```
.flash { outline: 2px solid #ff9800; }
.tok-keyword { color: #a626a4; font-weight: bold; }
.tok-string { color: #50a14f; }
.tok-comment { color: #a0a1a7; font-style: italic; }
"""
```

把 `_render_line` 改为使用高亮：

当前：

```python
def _render_line(file_path, line):
    old_no = line["old_line"] if line["old_line"] is not None else ""
    new_no = line["new_line"] if line["new_line"] is not None else ""
    css_class = {"add": "add", "remove": "remove", "context": ""}[line["type"]]
    anchor_id = html.escape(_anchor_id(file_path, line), quote=True)
    content_html = html.escape(line["content"])
    return (
```

改为：

```python
def _render_line(file_path, line):
    old_no = line["old_line"] if line["old_line"] is not None else ""
    new_no = line["new_line"] if line["new_line"] is not None else ""
    css_class = {"add": "add", "remove": "remove", "context": ""}[line["type"]]
    anchor_id = html.escape(_anchor_id(file_path, line), quote=True)
    content_html = _highlight_line(_language_for_path(file_path), line["content"])
    return (
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（32 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): add lightweight regex-based syntax highlighter"
```

---

## Task 7: JS 交互（锚定跳转、hover 联动、折叠、拖拽记忆宽度）

**Files:**
- Modify: `scripts/render-diff-review.py`（新增 `JS_SCRIPT` 常量，`render_html` 追加 `<script>` 块）
- Modify: `scripts/tests/test_render_diff_review.py`（追加测试）

**Interfaces:**
- Consumes：Task 5 的 `render_html`（本任务只在其输出末尾追加一个 `<script>` 块，不改变已有的 pane/anchor HTML 结构）。
- Produces：无新的 Python 函数供其他任务调用——`JS_SCRIPT` 是内部实现细节，只被 `render_html` 使用。

- [ ] **Step 1: 追加测试（结构性断言，不执行 JS 运行时）**

在 `scripts/tests/test_render_diff_review.py` 追加：

```python
class TestJsInteractivity(unittest.TestCase):
    def test_script_block_present_with_expected_handlers(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        self.assertIn("<script>", out)
        self.assertIn("addEventListener('click'", out)
        self.assertIn("addEventListener('mouseenter'", out)
        self.assertIn("addEventListener('mouseleave'", out)
        self.assertIn("localStorage", out)
        self.assertIn("scrollIntoView", out)

    def test_no_accept_reject_writeback_controls(self):
        out = render_html(SAMPLE_ANNOTATIONS)
        for forbidden in ("accept", "reject", "approve", "apply-fix", "write-back"):
            self.assertNotIn(forbidden, out.lower())
```

- [ ] **Step 2: 运行测试，确认新用例失败**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: 第一个测试 FAIL（没有 `<script>` 块）；第二个测试通过（当前确实没有这些词，属于巧合通过，Task 完成后仍应保持不出现）。

- [ ] **Step 3: 实现 JS 交互并接入 `render_html`**

在 `TOKEN_RULES`/`_highlight_line` 定义之后（`_anchor_id` 之前或之后均可，放在 `render_html` 定义之前）新增：

```python
JS_SCRIPT = """
document.addEventListener('DOMContentLoaded', function () {
  document.querySelectorAll('.file-item').forEach(function (item) {
    item.addEventListener('click', function () {
      var sel = document.querySelector(
        '.file-section[data-file="' + CSS.escape(item.dataset.file) + '"]'
      );
      if (sel) { sel.scrollIntoView({behavior: 'smooth'}); }
    });
  });

  function highlightAnchor(anchorId) {
    document.querySelectorAll('.diff-line, .annotation-card').forEach(function (el) {
      el.classList.toggle('hover-highlight', el.dataset.anchor === anchorId);
    });
  }

  document.querySelectorAll('.diff-line').forEach(function (line) {
    line.addEventListener('mouseenter', function () { highlightAnchor(line.dataset.anchor); });
    line.addEventListener('mouseleave', function () { highlightAnchor(null); });
  });

  document.querySelectorAll('.annotation-card').forEach(function (card) {
    card.addEventListener('mouseenter', function () { highlightAnchor(card.dataset.anchor); });
    card.addEventListener('mouseleave', function () { highlightAnchor(null); });
    card.addEventListener('click', function () {
      var target = document.getElementById(card.dataset.anchor);
      if (target) {
        target.scrollIntoView({behavior: 'smooth', block: 'center'});
        target.classList.add('flash');
        setTimeout(function () { target.classList.remove('flash'); }, 800);
      }
    });
  });

  function setupResize(handleId, paneId, storageKey) {
    var handle = document.getElementById(handleId);
    var pane = document.getElementById(paneId);
    if (!handle || !pane) { return; }
    var saved = null;
    try { saved = localStorage.getItem(storageKey); } catch (e) { saved = null; }
    if (saved) { pane.style.width = saved + 'px'; }
    var dragging = false;
    handle.addEventListener('mousedown', function () { dragging = true; });
    document.addEventListener('mouseup', function () {
      if (dragging) {
        try { localStorage.setItem(storageKey, pane.getBoundingClientRect().width); } catch (e) {}
      }
      dragging = false;
    });
    document.addEventListener('mousemove', function (ev) {
      if (!dragging) { return; }
      var rect = pane.getBoundingClientRect();
      var newWidth = paneId === 'file-tree' ? (ev.clientX - rect.left) : (rect.right - ev.clientX);
      pane.style.width = Math.max(120, newWidth) + 'px';
    });
  }
  setupResize('resize-tree', 'file-tree', 'diff-review-tree-width');
  setupResize('resize-annotation', 'annotation-pane', 'diff-review-annotation-width');
});
"""
```

把 `render_html` 的返回值改为在 `</div>\n` 之后、`"</body>\n</html>\n"` 之前插入 `<script>` 块：

当前：

```python
        f'{annotation_pane}\n'
        '</div>\n'
        "</body>\n</html>\n"
    )
```

改为：

```python
        f'{annotation_pane}\n'
        '</div>\n'
        f'<script>{JS_SCRIPT}</script>\n'
        "</body>\n</html>\n"
    )
```

- [ ] **Step 4: 运行测试，确认全部通过**

Run: `python3 scripts/tests/test_render_diff_review.py`
Expected: `OK`（34 项测试全过）。

- [ ] **Step 5: Commit**

```bash
git add scripts/render-diff-review.py scripts/tests/test_render_diff_review.py
git commit -m "feat(scripts): add JS interactivity (anchor scroll, hover link, drag-resize)"
```

---

## Task 8: Makefile 目标 + 全局约束核实 + AC8 真实 PR 验收

**Files:**
- Modify: `Makefile`（新增 `render-diff-review` 目标，`.PHONY` 列表追加）

**Interfaces:**
- Consumes：Task 4/5 的 `main()` CLI（`scan`/`render` 子命令）。

- [ ] **Step 1: 新增 Makefile 目标**

当前（`Makefile` 里 `render-workflow-dashboard` 目标，Issue #345 引入）：

```makefile
render-workflow-dashboard: ## 从 .cache/workflows/active/*.json 生成派生的 HTML 进度看板（勿手编产物）
	@python3 scripts/render-workflow-dashboard.py
	@echo "✓ 已生成 .cache/workflows/dashboard.html"
```

改为（在其后追加空行 + 新目标）：

```makefile
render-workflow-dashboard: ## 从 .cache/workflows/active/*.json 生成派生的 HTML 进度看板（勿手编产物）
	@python3 scripts/render-workflow-dashboard.py
	@echo "✓ 已生成 .cache/workflows/dashboard.html"

render-diff-review: ## 对 dev..HEAD 的改动生成交互式 diff 审阅页（勿手编产物），可用 RANGE=<base>..<head> 覆盖
	@python3 scripts/render-diff-review.py scan "$${RANGE:-dev..HEAD}"
	@python3 scripts/render-diff-review.py render ".cache/diff-review/$$(echo "$${RANGE:-dev..HEAD}" | tr '/' '-').json"
```

- [ ] **Step 2: `.PHONY` 列表追加目标名**

当前：

```makefile
        update-submodule check-agent-sync check-smell-skill check-refactor-skill check-architecture-diagram-skill check-walkthrough-skill check-quality-review-evidence-skill check-decompose-skill check-skills-drift render-workflow-dashboard release release-quick release-rehearse \
```

改为：

```makefile
        update-submodule check-agent-sync check-smell-skill check-refactor-skill check-architecture-diagram-skill check-walkthrough-skill check-quality-review-evidence-skill check-decompose-skill check-skills-drift render-workflow-dashboard render-diff-review release release-quick release-rehearse \
```

- [ ] **Step 3: 验证——全部单元测试**

```bash
python3 scripts/tests/test_render_diff_review.py
```

预期：`OK`（34 项测试全过，与 Task 7 结束时一致）。

- [ ] **Step 4: 验证——AC8 真实 PR 验收**

用 Issue #345 自己的合并 range 作为验收样本（`c5ec8ac..d352cc3`）：

```bash
python3 scripts/render-diff-review.py scan "c5ec8ac..d352cc3"
python3 scripts/render-diff-review.py render ".cache/diff-review/c5ec8ac..d352cc3.json"
ls -la .cache/diff-review/
```

预期：两条命令均退出码 0，`.cache/diff-review/` 下出现 `c5ec8ac..d352cc3.json` 和 `c5ec8ac..d352cc3.html`。

用一个独立断言脚本核对渲染的改动行数与 `git diff --stat` 吻合（AC8 的字面要求）：

```bash
python3 -c "
import json
with open('.cache/diff-review/c5ec8ac..d352cc3.json', encoding='utf-8') as f:
    data = json.load(f)
rendered_add = sum(1 for f in data['files'] for l in f['lines'] if l['type'] == 'add')
rendered_remove = sum(1 for f in data['files'] for l in f['lines'] if l['type'] == 'remove')
print(f'rendered: +{rendered_add} -{rendered_remove}')
"
git diff --stat c5ec8ac..d352cc3 | tail -1
```

预期：两边报告的插入/删除总数一致（`git diff --stat` 的汇总行给出 `N insertions(+), M deletions(-)`，与上面 Python 脚本算出的 `rendered_add`/`rendered_remove` 应分别相等）。

- [ ] **Step 5: 验证——gitignore 覆盖 + check-agent-sync 基线**

```bash
git check-ignore .cache/diff-review/c5ec8ac..d352cc3.html && echo "ignored: OK"
make check-agent-sync
```

预期：`git check-ignore` 命中并原样打印路径；`make check-agent-sync` 与本计划开始前的基线完全一致（Commands=78 Files=29 Refs=194 Mismatches=0）——本计划未触碰任何 skill 文件。

人工用浏览器打开 `.cache/diff-review/c5ec8ac..d352cc3.html`（`file://` 本地协议）确认：三栏布局正常、点击文件树能跳转、鼠标悬停 diff 行能联动右侧、拖拽调整宽度后刷新页面宽度被记住、语法高亮对 `.py`/`.md` 等文件生效、没有浏览器控制台报错、没有任何对外部资源的网络请求。

- [ ] **Step 6: Commit**

```bash
git add Makefile
git commit -m "feat(makefile): add render-diff-review target"
```

---

## Final Validation

```bash
python3 scripts/tests/test_render_diff_review.py
make check-agent-sync
make test
```

预期：Python 测试 34 项全过；`check-agent-sync` 基线不变；Rust 测试套件不受影响（本计划零 Rust 代码改动）。人工再通读一遍 `scripts/render-diff-review.py` 全文，确认没有遗留的 `TODO`/占位符，确认每一处从 diff/文件系统读入的字符串在写入 HTML 前都经过 `html.escape()`（尤其 `_render_file_tree`、`_render_file_section` 的 header_extra 拼接部分）。
