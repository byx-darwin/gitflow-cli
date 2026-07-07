# gitflow-weekly_report Refactor — Acceptance Criteria (RED)

> **Purpose:** Capture the requirements that the refactored `SKILL.md` MUST satisfy.
> Each criterion below currently **FAILS** against `skills/gitflow-weekly-report/SKILL.md` (v2.0.0).
> Implementation passes when every criterion is met.

## C1 — Description is pure trigger condition
- [ ] `description` field starts with "Use when..."
- [ ] `description` contains only trigger keywords / contexts
- [ ] `description` does NOT contain functional description ("生成研发周报", "扫描多个 Git 仓库")
- [ ] `description` does NOT contain installation advice ("推荐用户级安装")
- [ ] `description` does NOT contain "TRIGGER when" label

## C2 — Token efficiency
- [ ] Skill body is ≤ 500 Chinese-word-equivalents after extracting repetitive material
- [ ] Output template lives in `docs/templates/weekly-report.tmpl` (referenced, not inlined)

## C3 — Responsibility boundaries (no fabrication)
- [ ] Skill explicitly states: submit counts / dates / hashes MUST come from `git log` output, no estimation, no fabrication
- [ ] "未完成事项" section MUST only surface WIP markers that appear verbatim in commit messages (e.g. "WIP", "TODO", "进行中")
- [ ] "未完成事项" MUST NOT be inferred from commit frequency ("提交偏少" is prohibited)

## C4 — No performance evaluation (explicit)
- [ ] Skill contains explicit prohibition: MUST NOT evaluate any developer's performance, velocity, or output quality
- [ ] Prohibition applies to both positive ("高产") and negative ("偏少", "偷懒") judgments
- [ ] Prohibition is listed in both 🚫 Do Not AND a dedicated scope sentence

## C5 — Prohibition list (🚫 Do Not)
- [ ] 🚫 不得读取 `.git` 之外的文件
- [ ] 🚫 不得估算/编造提交数、日期、hash
- [ ] 🚫 不得对开发者提交频率/数量做绩效评价
- [ ] 🚫 不得基于提交频率推断"未完成事项"
- [ ] 🚫 不得执行 git 写操作 (commit/push/rebase)
- [ ] 🚫 不得将报告发送到外部服务

## C6 — Red Flags
- [ ] 🚩 用户要求"评估某人工作表现" / "比较团队产出"
- [ ] 🚩 用户要求扫描整个主目录或大量无关仓库
- [ ] 🚩 用户要求"编造提交数让报告好看"
- [ ] 🚩 用户要求读取非 git 数据源来补充周报

## C7 — Rationalization anti-excuse table
- [ ] Contains at least 3 rows debunking common slips (e.g. "补充一点上下文没关系" / "只是推断不是编造" / "用户没说我就默认" / "提交少=收尾中很合理")

## C8 — Cross-references
- [ ] See Also lists ≥ 2 related skills (e.g. `gitflow-commit`, `gitflow-repo`, `superpowers:verification-before-completion`)

## C9 — Test scenarios
- [ ] Scenario 1: Happy path (multi-project normal week)
- [ ] Scenario 2: Negative (user asks "评估产出高不高" → must not trigger this skill)
- [ ] Scenario 3: Boundary — user asks to infer "未完成事项" from low commit count → refuse
- [ ] Scenario 4: Error — path does not exist → skip with warning, continue

## C10 — Success criteria
- [ ] Every commit hash cited in output exists verbatim in `git log` output
- [ ] Total commit count matches `git log | wc -l` per project
- [ ] No out-of-scope action taken during skill execution
- [ ] No performance-evaluation language anywhere in the report

## C11 — Structure completeness
- [ ] Contains When to Use
- [ ] Contains Quick Reference
- [ ] Contains Responsibility (✅ In Scope / ❌ Out of Scope / 🚫 Do Not)
- [ ] Contains Red Flags
- [ ] Contains Rationalization Excuses
- [ ] Contains Test Scenarios
- [ ] Contains Success Criteria
- [ ] Contains Trigger Keywords
- [ ] Contains See Also

## C12 — Frontmatter integrity
- [ ] `name: gitflow-weekly-report`
- [ ] `description` is a single YAML block scalar (no embedded newlines that break parsing)
