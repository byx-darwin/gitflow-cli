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
