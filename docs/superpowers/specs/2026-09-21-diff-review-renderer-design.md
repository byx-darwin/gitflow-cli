# diff 交互式审阅网页渲染设计

> **Issue:** #346（收窄后）· 关联 #397（Phase 4 接入，依赖本 Issue）、#398（annotations 内容自动生成，依赖本 Issue）
> **Workflow:** wf-2026-09-21-006
> **Date:** 2026-09-21
> **Status:** Approved (in-chat, section by section)

## 1. Context

当前 PR 审阅产物全是 Markdown（`gf-pr-inline-review` 的行内评论、`gf-review` 的 `docs/code-review-report-prN-*.md`），审阅人需要在代码和报告间来回对照。参照外部实现 `smallnest/goal-workflow` 的 `skills/understand/`（`scan`+`render` 两段式，纯标准库，产出交互式 diff 审阅页），结合本仓库 Issue #345 刚建立的"派生 HTML 视图"约定（Python3 标准库、`scripts/` 目录、`.cache/` 输出、Makefile 目标、派生视图横幅），设计一个同类工具。

## 2. Scope Decisions（本次澄清阶段的关键决定）

- **范围收窄**：原 Issue #346 的 AC 混合了"渲染核心能力"与"Phase 4 编排接入"，已拆分：#346 只做工具链本身，Phase 4 接入拆到 #397。
- **annotations 内容生成拆分**：伪代码/调用树等语义字段的自动生成需要语言相关的 AST 分析或 LLM 调用，与"零依赖、离线可用、语言无关"的定位冲突，已拆到 #398。#346 只定义 schema 并实现"有内容就展示折叠区块、没有就不显示"的 UI 能力，不实现内容生成。
- **语法高亮**：不内联打包 Prism（体积大，且违背"零依赖"精神），自研覆盖本仓库实际语言（Rust/Python/Shell/Markdown/YAML/TOML/JSON）的轻量正则高亮器。
- **功能范围**：采用参考实现的全套交互功能（不做 MVP 阉割）——三栏布局、行级锚定、hover 联动、伪代码/调用树折叠 UI（内容留空）、侧栏拖拽记忆宽度。

## 3. Non-Goals

- 不做 Phase 4 自动触发（#397）。
- 不做 annotations 内容的自动生成（#398）。
- 不做双向编辑/写回代码——单向只读展示。
- 不引入任何构建工具链（无 Webpack/Vite/npm），纯字符串模板替换。

## 4. Component & Command Shape

单入口脚本 `scripts/render-diff-review.py`（Python3 标准库），两个子命令：

- `scan <base>..<head>`：调用 `git diff` 拿 diff 文本 + `git status --porcelain` 拿未跟踪文件列表，手写 unified diff 解析器，产出结构化 `annotations.json`（含空的语义字段占位）。
- `render <annotations.json>`：读取该 JSON，注入固定 HTML/JS/CSS 模板（纯字符串替换，无模板引擎），产出自包含单文件 HTML。

Makefile 新增目标 `render-diff-review`，默认对 `dev` 与 `HEAD` 之间的 diff 跑一遍 scan+render 两步（对齐 #345 的 Makefile 目标风格）。

## 5. Diff Parser Coverage

手写 unified diff 解析，不依赖第三方 diff 解析库：

- **标准 hunk**：解析 `@@ -l,s +l,s @@` 头，逐行还原双行号（删除行只有旧行号，新增行只有新行号，上下文行两者都有）。
- **重命名**：识别 `rename from`/`rename to` 头，文件树标注"重命名"而非拆成删除+新增。
- **未跟踪文件**：`git diff` 本身不含未跟踪文件，额外用 `git status --porcelain` 找出 `??` 开头的文件，对每个用 `git diff --no-index /dev/null <file>` 生成"全新增"diff 段，合并进同一份结构化输出并标注来源。
- **二进制文件**：识别 `Binary files ... differ`，标注"二进制，无法显示 diff"，不渲染 hunk。

## 6. Template & Interactive Features

- **三栏布局**：文件树（左）/ diff 内容（中，逐行增删底色）/ 注释卡片区（右，展示对应 annotations，空则显示"暂无说明"）。
- **行级锚定**：每行有唯一锚点 id（`file:new_line`/`file:old_line`），点击注释条目 `scrollIntoView` 定位并高亮闪烁。
- **hover 联动**：diff 行与右侧注释卡片鼠标悬停同步高亮，原生事件委托实现。
- **伪代码/调用树折叠**：annotations 里若有 `pseudocode`/`call_tree` 字段，渲染成原生 `<details>/<summary>`；为空则该区块整体不渲染。
- **侧栏拖拽记忆**：文件树/注释区宽度可拖拽，用 `localStorage` 记住宽度（仅同源持久化，不涉及跨用户同步）。
- **语法高亮**：自研正则 tokenizer，按扩展名选规则集（Rust/Python/Shell/Markdown/YAML/TOML/JSON），非目标语言只走 diff 底色不高亮。

## 7. Error Handling & Output

- `scan` 遇到无法解析的片段 → 记为"解析失败"条目（保留原文），不中断其余文件（沿用 #345 的逐项容错惯例）。
- 二进制文件、空 diff → 有明确空状态展示，非渲染失败。
- `render` 阶段 `annotations.json` 缺失/损坏 → 可读报错 + 非零退出，不产出半成品 HTML。
- 产物路径：`.cache/diff-review/<base>..<head>.html`（`.cache/` 已被 `.gitignore` 覆盖），按 base/head 短 SHA 命名，支持多轮结果共存。
- 顶部横幅："派生视图，请勿手工编辑"（对齐 #345）。

## 8. Testing Strategy

- `scripts/tests/test_render_diff_review.py`，standalone `unittest`，不进 `make test`（同 #345 模式）。
- **scan 测试**：构造已知 diff 片段（标准 hunk、重命名、二进制、未跟踪文件、畸形片段）验证 `annotations.json` 结构正确。
- **render 测试**：构造已知 `annotations.json`，验证产出 HTML 含预期结构标记（三栏容器、行锚点 id、折叠区块、语法高亮 CSS 类、banner、无 CDN 链接）——不测试 JS 运行时交互效果（浏览器行为超出标准库测试能力）。
- **AC8 人工验收**：对本仓库一个真实历史 PR 的 diff 跑完整流程，人工打开确认可用，并用脚本断言渲染行数与 `git diff --stat` 吻合。

## 9. Files to Touch

- `scripts/render-diff-review.py`（新建，scan+render 主逻辑）
- `scripts/tests/test_render_diff_review.py`（新建）
- `Makefile`（新增 `render-diff-review` 目标）

## 10. Acceptance Criteria Mapping

| Issue #346 AC | 对应章节 |
|---|---|
| 输入为 git diff，语言无关 | §5 |
| 零第三方依赖，不内联 Prism | §4、§6（自研高亮器） |
| 自包含单文件，离线高亮生效 | §6、§7 |
| 覆盖未跟踪文件与重命名 | §5 |
| 行级锚定 | §6 |
| 三栏/hover/折叠/拖拽记忆全套功能 | §6 |
| 单向只读 | §2 Non-Goals |
| 产物落在 gitignore 覆盖路径 | §7 |
| 真实 PR 验收，行数吻合 | §8 |
