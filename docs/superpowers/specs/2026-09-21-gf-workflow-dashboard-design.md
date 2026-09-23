# workflow 合同渲染进度看板 HTML 设计

> **Issue:** #345
> **Workflow:** wf-2026-09-21-005
> **Date:** 2026-09-21
> **Status:** Approved (in-chat, section by section)

## 1. Context

`gf-workflow`/`gf-workflow-batch` 的执行状态落盘在 `.cache/workflows/`（`active/*.json`、`archive/<YYYY-MM>/*.json`），合同字段已经很丰富（`mode`、`current_phase`、`phases.{1-4}.{name,status,started_at,completed_at,executor,evidence}`）。目前只能靠人工读 JSON 查看进度，批量跑多个 Issue 时没有可视化手段。

## 2. Goal

新增一个纯 Python3 标准库渲染脚本，把 `.cache/workflows/active/*.json` 派生为单文件、自包含、可离线打开的 HTML 进度看板。

## 3. Non-Goals

- 不渲染 `archive/` 下已归档的历史工作流（本次范围只覆盖 `active/`）。
- 不做实时 WebSocket/服务端推送——用 `<meta http-equiv="refresh">` 定时刷新页面，配合外部重跑生成命令。
- 不引入任何第三方 Python 包或 CDN 资源。
- 不修改 `gf-workflow` 合同的 schema 或写入逻辑，只读取现有字段。

## 4. Component & Placement

- 渲染脚本：`scripts/render-workflow-dashboard.py`（Python3 标准库：`json`、`html`、`glob`、`sys`、`datetime`）。
- 触发方式：新增 Makefile 目标 `render-workflow-dashboard`，风格对齐现有 `check-*` 系列目标。
- 输出：`.cache/workflows/dashboard.html`（`.cache/` 已被 `.gitignore` 覆盖，天然满足"产物不得提交进仓库"）。
- 数据源：仅 `.cache/workflows/active/*.json`，不读取 `archive/` 或任何其他路径。

## 5. Page Structure & Coloring

- 顶部固定横幅：`⚠️ 派生视图 — 请勿手工编辑，由 make render-workflow-dashboard 从 .cache/workflows/active/*.json 生成`。
- 每个合同渲染为一张卡片，按 `workflow_id` 分组（不同合同互不覆盖），卡片头部展示 `title`、`mode`、`current_phase`。
- 每张卡片内四格代表 Phase 1-4，每格显示 `name` + `status`，按下表着色：

| status | 颜色 |
|---|---|
| `complete` | 绿色 |
| `in_progress` | 蓝色 |
| `blocked` | 红色 |
| `skipped` | 灰色（斜体） |
| `pending` | 浅灰 |

- 每格下方小字展示 `started_at`/`completed_at`/`executor`；字段缺失显示 `—`，不报错、不留空白导致布局错位。
- `<meta http-equiv="refresh" content="5">`：页面每 5 秒自动重新加载，配合外部重跑渲染命令更新内容（脚本本身不常驻、不监听文件变化）。

## 6. Error Handling

- **逐文件容错**：`active/*.json` 中某文件 JSON 解析失败或缺少必需字段（`workflow_id`、`phases`）时，该文件渲染成一张"⚠️ 解析失败"的错误卡片（文件名 + 具体异常信息），不中断其余文件的渲染，也不产出空白页。
- **空目录**：`active/` 不存在或为空 → 渲染"当前没有活跃工作流"的提示页面，仍是合法 HTML。
- **脚本级失败**：仅当整体不可恢复（如无权限写输出文件）时以非零退出码 + stderr 可读报错终止，供 `make` 目标判断成功/失败。

## 7. Files to Touch

- `scripts/render-workflow-dashboard.py`（新建）
- `Makefile`（新增 `render-workflow-dashboard` 目标，帮助文案注明"生成 .cache/workflows/dashboard.html，派生视图勿手编"）

## 8. Acceptance Criteria Mapping

| Issue #345 AC | 对应章节 |
|---|---|
| 输入仅 `.cache/workflows/` 合同 JSON | §4 |
| 四阶段按 status 着色，五态可区分 | §5 |
| 展示 mode/current_phase/时间戳/executor | §5 |
| 自包含单文件，无外部 CDN | §4（标准库 + 内联 CSS，无外链） |
| 产物落在 gitignore 覆盖路径 | §4（`.cache/workflows/dashboard.html`） |
| 明确标注派生视图，禁止手编 | §5（横幅） |
| 合同损坏/缺字段给出可读报错，不静默空白 | §6 |
| 多合同按 workflow_id 分组，互不覆盖 | §5（单文件多卡片，非多文件） |
