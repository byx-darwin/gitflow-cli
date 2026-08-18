# 设计文档 — 主动上报 bug 功能 P1/P2 遗留项修复

> **设计日期：** 2026-08-18
> **工作流：** `wf-2026-08-18-007`（standard 模式）
> **来源：** 上轮评估 `wf-2026-08-18-005` P1/P2 建议 + `wf-2026-08-18-006` P0 修复后遗留
> **依据：** `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md`

---

## 1. 目标

修复「主动上报 bug」功能的 4 项 P1/P2 遗留问题，提升可靠性、合规性与规范性：

| # | 子任务 | 类型 | 对应建议 |
|---|--------|------|----------|
| T1 | P1-1 hook 认证口径统一（`gh` → `gf --platform`） | hook | P1-1 |
| T2 | P1-4 co_contribution 知情同意（可发现 + 可退出） | Rust doctor | P1-4 |
| T3 | P2-1 品牌统一（gitflow → gf） | hook + skill + params | P2-1 |
| T4 | P2-2 skill 规范（When to Use/Red Flags/Rationalization + <500 词） | skill | P2-2 |

## 2. 变更面

| 文件 | 变更 | 类型 |
|------|------|------|
| `hooks/auto-report-bug.sh` | 认证改 `gf auth status --platform`；banner 品牌 gf | bash |
| `apps/cli/src/commands/doctor.rs` | 报告 co_contribution 状态 + 退出指引 | Rust |
| `apps/cli/src/commands/skills.rs` | 复用 co_contribution 读取逻辑（只读展示，不改写入） | Rust |
| `skills/gf-autoreport-bug/SKILL.md` | 品牌 gf + 规范章节 + 压缩 | skill 源 |
| `.claude/skills/gf-autoreport-bug/SKILL.md` | 同步副本 | skill 副本 |
| `docs/references/gf-autoreport-bug-params.md` | 品牌 gf + 示例命令 | 文档 |
| `hooks/tests/auto-report-bug.bats` | 认证 mock 改 gf + 品牌断言 | 测试 |

## 3. 详细设计

### T1: P1-1 hook 认证口径统一

**现状：** `hooks/auto-report-bug.sh:77` 用 `gh auth status` 验证 GitHub 认证；pending.json 含 `platform` 字段但被忽略 → GitLab/GitCode 平台判定失真。

**改法：**
- `gh auth status >/dev/null 2>&1` → `gf auth status --platform "$PLATFORM" >/dev/null 2>&1`
- `PLATFORM` 已在脚本前面从 pending.json 提取（`grep '"platform"'`）
- 缺失 platform 时降级为不带 `--platform`（gf 自动检测）

**测试：** bats mock `gh` → mock `gf`（记录调用 + auth 结果）；新增断言验证 `--platform` 传参正确。

### T2: P1-4 co_contribution 知情同意

**现状：** `gitflow.co_contribution` 由 `skills install` 全局写入（历史设计 #82），但无任何读取/展示 → 用户不知道自己已加入、不知道如何退出。

**改法（只读展示，不改写入）：**
- `gf doctor` 增加 `co_contribution` 类别：
  - 状态：`已启用（~/.claude/settings.json）` / `未启用`
  - 说明：`bug 自动上报已开启。如要退出，编辑 ~/.claude/settings.json 移除 gitflow.co_contribution`
- 复用 `error_reporter::read_co_contribution_flag`（已是 `pub(crate)`）

**验证：** `gf doctor` 输出含 co_contribution 状态；doctor 单测覆盖。

### T3: P2-1 品牌统一（gitflow → gf）

**保留不动：** 仓库 URL `byx-darwin/gitflow-cli`（真实 repo 名）、配置键 `gitflow.co_contribution`（既有键名，改造成本高且非品牌展示）。

**统一为 gf：**
| 位置 | 现值 | 改后 |
|------|------|------|
| hook:120 banner | `检测到 gitflow CLI 错误报告` | `检测到 gf CLI 错误报告` |
| SKILL.md:106 标题前缀 | `[auto-report] gitflow {command}` | `[auto-report] gf {command}` |
| params.md:10,25-26,44 示例 | `gitflow issue create` | `gf issue create` |

**验证：** grep 确认 `gitflow` 仅剩 repo URL 与配置键。

### T4: P2-2 skill 规范（<500 词）

**改 `skills/gf-autoreport-bug/SKILL.md`（源）→ 同步 `.claude/skills/` 副本。**

增加章节（Superpowers 规范）：
- **When to Use** 触发表（EN/ZH 双语）
- **Red Flags** 表（越界信号）
- **Rationalization Excuses** 表（合理化借口反制）

压缩策略：
- 精简 `Decision Flow` 冗余表述（保留 mermaid 图）
- 合并重复的「职责边界」段落（保留 ✅/🚫 清单）
- 目标 `< 500` 词（现 739 词）

**验证：** `wc -w < 500`；`diff` 源与副本一致；`make check-agent-sync`。

## 4. 验收标准

- [ ] hook 认证用 `gf auth status --platform`（bats mock gf + 传参断言）
- [ ] `gf doctor` 报告 co_contribution 状态 + 退出指引
- [ ] `gitflow` 品牌残留仅剩 repo URL 与配置键
- [ ] skill 源与副本同步、词数 <500、含 When to Use/Red Flags/Rationalization
- [ ] `make test` + clippy + hook bats 通过

## 5. 范围外

- 不改 `skills install` 写入逻辑（#82 历史设计保持）
- 不实施 P1-2/P1-3/P1-5/P2-3（去重粒度、日志、多报告队列、公开预览）
