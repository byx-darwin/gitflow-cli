# gf-architecture-diagram Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 `skills/gf-architecture-diagram/`，把 `docs/architecture-diagram.dot` 从手工维护改为从依赖清单结构化提取 + 几何回验后生成，语言无关的生成/回验协议在 SKILL.md，语言专属的依赖提取下沉到 `references/<lang>.md`。

**Architecture:** SKILL.md 承载五阶段协议：Stage 0 语言探测（复用 `gf-quality/references/detector.md`）→ Stage 1 按 `references/<lang>.md` 的结构化命令提取模块与依赖边 → Stage 2 生成前读取 `examples/<lang>.svg` 金标样例收敛输出格式 → Stage 3 生成 `.dot` 并用 `dot -Tsvg` 渲染 → Stage 4 用 `scripts/review_svg.py` 做几何回验（标签越界 / 元素重叠）→ Stage 5 确定性检查（同输入重新生成两次，拓扑一致）。`references/rust.md` 用 `cargo metadata --no-deps --format-version=1` 结构化解析工作区成员与内部依赖边，不做语义推断。

**Tech Stack:** Markdown（skill 定义）· Graphviz `dot`（渲染）· Python 3 标准库（`scripts/review_svg.py` 几何回验，无第三方依赖）· GNU Make（验收断言）

**Spec:** Issue #331（本仓库无独立 spec 文档，Issue 正文即需求来源；`spec_path` 记为本计划自身）

## Global Constraints

- SKILL.md 正文**不得**出现任何单一语言的清单文件名或包管理器名（`Cargo.toml` / `go.mod` / `go.work` / `package.json` / `pyproject.toml` / `setup.py` / `pom.xml` / `build.gradle` / `Gemfile` / `cargo` / `npm` / `yarn` / `pnpm` / `pip` / `poetry` / `maven` / `gradle` / `bundler`，单词边界匹配）。
- `references/` 下**至少**含 `rust.md`，四段契约结构须可扩展到 go / node / python / java（本轮只落 rust.md，其余语言留空目录结构由后续 Issue 补齐——不在本轮范围内）。
- 语言探测复用 `gf-quality/references/detector.md`，**不得**自行实现探测逻辑。
- 提取出的模块集合须与该语言依赖清单完全一致，缺失任一现存模块视为假。
- 生成的依赖边**不得**包含清单中不存在的边。
- 生成前**必须**读取 `examples/<lang>.svg` 金标样例；样例缺失时停止，不臆造格式。
- 生成后**必须**跑 `scripts/review_svg.py`；存在标签越界或元素重叠则本次生成视为假，不落盘。
- 输出格式固定为 SVG（与既有 `docs/assets/demo.svg` 一致），**不产出** PNG。
- 同一输入重新生成两次，节点集合与边集合（拓扑）须完全一致；只允许坐标级的非语义抖动。
- 禁止修改 `clippy.toml` / `deny.toml` / `.pre-commit-config.yaml` / `rust-toolchain.toml`。
- 禁止运行 `cargo clean`。
- Skill 源码只改 `skills/gf-architecture-diagram/`，**不改** `.claude/skills/`（那是副本）。
- skill-only 变更 + 一个 Python 脚本 + 一次 dogfooding dot 渲染；不跑 Rust 构建/测试全套（脚本本身用 `python3` 语法检查即可）；验证用 `make check-architecture-diagram-skill` + `make check-agent-sync`。

---

### Task 1: 验收断言（RED）

**Files:**
- Modify: `Makefile`（在 `check-smell-skill` target 之后新增 `check-architecture-diagram-skill`，并把新 target 加入 `.PHONY` 列表）

**Interfaces:**
- Consumes: 无
- Produces: `make check-architecture-diagram-skill` —— 后续每个 Task 都用它验证；退出码 0 表示全部断言通过

- [ ] **Step 1: 在 `.PHONY` 列表中加入新 target**

在 Makefile 第 502 行附近的 `.PHONY` 列表里，`check-smell-skill` 后面加入 `check-architecture-diagram-skill`：

```make
	update-submodule check-agent-sync check-smell-skill check-architecture-diagram-skill check-walkthrough-skill check-decompose-skill check-skills-drift release release-quick release-rehearse \
```

- [ ] **Step 2: 写入 Makefile target（此时必然失败，因为 skill 尚不存在）**

在 `check-smell-skill` target 结束之后（第 ~275 行附近，紧接其 recipe 末尾）插入：

```make
check-architecture-diagram-skill: ## Verify gf-architecture-diagram skill meets Issue #331 acceptance criteria
	@S=skills/gf-architecture-diagram/SKILL.md; R=skills/gf-architecture-diagram/references; \
	E=skills/gf-architecture-diagram/examples; SC=skills/gf-architecture-diagram/scripts/review_svg.py; FAIL=0; \
	if [ ! -f "$$S" ]; then echo "✗ missing $$S"; exit 1; fi; \
	if grep -nEi '\bCargo\.toml\b|\bgo\.mod\b|\bgo\.work\b|\bpackage\.json\b|\bpyproject\.toml\b|\bsetup\.py\b|\bpom\.xml\b|\bbuild\.gradle\b|\bGemfile\b|\bcargo\b|\bnpm\b|\byarn\b|\bpnpm\b|\bpip\b|\bpoetry\b|\bmaven\b|\bgradle\b|\bbundler\b' "$$S"; then \
		echo "✗ AC#1 SKILL.md 正文含单一语言的清单文件名或包管理器名"; FAIL=1; \
	else echo "✓ AC#1 正文无语言专属标识"; fi; \
	if [ ! -f "$$R/rust.md" ]; then echo "✗ AC#2 缺少 $$R/rust.md"; FAIL=1; \
	else echo "✓ AC#2 references/rust.md 存在"; fi; \
	if [ -f "$$R/rust.md" ]; then \
		for H in '## 提取命令' '## 模块判定' '## 边过滤规则' '## 工具缺失降级'; do \
			grep -qF "$$H" "$$R/rust.md" || { echo "✗ 契约 rust.md 缺少 $$H"; FAIL=1; }; \
		done; \
	fi; \
	grep -qF 'gf-quality/references/detector.md' "$$S" \
		&& echo "✓ AC#3 复用既有语言探测" \
		|| { echo "✗ AC#3 未引用 detector.md"; FAIL=1; }; \
	if [ -f "$$R/rust.md" ]; then \
		grep -qF 'cargo metadata --no-deps --format-version=1' "$$R/rust.md" \
			&& echo "✓ AC#4/5 提取命令为结构化解析（非语义推断）" \
			|| { echo "✗ AC#4/5 未使用 cargo metadata 结构化提取"; FAIL=1; }; \
	fi; \
	grep -qF 'examples/' "$$S" && grep -qF '金标样例' "$$S" \
		&& echo "✓ AC#6 声明生成前读取金标样例" \
		|| { echo "✗ AC#6 未声明读取 examples/ 金标样例"; FAIL=1; }; \
	if [ ! -f "$$E/rust.svg" ]; then echo "✗ AC#6 缺少 $$E/rust.svg 金标样例"; FAIL=1; \
	else echo "✓ AC#6 金标样例已落盘"; fi; \
	if [ ! -f "$$SC" ]; then echo "✗ AC#7 缺少 $$SC"; FAIL=1; \
	else \
		python3 -c "import ast; ast.parse(open('$$SC').read())" \
			&& echo "✓ AC#7 review_svg.py 语法有效" \
			|| { echo "✗ AC#7 review_svg.py 语法错误"; FAIL=1; }; \
	fi; \
	grep -qF '几何回验' "$$S" \
		&& echo "✓ AC#7 声明生成后几何回验" \
		|| { echo "✗ AC#7 未声明生成后几何回验"; FAIL=1; }; \
	if grep -nEi '\.png\b' "$$S" | grep -viF 'svg'; then \
		echo "✗ AC#8 SKILL.md 提及 PNG 输出"; FAIL=1; \
	else echo "✓ AC#8 未提及 PNG 输出"; fi; \
	grep -qF 'SVG' "$$S" \
		&& echo "✓ AC#8 声明输出为 SVG" \
		|| { echo "✗ AC#8 未声明 SVG 输出格式"; FAIL=1; }; \
	grep -qF '确定性' "$$S" && grep -qF '重新生成两次' "$$S" \
		&& echo "✓ AC#9 声明确定性检查（重新生成两次）" \
		|| { echo "✗ AC#9 未声明确定性检查"; FAIL=1; }; \
	if [ -f "$$SC" ]; then \
		GOOD=$$(mktemp -t gf-arch-good.svg); \
		BAD=$$(mktemp -t gf-arch-bad.svg); \
		printf '%s\n' \
			'<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">' \
			'<g class="node"><title>a</title><polygon points="10,10 90,10 90,50 10,50"/><text x="50" y="30" font-size="10">a</text></g>' \
			'<g class="node"><title>b</title><polygon points="110,10 190,10 190,50 110,50"/><text x="150" y="30" font-size="10">b</text></g>' \
			'</svg>' > "$$GOOD"; \
		printf '%s\n' \
			'<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">' \
			'<g class="node"><title>a</title><polygon points="10,10 90,10 90,50 10,50"/><text x="50" y="30" font-size="10">a</text></g>' \
			'<g class="node"><title>b</title><polygon points="70,10 150,10 150,50 70,50"/><text x="110" y="30" font-size="10">this label is far too long to fit</text></g>' \
			'</svg>' > "$$BAD"; \
		python3 "$$SC" "$$GOOD" >/dev/null 2>&1; GOOD_RC=$$?; \
		python3 "$$SC" "$$BAD" >/dev/null 2>&1; BAD_RC=$$?; \
		rm -f "$$GOOD" "$$BAD"; \
		if [ "$$GOOD_RC" -eq 0 ] && [ "$$BAD_RC" -ne 0 ]; then \
			echo "✓ AC#7 review_svg.py 正确区分干净样例与越界/重叠样例"; \
		else \
			echo "✗ AC#7 review_svg.py 未正确判定（good_rc=$$GOOD_RC bad_rc=$$BAD_RC）"; FAIL=1; \
		fi; \
	fi; \
	if [ $$FAIL -ne 0 ]; then echo "FAILED"; exit 1; fi; \
	echo "ALL CHECKS PASSED"
```

- [ ] **Step 3: 运行断言，确认它失败（RED）**

Run: `make check-architecture-diagram-skill`
Expected: FAIL，第一行输出 `✗ missing skills/gf-architecture-diagram/SKILL.md`，非零退出。

- [ ] **Step 4: 确认 help 列表能看到新 target**

Run: `make help | grep check-architecture-diagram-skill`
Expected: 输出一行 `check-architecture-diagram-skill  Verify gf-architecture-diagram skill meets Issue #331 acceptance criteria`

- [ ] **Step 5: Commit**

```bash
git add Makefile
git commit -m "test(architecture-diagram): 新增 check-architecture-diagram-skill 验收断言（RED）

把 Issue #331 中可机械验证的验收标准写成 Makefile target：
正文无语言标识、references/rust.md 四段契约、复用 detector.md、
结构化提取命令（cargo metadata --no-deps）、金标样例存在且被引用、
review_svg.py 语法有效并能正确区分干净/越界样例、声明 SVG 输出、
声明确定性检查。

当前 skills/gf-architecture-diagram/ 尚不存在，断言必然失败。

Refs #331"
```

---

### Task 2: `scripts/review_svg.py`（几何回验脚本）

**Files:**
- Create: `skills/gf-architecture-diagram/scripts/review_svg.py`

**Interfaces:**
- Consumes: 一个 Graphviz `dot -Tsvg` 渲染出的 `.svg` 文件路径（命令行参数）
- Produces: Task 3 SKILL.md Stage 4 引用的可执行脚本；退出码 0 = 干净，1 = 存在标签越界或元素重叠，2 = 用法错误

- [ ] **Step 1: 写入 `skills/gf-architecture-diagram/scripts/review_svg.py`**

```bash
mkdir -p skills/gf-architecture-diagram/scripts skills/gf-architecture-diagram/references skills/gf-architecture-diagram/examples
```

写入 `skills/gf-architecture-diagram/scripts/review_svg.py`：

```python
#!/usr/bin/env python3
"""Geometric review for Graphviz-rendered SVG diagrams.

Checks two failure classes in an SVG produced by `dot -Tsvg`:
  1. Label overflow: a <text> element's estimated rendered width exceeds
     its enclosing node's shape width.
  2. Element overlap: two distinct node shapes' bounding boxes intersect.

Usage: review_svg.py <path-to.svg>
Exit code: 0 = clean, 1 = findings present, 2 = usage error.
"""
import sys
import xml.etree.ElementTree as ET

SVG_NS = "{http://www.w3.org/2000/svg}"
OVERFLOW_TOLERANCE = 1.05
AVG_CHAR_WIDTH_RATIO = 0.6
DEFAULT_FONT_SIZE = 14.0


def _bbox_from_points(points_attr):
    pts = []
    for pair in points_attr.strip().split(" "):
        pair = pair.strip()
        if not pair:
            continue
        x_str, y_str = pair.split(",")
        pts.append((float(x_str), float(y_str)))
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    return min(xs), min(ys), max(xs), max(ys)


def _bbox_from_ellipse(el):
    cx = float(el.get("cx"))
    cy = float(el.get("cy"))
    rx = float(el.get("rx"))
    ry = float(el.get("ry"))
    return cx - rx, cy - ry, cx + rx, cy + ry


def node_bbox(node_g):
    poly = node_g.find(f"{SVG_NS}polygon")
    if poly is not None:
        return _bbox_from_points(poly.get("points"))
    ellipse = node_g.find(f"{SVG_NS}ellipse")
    if ellipse is not None:
        return _bbox_from_ellipse(ellipse)
    return None


def text_width(text_el):
    content = text_el.text or ""
    font_size = float(text_el.get("font-size", DEFAULT_FONT_SIZE))
    return len(content) * font_size * AVG_CHAR_WIDTH_RATIO


def boxes_overlap(a, b):
    ax0, ay0, ax1, ay1 = a
    bx0, by0, bx1, by1 = b
    return ax0 < bx1 and bx0 < ax1 and ay0 < by1 and by0 < ay1


def collect_nodes(root):
    nodes = []
    for g in root.iter(f"{SVG_NS}g"):
        if g.get("class") != "node":
            continue
        title_el = g.find(f"{SVG_NS}title")
        name = title_el.text if title_el is not None else "<unnamed>"
        bbox = node_bbox(g)
        if bbox is None:
            continue
        nodes.append((name, bbox, g))
    return nodes


def review(svg_path):
    tree = ET.parse(svg_path)
    root = tree.getroot()
    findings = []

    nodes = collect_nodes(root)

    for name, bbox, g in nodes:
        bbox_w = bbox[2] - bbox[0]
        for text_el in g.findall(f"{SVG_NS}text"):
            w = text_width(text_el)
            if w > bbox_w * OVERFLOW_TOLERANCE:
                findings.append(
                    f"overflow: node '{name}' label '{text_el.text}' "
                    f"estimated width {w:.1f} exceeds shape width {bbox_w:.1f}"
                )

    for i in range(len(nodes)):
        for j in range(i + 1, len(nodes)):
            name_a, bbox_a, _ = nodes[i]
            name_b, bbox_b, _ = nodes[j]
            if boxes_overlap(bbox_a, bbox_b):
                findings.append(
                    f"overlap: node '{name_a}' and node '{name_b}' bounding boxes intersect"
                )

    return findings


def main(argv):
    if len(argv) != 2:
        print("usage: review_svg.py <path-to.svg>", file=sys.stderr)
        return 2
    findings = review(argv[1])
    if not findings:
        print("CLEAN: no label overflow or element overlap detected")
        return 0
    for finding in findings:
        print(f"✗ {finding}")
    print(f"FAILED: {len(findings)} finding(s)")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
```

- [ ] **Step 2: 语法与可执行性检查**

Run: `python3 -m py_compile skills/gf-architecture-diagram/scripts/review_svg.py && echo OK`
Expected: `OK`，无异常。

- [ ] **Step 3: 用两个手工构造的最小 SVG 验证判定逻辑**

Run:
```bash
cat > /tmp/gf-arch-good.svg <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
<g class="node"><title>a</title><polygon points="10,10 90,10 90,50 10,50"/><text x="50" y="30" font-size="10">a</text></g>
<g class="node"><title>b</title><polygon points="110,10 190,10 190,50 110,50"/><text x="150" y="30" font-size="10">b</text></g>
</svg>
EOF
cat > /tmp/gf-arch-bad.svg <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
<g class="node"><title>a</title><polygon points="10,10 90,10 90,50 10,50"/><text x="50" y="30" font-size="10">a</text></g>
<g class="node"><title>b</title><polygon points="70,10 150,10 150,50 70,50"/><text x="110" y="30" font-size="10">this label is far too long to fit</text></g>
</svg>
EOF
python3 skills/gf-architecture-diagram/scripts/review_svg.py /tmp/gf-arch-good.svg; echo "good_rc=$?"
python3 skills/gf-architecture-diagram/scripts/review_svg.py /tmp/gf-arch-bad.svg; echo "bad_rc=$?"
rm -f /tmp/gf-arch-good.svg /tmp/gf-arch-bad.svg
```
Expected: `good_rc=0`（`CLEAN: ...`）；`bad_rc=1`，输出至少一条 `overflow:` 和一条 `overlap:`（`b` 的多字符标签超出其收窄后的形状宽度，且 `a`/`b` 两个多边形在 x∈[70,90] 处重叠）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-architecture-diagram/scripts/review_svg.py
git commit -m "feat(architecture-diagram): 几何回验脚本 review_svg.py

纯标准库实现：解析 dot -Tsvg 输出的 <g class=\"node\"> 多边形/椭圆边界框，
逐条 <text> 估算渲染宽度判定标签越界，逐对节点边界框判定重叠。
退出码 0=干净 / 1=存在越界或重叠 / 2=用法错误。

Refs #331"
```

---

### Task 3: SKILL.md（语言无关层）

**Files:**
- Create: `skills/gf-architecture-diagram/SKILL.md`

**Interfaces:**
- Consumes: Task 2 的 `scripts/review_svg.py`（Stage 4 调用）
- Produces: 语言层契约（四个二级标题 `## 提取命令` / `## 模块判定` / `## 边过滤规则` / `## 工具缺失降级`），Task 4 的 `references/rust.md` 必须照此结构填写；金标样例路径约定 `examples/<lang>.svg`

- [ ] **Step 1: 写入 `skills/gf-architecture-diagram/SKILL.md`**

````markdown
---
name: gf-architecture-diagram
description: |
  Use when the user wants to (re)generate the project's architecture diagram
  from real dependency data, verify it hasn't drifted from the manifest, or
  add a language layer for architecture-diagram generation.
  当用户希望从真实依赖数据（重新）生成项目架构图、验证架构图与依赖清单是否
  漂移，或为架构图生成新增语言层时使用。
allowed-tools: Read, Bash, Write, Glob
---

# gf-architecture-diagram — Dependency-Derived Architecture Diagrams

Five-stage protocol: detect → extract → converge on format → render → verify.
Language-agnostic protocol in this file; dependency extraction in
`references/<lang>.md`. **Extraction is structured parsing, never semantic
inference** — every node and edge must trace back to a manifest entry.

## Preconditions

- Read access to the target source tree
- `dot` (Graphviz) installed for rendering
- `python3` (standard library only) for `scripts/review_svg.py`
- The language layer's extraction tool installed (see `references/<lang>.md`)

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| regenerate architecture diagram | 重新生成架构图 | manifest changed, diagram may have drifted |
| verify diagram matches dependencies | 验证架构图与依赖一致 | audit before a release |
| add a language layer | 新增语言层 | extend to go/node/python/java |
| architecture overview | 架构总览图 | onboarding, design review |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Hand-drawing a conceptual diagram not tied to real dependencies | This skill only renders what the manifest structurally contains | A design doc's own diagram, drawn by hand |
| Detecting code smells or complexity | Different problem class | `/gf-smell` |
| Auditing licenses or vulnerabilities | Different problem class | `/gf-security-check` |
| Producing a raster image for a slide deck | This skill's contract is SVG only | Export SVG externally with your own tool |

## Stage 0: Detect Language

Detect the project language(s) by following `gf-quality/references/detector.md`.
**Do not implement a second detection mechanism.** Consume its output
(language · path · workspace type) and load the matching `references/<lang>.md`.
For multi-language projects, use the selection interaction that reference
already defines; generate one diagram per selected language.

## Stage 1: Extract (structured parsing only)

Run the language layer's `## 提取命令` **once**, capture the output, and reuse
that capture for the rest of the run — a repeated invocation can hit a build
cache and silently return empty output.

Apply `## 模块判定` to decide which manifest entries become diagram nodes
(internal/workspace modules only — external dependencies never become
nodes), and `## 边过滤规则` to decide which relationships become edges
(only edges where **both** endpoints are internal modules).

The resulting node set MUST equal the internal module set in the manifest
exactly — no omissions, no additions. The resulting edge set MUST be a
subset of the manifest's declared relationships — never a fabricated or
inferred edge.

## Stage 2: Read the Gold Sample Before Generating

**Before writing any `.dot` content**, read `examples/<lang>.svg` for the
detected language. This gold sample fixes node shape, cluster styling, font,
and color conventions — read it instead of re-deriving format rules from
prose. If `examples/<lang>.svg` does not exist for the detected language,
**stop** and tell the user a gold sample must be authored first; do not
invent a format.

## Stage 3: Generate and Render

1. Write `.dot` source using the node/edge set from Stage 1, matching the
   gold sample's visual conventions (clusters, colors, label style).
2. Render: `dot -Tsvg <input.dot> -o <output.svg>`.
3. Output format is **fixed to SVG** — never render to PNG or any raster
   format. SVG stays legible at any zoom and diffs as text.

## Stage 4: Geometric Review

Run `scripts/review_svg.py <output.svg>`. A non-zero exit means the render
has **label overflow or element overlap** — the generation is invalid.
Fix the `.dot` source (adjust node size, cluster layout, or label length)
and re-render; do not hand-edit the SVG directly, since the next
regeneration would silently discard the edit.

## Stage 5: Determinism Check

Regenerate the same diagram a second time from the same extraction capture
(Stage 1 output). Compare the node set and edge set (topology) between the
two 生成结果 — 同一输入重新生成两次，拓扑必须完全一致. Coordinate-level
jitter from the layout engine is acceptable; a different node set, edge set,
or cluster membership is not and means Stage 1/3 has a non-determinism bug.

## Report

Overwrite the target `.svg` (and its `.dot` source) in place — this skill's
whole purpose is to keep the diagram in sync with the manifest, not to
produce a dated report file. Announce which nodes/edges changed relative to
the previous version, if one existed.

## Responsibility

### ✅ In Scope

- Extract internal module/edge structure from a manifest via structured parsing
- Render an SVG diagram matching an existing gold sample's format
- Geometrically verify the render and check regeneration determinism

### ❌ Out of Scope

- Inferring architectural relationships not present in the manifest
- Code smell / complexity detection — `/gf-smell`
- License / vulnerability auditing — `/gf-security-check`

### 🚫 Do Not

- ❌ Add a node or edge not backed by a manifest entry
- ❌ Generate before reading the gold sample
- ❌ Skip the geometric review or ignore its findings
- ❌ Render to PNG or any raster format
- ❌ Hand-edit the rendered SVG instead of fixing the `.dot` source

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "I know roughly how these modules relate" | Extraction is structured parsing, not memory. Run the command. |
| "The label barely overflows, close enough" | `review_svg.py`'s verdict is binary. Fix the source, re-render. |
| "Skip the gold sample, I've seen enough diagrams" | Stage 2 is mandatory precisely because format drifts silently otherwise. |
| "Second run looks basically the same" | "Basically" is not identical. Diff the topology, not the picture. |

## Red Flags

- 🚩 "Just eyeball whether it overlaps" — Refuse. Run `scripts/review_svg.py`.
- 🚩 "Add this edge, it's obviously implied" — Refuse. No manifest entry, no edge.
- 🚩 "Render a PNG too, for the slide deck" — Refuse. SVG only.

## Common Mistakes

- ❌ **Re-running the extraction command and reading a cached, empty result.**
- ❌ **Treating Stage 4's tolerance as advisory** — a non-zero exit is a hard stop.
- ❌ **Comparing rendered pixels instead of node/edge sets for determinism.**

## Test Scenarios

### 1: Happy Path
- **Given** a Rust workspace — **When** "regenerate the architecture diagram"
- **Then** Stage 0-5 run → SVG matches gold-sample format → geometric review clean → regenerated twice with identical topology

### 2: Negative
- **Given** "add an edge showing the CLI calls the database directly" (not in manifest)
- **Then** refuse — Stage 1 only emits manifest-backed edges

### 3: Boundary
- **Given** `examples/<lang>.svg` missing for the detected language
- **Then** stop, do not invent a format

### 4: Error
- **Given** `scripts/review_svg.py` reports label overflow
- **Then** adjust `.dot` source and re-render; never edit the SVG directly

## Success Criteria

- [ ] Node set exactly matches manifest's internal modules
- [ ] Edge set is a subset of manifest-declared relationships
- [ ] Gold sample read before generation
- [ ] Geometric review passes (no overflow, no overlap)
- [ ] Regenerating twice yields identical topology
- [ ] Output is SVG, never PNG

## Trigger Keywords

| English | 中文 |
|---------|------|
| architecture diagram | 架构图 |
| dependency graph | 依赖图 |
| regenerate diagram | 重新生成架构图 |
| diagram drift | 架构图漂移 |

## See Also

- `/gf-smell` — code smell and complexity hotspot detection
- `/gf-security-check` — secrets, vulnerabilities, license compliance
- `gf-quality/references/detector.md` — shared language detection
````

- [ ] **Step 2: 运行断言，确认语言无关层大部分转绿、references/examples 仍红**

Run: `make check-architecture-diagram-skill`
Expected: `✓ AC#1` / `✓ AC#3` / `✓ AC#6`（引用金标样例的声明部分）/ `✓ AC#8` / `✓ AC#9` 通过；`✗ AC#2 缺少 skills/gf-architecture-diagram/references/rust.md`、`✗ AC#6 缺少 .../examples/rust.svg`、`✗ AC#4/5`（rust.md 不存在）仍失败，退出码 1。

- [ ] **Step 3: 单独复验 AC#1（最易回归的一条）**

Run: `grep -nEi '\bCargo\.toml\b|\bcargo\b|\bnpm\b|\bpip\b' skills/gf-architecture-diagram/SKILL.md`
Expected: 无输出（退出码 1）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-architecture-diagram/SKILL.md
git commit -m "feat(architecture-diagram): SKILL.md 语言无关生成/回验协议

五阶段协议：探测（复用 detector.md）→ 结构化提取 → 生成前读金标样例
→ 生成并渲染 SVG → 几何回验（review_svg.py）→ 确定性检查（重新生成
两次比对拓扑）。allowed-tools 含 Write（本 skill 产出文件，与只读的
gf-smell 不同）。语言层契约四个标题：提取命令/模块判定/边过滤规则/
工具缺失降级。

Refs #331"
```

---

### Task 4: `references/rust.md`（唯一语言层，本轮范围）

**Files:**
- Create: `skills/gf-architecture-diagram/references/rust.md`

**Interfaces:**
- Consumes: Task 3 的 Language Layer Contract（四个二级标题）
- Produces: Task 6 dogfooding 实跑所用的提取命令；Task 5 金标样例所依据的真实节点/边集合

- [ ] **Step 1: 写入 `skills/gf-architecture-diagram/references/rust.md`**

````markdown
# Rust Extraction Layer

**Detection:** `Cargo.toml` at project root (per `gf-quality/references/detector.md`).

## 提取命令

**单次运行，捕获输出后复用。**

```bash
cargo metadata --no-deps --format-version=1 > /tmp/gf-arch-metadata.json
```

`--no-deps` 是刻意的：只需要工作区成员（`packages[]`）与它们互相之间的依赖，
不需要展开每个成员的完整第三方依赖树——那会把外部 crate 也拉进 `packages[]`，
污染「内部模块」的判定。

## 模块判定

一个 `packages[]` 条目算作**内部模块**（成为图中的节点），当且仅当它出现在
`cargo metadata --no-deps` 的输出里——该命令本身已经把工作区外的 crate 排除，
不需要额外过滤逻辑。节点名取 `packages[].name`。

```python
import json
data = json.load(open("/tmp/gf-arch-metadata.json"))
modules = sorted(p["name"] for p in data["packages"])
```

## 边过滤规则

一条边 `A -> B` 成立，当且仅当 `A` 的 `dependencies[]` 中存在一项 `dep`
满足 `dep["name"] == B` **且** `dep.get("path")` 非空（`path` 字段非空是
「这是工作区内路径依赖，不是 crates.io 版本依赖」的判定依据）。

```python
edges = []
for pkg in data["packages"]:
    for dep in pkg["dependencies"]:
        if dep.get("path") and dep["name"] in modules:
            edges.append((pkg["name"], dep["name"]))
edges = sorted(set(edges))
```

`dep["name"] in modules` 是双重校验：`path` 非空已经意味着工作区内，但显式
再校验一次目标名称也在 `modules` 集合里，防止 `cargo metadata` 未来版本的
字段语义变化导致误判。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `cargo` 不可用 | 停止，不改用手工解析 `Cargo.toml`——那是本 Issue 明确要避免的语义推断 |
| `cargo metadata` 因工作区配置错误失败 | 报告 cargo 的原始错误，不修复 `Cargo.toml`，不重试 |
| 输出为空 `packages[]`（非工作区项目） | 报告「未检测到工作区成员，无内部模块可绘制」，不生成图 |

不得自行安装工具，不得用 `cargo clean` 规避元数据读取问题。
````

- [ ] **Step 2: 运行断言，确认 references 契约转绿**

Run: `make check-architecture-diagram-skill`
Expected: `✓ AC#2` / 四段契约齐备 / `✓ AC#4/5 提取命令为结构化解析（非语义推断）` 通过；`✗ AC#6 缺少 .../examples/rust.svg` 仍失败，退出码 1。

- [ ] **Step 3: 实跑提取命令，确认能拿到本仓库真实的模块/边集合**

Run:
```bash
cargo metadata --no-deps --format-version=1 > /tmp/gf-arch-metadata.json
python3 - <<'PYEOF'
import json
data = json.load(open("/tmp/gf-arch-metadata.json"))
modules = sorted(p["name"] for p in data["packages"])
edges = []
for pkg in data["packages"]:
    for dep in pkg["dependencies"]:
        if dep.get("path") and dep["name"] in modules:
            edges.append((pkg["name"], dep["name"]))
print("modules:", modules)
print("edges:", sorted(set(edges)))
PYEOF
rm -f /tmp/gf-arch-metadata.json
```
Expected: `modules` 含 `gitflow-cli`、`gitflow-core`、`gitflow-github`、`gitflow-gitlab`、
`gitflow-gitcode`、`gitflow-cli-adapter-utils`、`e2e-core`、`e2e-github`、`e2e-gitlab`、
`e2e-gitcode`、`release-signer`（工作区当前全部成员）；`edges` 含
`('gitflow-cli', 'gitflow-core')`、`('gitflow-cli', 'gitflow-gitcode')`、
`('gitflow-cli', 'gitflow-github')`、`('gitflow-cli', 'gitflow-gitlab')`、
`('gitflow-gitcode', 'gitflow-cli-adapter-utils')`、`('gitflow-gitcode', 'gitflow-core')`
（gitlab/github 同理）、`('e2e-gitcode', 'e2e-core')`（gitlab/github 同理）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-architecture-diagram/references/rust.md
git commit -m "feat(architecture-diagram): Rust 提取层

cargo metadata --no-deps --format-version=1 结构化解析工作区成员与
内部路径依赖边。模块判定：出现在 --no-deps 输出即为内部模块。
边过滤：dependencies[].path 非空且目标名称在模块集合内。
不做语义推断，不手工解析 Cargo.toml。

Refs #331"
```

---

### Task 5: `examples/rust.svg`（金标样例）

**Files:**
- Create: `skills/gf-architecture-diagram/examples/rust.svg`
- Create (temporary, removed at end of task): none — the `.dot` source used to render it is inlined below and not committed separately; only the rendered `.svg` is a skill artifact.

**Interfaces:**
- Consumes: Task 4 的真实模块/边集合（Step 3 的实跑结果），作为样例的拓扑依据
- Produces: Task 3 Stage 2 引用的 `examples/rust.svg`；Task 6 dogfooding 渲染时对照的视觉基准

- [ ] **Step 1: 写一份精简但代表性的 `.dot`，覆盖既有 `docs/architecture-diagram.dot` 的分层/配色约定**

```bash
cat > /tmp/gf-arch-gold.dot <<'EOF'
digraph gf_architecture_gold_sample {
    rankdir=TB;
    fontname="Helvetica";
    node [fontname="Helvetica", fontsize=10, shape=box, style=filled];
    edge [fontname="Helvetica", fontsize=8];
    label="Gold Sample — Rust Workspace";
    labelloc=t;
    fontsize=14;
    nodesep=0.5;
    ranksep=0.8;
    pad=0.4;

    subgraph cluster_cli {
        label="apps/cli";
        style=filled;
        color="#dae8fc";
        gitflow_cli [label="gitflow-cli", fillcolor="#dae8fc"];
    }

    subgraph cluster_core {
        label="crates/core";
        style=filled;
        color="#e1d5e7";
        gitflow_core [label="gitflow-core", fillcolor="#e1d5e7"];
    }

    subgraph cluster_adapters {
        label="Platform Adapters";
        style=filled;
        color="#f5f5f5";
        gitflow_github [label="gitflow-github", fillcolor="#f5f5f5"];
        gitflow_cli_adapter_utils [label="gitflow-cli-adapter-utils", fillcolor="#f5f5f5"];
    }

    gitflow_cli -> gitflow_core;
    gitflow_cli -> gitflow_github;
    gitflow_github -> gitflow_core;
    gitflow_github -> gitflow_cli_adapter_utils;
}
EOF
dot -Tsvg /tmp/gf-arch-gold.dot -o skills/gf-architecture-diagram/examples/rust.svg
rm -f /tmp/gf-arch-gold.dot
```

- [ ] **Step 2: 用 Task 2 的脚本验证金标样例自身几何干净**

Run: `python3 skills/gf-architecture-diagram/scripts/review_svg.py skills/gf-architecture-diagram/examples/rust.svg`
Expected: `CLEAN: no label overflow or element overlap detected`，退出码 0。如果不是，调整 Step 1 的 `.dot`（放宽 `nodesep`/`ranksep` 或缩短标签）后重新渲染，直到几何干净——金标样例本身不能是一份几何有问题的样例。

- [ ] **Step 3: 运行完整断言，确认 AC#6 转绿**

Run: `make check-architecture-diagram-skill`
Expected: `✓ AC#6 金标样例已落盘`；全部 AC 转绿，最后一行 `ALL CHECKS PASSED`，退出码 0。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-architecture-diagram/examples/rust.svg
git commit -m "feat(architecture-diagram): Rust 金标样例

精简的三集群（CLI / Core / Adapters）示例图，配色沿用既有
docs/architecture-diagram.dot 的分层配色约定。已过 review_svg.py
几何回验（无越界、无重叠）。Stage 2 生成前必读此样例收敛输出格式。

Refs #331"
```

---

### Task 6: Dogfooding 实跑 —— 重新生成本仓库的架构图

**Files:**
- Modify: `docs/architecture-diagram.dot`（用结构化提取的真实依赖数据替换手工维护版本）
- Create: `docs/assets/architecture-diagram.svg`（渲染产物，与既有 `docs/assets/demo.svg` 同目录）
- Modify: `docs/index.md`（如尚未收录架构图产物，补一行引用）

**Interfaces:**
- Consumes: Task 4 的提取命令、Task 3 的五阶段协议、Task 5 的金标样例
- Produces: 无（终端产物）

- [ ] **Step 1: Stage 0+1 — 探测并提取本仓库真实模块/边集合**

Run:
```bash
cargo metadata --no-deps --format-version=1 > /tmp/gf-arch-metadata.json
python3 - <<'PYEOF' > /tmp/gf-arch-topology.txt
import json
data = json.load(open("/tmp/gf-arch-metadata.json"))
modules = sorted(p["name"] for p in data["packages"])
edges = []
for pkg in data["packages"]:
    for dep in pkg["dependencies"]:
        if dep.get("path") and dep["name"] in modules:
            edges.append((pkg["name"], dep["name"]))
print("MODULES", modules)
print("EDGES", sorted(set(edges)))
PYEOF
cat /tmp/gf-arch-topology.txt
```
Expected: 与 Task 4 Step 3 记录的结果一致（11 个模块，含 `gitflow-cli`/`gitflow-core`/三个平台适配器/`gitflow-cli-adapter-utils`/四个 `e2e-*`/`release-signer`）。

- [ ] **Step 2: Stage 2 — 读金标样例，确认分层配色约定**

Run: `cat skills/gf-architecture-diagram/examples/rust.svg | head -5`
确认样例采用「按 crate 所在目录分 cluster + 既有配色」的约定（`#dae8fc` CLI / `#e1d5e7` Core / `#f5f5f5` Adapters），生成时延用这套配色，并为 `e2e-*`、`release-signer` 补充与既有 `docs/architecture-diagram.dot` 一致的新增 cluster 颜色（沿用其 `#fff2cc` 等既有配色板，不新造配色）。

- [ ] **Step 3: Stage 3 — 用真实拓扑重写 `docs/architecture-diagram.dot` 并渲染**

按 Step 1 的 11 模块、Step 2 的配色约定，重写 `docs/architecture-diagram.dot`（保留原有的 CLI/Core/Adapters/Ecosystem 分层，新增 `e2e-*` 与 `release-signer` 所在的 cluster），然后渲染：

```bash
dot -Tsvg docs/architecture-diagram.dot -o docs/assets/architecture-diagram.svg
```

- [ ] **Step 4: Stage 4 — 几何回验**

Run: `python3 skills/gf-architecture-diagram/scripts/review_svg.py docs/assets/architecture-diagram.svg`
Expected: `CLEAN: ...`，退出码 0。若报告越界或重叠，回到 Step 3 调整 `.dot`（放宽间距或缩短标签）后重新渲染，**不得**直接编辑生成的 `.svg`。

- [ ] **Step 5: Stage 5 — 确定性检查**

Run:
```bash
dot -Tsvg docs/architecture-diagram.dot -o /tmp/gf-arch-rerender.svg
python3 - <<'PYEOF'
import re
def topology(path):
    text = open(path).read()
    titles = sorted(set(re.findall(r'<title>([^<]+)</title>', text)))
    return titles
a = topology("docs/assets/architecture-diagram.svg")
b = topology("/tmp/gf-arch-rerender.svg")
assert a == b, f"topology mismatch: {a} != {b}"
print("DETERMINISTIC:", len(a), "titled elements match")
PYEOF
rm -f /tmp/gf-arch-rerender.svg /tmp/gf-arch-metadata.json /tmp/gf-arch-topology.txt
```
Expected: `DETERMINISTIC: N titled elements match`，无 `AssertionError`。

- [ ] **Step 6: 更新 `docs/index.md`（若架构图产物尚未收录）**

Run: `grep -n 'architecture-diagram' docs/index.md`

若无输出，在合适位置（靠近现有对 `docs/architecture-diagram.dot` 的引用，若存在）补一行：

```markdown
- [`architecture-diagram.dot`](./architecture-diagram.dot) / [`assets/architecture-diagram.svg`](./assets/architecture-diagram.svg) — dependency-derived architecture diagram, regenerated via `/gf-architecture-diagram` from `cargo metadata`; do not hand-edit the `.svg`.
```

若已有引用该文件的行，改为同时提及 `.svg` 渲染产物与「不要手工编辑」的约束，而不是新增重复行。

- [ ] **Step 7: 运行完整验证套件**

Run: `make check-architecture-diagram-skill && make check-agent-sync`
Expected: 两条命令均以 `ALL CHECKS PASSED`（或 `check-agent-sync` 自身的成功输出）结束，退出码均为 0。

- [ ] **Step 8: Commit**

```bash
git add docs/architecture-diagram.dot docs/assets/architecture-diagram.svg docs/index.md
git commit -m "docs(architecture-diagram): 本仓库 dogfooding 实跑

用 cargo metadata --no-deps 结构化提取真实工作区拓扑（11 个内部模块），
重写 docs/architecture-diagram.dot 补齐 e2e-*/release-signer 缺失的
cluster，渲染为 docs/assets/architecture-diagram.svg。
几何回验干净，重新生成两次拓扑一致。

docs/index.md 补充产物引用与「不要手工编辑 .svg」的约束。

Refs #331"
```

---

## Self-Review

**1. Spec coverage**

| Issue #331 验收标准 | 对应 Task |
|---|---|
| SKILL.md 正文不含任何单一语言的清单文件名或包管理器名 | Task 3 · Task 1 的 AC#1 断言 |
| `references/` 下至少含 `rust.md`，结构可扩展 | Task 4 · Task 1 的 AC#2 断言 |
| 语言探测复用 `detector.md`，不自行实现 | Task 3 Stage 0 · Task 1 的 AC#3 断言 |
| 提取出的模块集合与依赖清单一致，不缺失现存模块 | Task 4 模块判定 · Task 6 Step 1 实跑核对 |
| 图中不出现依赖清单里不存在的依赖边 | Task 4 边过滤规则 · Task 6 Step 1 实跑核对 |
| 生成前读取 `examples/` 金标样例，输出格式与样例一致 | Task 5 · Task 3 Stage 2 · Task 1 的 AC#6 断言 |
| 生成后做几何回验，越界或重叠则本次生成为假 | Task 2（脚本）· Task 3 Stage 4 · Task 6 Step 4 |
| 产出格式为 SVG，不产出大体积 PNG | Task 3 Stage 3 · Task 1 的 AC#8 断言 |
| 重新生成两次结果稳定 | Task 3 Stage 5 · Task 6 Step 5 |

无缺口。

**2. Placeholder scan**

无 TBD / TODO / "similar to Task N" / "add appropriate error handling"。`review_svg.py`、
`SKILL.md`、`references/rust.md`、金标样例的 `.dot` 源、dogfooding 步骤均逐字给出完整
内容，无交叉引用式占位。

**3. Type consistency**

- 四段契约标题在 Task 1 断言、Task 3 契约表、Task 4 实际文件中一律为 `## 提取命令` / `## 模块判定` / `## 边过滤规则` / `## 工具缺失降级`。
- 提取命令在 Task 1 断言、Task 4 正文、Task 6 Step 1/3 中一律为 `cargo metadata --no-deps --format-version=1`。
- 脚本文件名在 Task 1/2/3/5/6 中一律为 `scripts/review_svg.py`；调用方式一律为 `python3 <path> <svg>`，退出码约定（0/1/2）在 Task 2 与 Task 1 断言中一致。
- 金标样例路径在 Task 3 Stage 2、Task 5、Task 1 断言、Task 6 Step 2 中一律为 `examples/rust.svg`（对应语言探测输出 `rust`）。
- Makefile target 名在全文一律为 `check-architecture-diagram-skill`。
- 阶段编号（Stage 0-5）在 Task 3 SKILL.md 正文与 Task 6 dogfooding 步骤标题中一一对应。
