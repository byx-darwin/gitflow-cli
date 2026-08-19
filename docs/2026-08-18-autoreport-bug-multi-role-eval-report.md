# 多角色评估报告 — 主动上报 bug 功能是否 OK

> **工作流：** `wf-2026-08-18-005`（standard）
> **Issue：** #209
> **评估日期：** 2026-08-18
> **评估对象：** 主动上报 bug 功能端到端链路（`error_reporter.rs` → Stop Hook → `auto-report-bug.sh` → `gf-autoreport-bug` skill → Issue）
> **方法论：** Issue #93 五角色 × 三层面（文档规范性 / 职责边界 / 可测试性&可靠性）
> **证据基线：** `docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`（E1-E40）

---

## 一、总体判定

# 🔴 NOT OK（当前状态不可用）

**一句话结论：** 「主动上报 bug」功能的**写入端设计正确且安全**，但**处理端从未成功产出过一条 Issue**，且存在**确定性最后一步失败**（label 缺失）与**系统性误报**（用户输入错误被当 bug），当前不能声称该功能可用。

**修复 P0 后可达到：** 🟡 有条件 OK（设计意图合理，链路修复后价值成立）。

### 五角色判定汇总

| 角色 | 判定 | 一句话理由 | 证据锚点 |
|------|------|-----------|---------|
| 产品负责人 | 🟡 有条件 OK | 设计意图成立但零交付 + 系统性误报 | E36, E37, E38 |
| 架构师 | 🟡 有条件 OK | 写入端优秀但触发链非确定 + 平台口径矛盾 | E12, E14, E18 |
| DevOps | 🔴 NOT OK | 处理段从未成功 + label 缺失确定失败 | E30, E38, E39 |
| 开源社区运营 | 🟡 有条件 OK | 模板好但误报不可控 | E37, E39 |
| 终端用户 | 🟡 有条件 OK | 隐私扎实但知情同意缺失 | E4, E6, E7 |

> **判定规则：** 5 角色中 1 个 NOT OK（DevOps，最严格视角）。因「零产出」是硬事实、且存在确定失败点，**总体取 NOT OK**。但写入端（安全/脱敏/权限）全部 🟢，说明设计根基良好——故判定是「当前不可用」，不是「设计错误」。

---

## 二、核心事实（证据锚点）

1. **存在未处理 pending.json，且为误报**（E36/E37）：实测 `.cache/bug-reports/pending.json` 内容是 `Invalid state 'invalid'`——这是**用户传参错误**，非 CLI 缺陷。写入端无错误分类，一律照收。
2. **零 `[auto-report]` Issue**（E38）：`gf issue list --search "auto-report"`（open+all）均空 → 功能**从未交付过一次**。
3. **`auto-report` label 不存在**（E39）：skill 创建 Issue 用 `--label "auto-report"`（SKILL.md:106），GitHub 对不存在的 label 会 422 → **即使触发成功也在最后一步失败**。
4. **触发链非确定性**（E12/E14）：Stop Hook matcher=`gitflow` + Hook 只输出 banner「请加载 skill」、依赖模型自主加载 → 至少 2 跳非确定。
5. **`co_contribution` 全局开启但不可发现**（E4）：`~/.claude/settings.json:37` 为 true，项目无字段、无文档说明。
6. **认证口径矛盾**（E18）：Hook 用 `gh auth status`，skill 强制 `gf` CLI → GitLab/GitCode 判定失真。
7. **安全与写入端扎实**（🟢）：token/家目录脱敏（E7）、`0o600`（E6）、best-effort 不阻塞（E9）、id 防碰撞（E8）、单测+hook 测试（E11/E23）。

---

## 三、分级改进建议

### P0 — 阻断正确性（功能不可用根因）

| # | 建议 | 归属环节 | 证据 | 期望效果 |
|---|------|---------|------|---------|
| P0-1 | **创建 `auto-report` label**（或用已存在 label） | GitHub 仓库配置 | E39, E30 | 消除 `gf issue create` 确定性 422 失败 |
| P0-2 | **消除触发非确定性**：由 Stop Hook 直接驱动处理脚本（或确定性加载 skill），不再依赖模型「看见 banner 后自主决定」 | hooks/auto-report-bug.sh + 架构定位 | E14, E12 | 有 bug 必触发，可预期 |
| P0-3 | **写入端错误分类**：区分「真实 CLI 缺陷」vs「用户输入/参数错误」，后者不写入 | apps/cli/src/error_reporter.rs | E37, E36 | 杜绝误报污染 Issue 流 |

### P1 — 重要（可靠性与合规）

| # | 建议 | 归属环节 | 证据 | 期望效果 |
|---|------|---------|------|---------|
| P1-1 | Hook 认证改 `gf auth status --platform {platform}`（读取报告 platform 字段） | hooks/auto-report-bug.sh:77 | E18 | 三平台认证口径统一 |
| P1-2 | 统一去重命令（含 `--state all`）+ 明确去重粒度（避免同键误合并） | SKILL.md:105 vs params.md:43 | E28, E29 | 幂等性保证 |
| P1-3 | 增加处理端日志/可观测性（失败痕迹必留，不依赖模型在场） | hooks + skill | E14 | CI 可确认上报是否跑过 |
| P1-4 | `co_contribution` 可发现 + 可退出（文档、`gf` 命令、安装提示） | 配置 + 文档 | E4 | 知情同意合规 |
| P1-5 | pending.json 支持多报告或失败日志兜底（避免单文件覆盖漏报） | error_reporter.rs | E5 | 多失败不丢报告 |

### P2 — 打磨（规范性）

| # | 建议 | 归属环节 | 证据 | 期望效果 |
|---|------|---------|------|---------|
| P2-1 | 品牌统一 `gitflow` → `gf`（banner 文案 + Issue 标题前缀） | auto-report-bug.sh:11,133 + SKILL.md:106 | E21, E31 | 品牌一致 |
| P2-2 | skill 补 When to Use / Red Flags / Rationalization 反制表；词数压到 <500 | skills/gf-autoreport-bug/SKILL.md | E26, E27 | Superpowers 规范达标 |
| P2-3 | 公开上报前内容预览（用户可选是否公开 error_message） | skill 流程 | E37 | 用户透明 |

---

## 四、三层面评分（结构性）

| 层面 | 评分 | 说明 |
|------|------|------|
| 文档规范性 | 🟡 | description 触发导向 ✅（E24）；缺 When to Use 正向表/Red Flags/反制表（E26）；739 词超标（E27） |
| 职责边界 | 🟢 | 只报告不修复声明 + Forbidden 清单（E33）——全项目最佳之一 |
| 可测试性 & 可靠性 | 🔴 | 写入端测试齐全（E11/E23）✅；但**端到端零验证**、触发非确定、最后一步确定失败（E38/E39/E14） |

---

## 五、结论

**「主动上报 bug」功能当前 NOT OK**——它从未成功交付过一条 Issue，且存在确定失败点与系统性误报。

但必须强调：**不是设计失败**。写入端的安全、边界、可测试性全部优秀；产品意图（收集真实缺陷）合理。这是一个「**前半正确、后半断裂**」的未完成功能。

**建议：** 按 P0-1 → P0-2 → P0-3 修复后，重新做一次端到端验证（目标：产生首条真实 `[auto-report]` Issue），即可达到「有条件 OK」。修复需另起 workflow（本评估仅报告，未改任何代码）。

---

## 六、相关文档

- 设计文档：`docs/superpowers/specs/2026-08-18-autoreport-bug-multi-role-eval-design.md`
- 执行计划：`docs/superpowers/plans/2026-08-18-autoreport-bug-multi-role-eval.md`
- 证据基线：`docs/superpowers/plans/evidence-baseline-wf-2026-08-18-005.md`
- 角色评估（产品/架构）：`docs/superpowers/plans/role-eval-pm-architect.md`
- 角色评估（DevOps/社区/用户）：`docs/superpowers/plans/role-eval-devops-community-user.md`
- 历史分析（2026-07-07）：`docs/research/skill-analysis-gf-autoreport-bug.md`
