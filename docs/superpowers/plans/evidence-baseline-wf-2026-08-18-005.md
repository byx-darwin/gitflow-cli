# 证据基线 — 主动上报 bug 功能多角色评估

> **工作流：** `wf-2026-08-18-005`（standard）
> **评估对象：** 主动上报 bug 功能端到端链路
> **日期：** 2026-08-18
> **说明：** 每条证据含 `文件:行号` 或命令实测输出；状态 🔴=阻断 / 🟠=重要 / 🟢=良好

---

## 一、写入端 — `apps/cli/src/error_reporter.rs` + `main.rs`

| # | 证据 | 锚点 | 状态 |
|---|------|------|------|
| E1 | `maybe_report_error` 双重 gate：`should_skip_reporting`（stderr 非 TTY）+ `is_co_contribution_enabled` | error_reporter.rs:175-193 | 🟠 |
| E2 | `should_skip_reporting`：stderr 是 TTY 则跳过 → 非交互模式才写报告 | error_reporter.rs:243-246 | 🟢 |
| E3 | `is_co_contribution_enabled`：检查项目 `.claude/settings.json` 再检查全局，需 `gitflow.co_contribution=true` | error_reporter.rs:204-224 | 🟠 |
| E4 | **全局已启用** `co_contribution: true`（项目 settings 无此字段）→ 功能实际被全局开关 gate，用户不可发现 | ~/.claude/settings.json:37-39 | 🟠 |
| E5 | `write_to_disk` 创建目录树 + 覆盖写单文件 `pending.json` | error_reporter.rs:87-104 | 🟢 |
| E6 | `set_pending_file_permissions` → `0o600` owner-only | error_reporter.rs:148-156 | 🟢 |
| E7 | `sanitize_error_message` 脱敏 GitHub token（`ghp_`/`github_pat_` 正则）与家目录路径 | error_reporter.rs:136（+LazyLock 正则约 119-134） | 🟢 |
| E8 | `generate_unique_id`：纳秒时间戳 XOR PID + 斐波那契哈希 → 防碰撞 | error_reporter.rs:254-262 | 🟢 |
| E9 | best-effort：`report_error_noninteractive` 丢弃 I/O 错误，不阻塞退出码 | main.rs:152-163 | 🟢 |
| E10 | 调用点覆盖 main.rs 3 处（命令分派 98/125 + 顶层错误 144） | main.rs:98,125,144 | 🟢 |
| E11 | 单测覆盖：`test_should_create_error_report_from_error`、`test_should_write_pending_json_to_disk` | error_reporter.rs:331-369 | 🟢 |

## 二、触发端 — `.claude/settings.json` Stop Hook

| # | 证据 | 锚点 | 状态 |
|---|------|------|------|
| E12 | Stop Hook matcher = `gitflow` → 仅当 Stop reason 含 `gitflow` 才触发 | .claude/settings.json (hooks.Stop[0].matcher) | 🟠 |
| E13 | Hook command 为 bash 执行 `hooks/auto-report-bug.sh`（无退出码强制） | .claude/settings.json (hooks.Stop[0].hooks[0]) | 🟢 |
| E14 | **Hook 只输出 banner，不自动调用 skill** → 依赖模型看见 banner 后自主加载 `gf-autoreport-bug` | auto-report-bug.sh:127-136 | 🔴 |

## 三、捕获端 — `hooks/auto-report-bug.sh`

| # | 证据 | 锚点 | 状态 |
|---|------|------|------|
| E15 | TTY guard：`[ -t 1 ] || [ -t 0 ]` 时退出 → 交互终端不触发 | auto-report-bug.sh:38 | 🟢 |
| E16 | 无 pending.json 时静默退出（exit 0） | auto-report-bug.sh:35-37 | 🟢 |
| E17 | 浅 JSON 校验：缺 `error_code` → 改名 `.invalid` 并警告 | auto-report-bug.sh:42-48 | 🟢 |
| E18 | **认证用 `gh auth status`（非 `gf`）** → 与 skill 强制 `gf` CLI 口径不一致 | auto-report-bug.sh:77 | 🟠 |
| E19 | auth cache：`.cache/auth-cache/{platform}.ttl`，TTL 86400，命中则跳过 live 检查 | auto-report-bug.sh:60-76 | 🟢 |
| E20 | auth 失败兜底：输出登录指引 + Issue 模板，保留 pending.json | auto-report-bug.sh:88-123 | 🟢 |
| E21 | **banner 引用过时技能名 `gitflow-autoreport-bug`**（实际为 `gf-autoreport-bug`）→ 模型可能加载失败 | auto-report-bug.sh:11,133（路径 134 正确） | 🔴 |
| E22 | banner 在 `$CLAUDE_DIR/skills/gf-autoreport-bug/SKILL.md` 指对路径 | auto-report-bug.sh:134 | 🟢 |
| E23 | Hook 测试 5 个：silent exit / .invalid / auth fail / auth ok+cache seed / cache hit skip | hooks/tests/auto-report-bug.bats | 🟢 |

## 四、处理端 — `gf-autoreport-bug` skill + 支撑文档

| # | 证据 | 锚点 | 状态 |
|---|------|------|------|
| E24 | description 为触发条件导向（Use when pending.json exists + 双语）→ 符合规范 | SKILL.md:3-6 | 🟢 |
| E25 | 结构含 CLI Requirement/Preconditions/Decision Flow/When NOT to Use/Workflow/Error Handling/Common Mistakes | SKILL.md 章节 | 🟢 |
| E26 | **缺 When to Use 正向触发表、Red Flags、Rationalization 反制表** | SKILL.md 结构比对 | 🟠 |
| E27 | 词数 739（超 Superpowers 500 词建议） | wc -w SKILL.md | 🟠 |
| E28 | 去重命令：skill 版 `gf issue list --repo ... --search "[auto-report] {command} {error_code}"`（**无 `--state all`**） | SKILL.md:105 | 🟠 |
| E29 | 去重命令：params doc 版 `gf issue list --search "[auto-report] {cmd} {err}" --state all`（**有 `--state all`**）→ 两者不一致 | params.md:43 | 🟠 |
| E30 | Issue 创建用 `--label "auto-report"`；**实测该 label 在仓库不存在**（`gf label list` 未命中）→ 创建可能失败 | SKILL.md:106 + gf label list 实测 | 🔴 |
| E31 | `--title "[auto-report] gitflow {command} — {error_code}"`（`gitflow` 前缀与 gf 品牌不一致） | SKILL.md:106 | 🟠 |
| E32 | 目标仓库固定 `byx-darwin/gitflow-cli` | SKILL.md:95-99 | 🟢 |
| E33 | 职责边界：只报告不修复 + 🚫 Forbidden 清单 + Fix Flow 用户发起 | SKILL.md:70-93 | 🟢 |
| E34 | skill 源副本同步：`skills/gf-autoreport-bug/SKILL.md` 与 `.claude/skills/` 一致 | diff 实测 | 🟢 |
| E35 | params doc 命令速查与 skill 存在不一致（E28/E29） | params.md:43-45 | 🟠 |

## 五、实测事实（命令输出）

| # | 证据 | 输出 | 状态 |
|---|------|------|------|
| E36 | **存在未处理 pending.json**（2026-08-18T06:59Z，约 8.5h+） | `.cache/bug-reports/pending.json` 内容为 `Invalid state 'invalid'` | 🔴 |
| E37 | **pending.json 是用户输入错误**（`--state invalid`）→ 误报（非 CLI 缺陷） | E36 error_message | 🔴 |
| E38 | **零 `[auto-report]` Issue**：`gf issue list --search "auto-report"`（open+all）均空 | gf 实测 | 🔴 |
| E39 | **`auto-report` label 不存在** | gf label list 实测 | 🔴 |
| E40 | `co_contribution` 全局 true / 项目无 → gate 不可发现 | ~/.claude/settings.json:37 | 🟠 |

---

## 六、证据统计

- **🔴 阻断级（7）**：E14, E21, E30, E36, E37, E38, E39
- **🟠 重要级（9）**：E1, E3, E4, E12, E18, E26, E27, E28, E29, E31, E35, E40
- **🟢 良好级（16）**：E2, E5, E6, E7, E8, E9, E10, E11, E13, E15, E16, E17, E19, E20, E22, E23, E24, E25, E32, E33, E34

> 注：统计含重复标注的混合项，供下游角色评估引用锚点使用。
