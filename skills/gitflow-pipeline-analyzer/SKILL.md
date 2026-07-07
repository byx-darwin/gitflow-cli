---
name: gitflow-pipeline-analyzer
description: |
  Use when the user wants to analyze CI/CD pipeline health (success rate trends, failure patterns, duration bottlenecks), diagnose flaky tests, or generate a pipeline improvement report.
  当用户想要分析 CI/CD 流水线健康状况（成功率趋势、失败模式、耗时瓶颈）、诊断 flaky test 或生成流水线改进报告时使用。
---

# gitflow-pipeline-analyzer

Analyzes CI/CD pipeline health across success-rate trends, failure patterns, and duration. Produces prioritized improvement reports from `docs/templates/pipeline-report.md`. Read-only — no pipeline modifications.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| pipeline health check | 流水线健康检查 | analyze success, failures, duration |
| flaky test diagnosis | 间歇性失败诊断 | identify recurring failures |
| CI analysis / report | CI 分析报告 | generate improvement report |
| pipeline is unstable | 流水线老挂 / 不稳定 | explain why pipelines fail |
| CI is slow | CI 太慢 / 耗时长 | identify duration bottlenecks |
| build keeps failing | build 持续失败 | **NOT** for fixing — analysis only |

## Core Pattern

```bash
command -v gitflow-cli && git rev-parse --show-toplevel    # preconditions
BRANCH=$(git branch --show-current); DAYS=7                 # parameters
gitflow-cli pipeline report --branch "$BRANCH" --days "$DAYS"
gitflow-cli pipeline status --branch "$BRANCH"
gitflow-cli pipeline jobs --pipeline-id <longest-id>         # duration deep-dive
```

## Quick Reference

| Goal | Command |
|------|---------|
| Report / Status | `pipeline report --branch <b> --days <n>` / `pipeline status --branch <b>` |
| Jobs / Logs | `pipeline jobs --id <id>` / `pipeline logs --id <id>` |

| Success Rate | Grade | Trend Arrow | Meaning |
|--------------|-------|-------------|---------|
| ≥ 95% | 🟢 Healthy | Latter half higher | 📈 Improving |
| 80%–94% | 🟡 Watch | Latter half lower | 📉 Degrading |
| < 80% | 🔴 Alert | Delta < 5% | ➡️ Stable |

| Priority | Condition |
|----------|-----------|
| 🔴 Urgent | < 80% or持续性 build 失败 |
| 🟠 High | 80%–94% or flaky test |
| 🟡 Medium | duration ↑ > 20% or lint repeats |
| 🟢 Low | ≥ 95% but room to optimize |

## Implementation

### Preconditions

- `command -v gitflow-cli` succeeds
- `git rev-parse --show-toplevel` succeeds
- Branch has pipeline runs in range

### Step 1: Gather

```bash
gitflow-cli pipeline report --branch "$BRANCH" --days "$DAYS"
gitflow-cli pipeline status --branch "$BRANCH"
```

Extract total, passed, failed, canceled, avg/max duration, per-job rate. **Empty → prompt to widen scope; stop.**

### Step 2: Success-Rate Trend

`passed / total × 100%` → grade. Split range in half → trend arrow.

### Step 3: Failure Patterns

Group by job. Classify: build / test / lint / deploy / timeout. ≥ 3 consecutive = persistent; intermittent = flaky; same stage cluster = shared root cause.

### Step 4: Duration Distribution

Longest run → `pipeline jobs` → bottleneck. Use P50/P90/P95 — not just mean.

### Step 5: Report

Render from `docs/templates/pipeline-report.md`. Required: overview, narrative, failure table, duration table, prioritized suggestions.

## Error Handling

| Error | Recovery |
|-------|----------|
| `pipeline report` empty | Prompt to widen scope; stop |
| `pipeline report` non-zero | Output error; suggest wider scope; stop |
| `pipeline jobs` / `logs` failure | Skip deep-dive; report available |

## Responsibility

### ✅ In Scope

- Fetch data via `report` / `status` / `jobs` / `logs`
- Analyze 3 dimensions: success-rate trend, failure pattern, duration
- Generate report from `docs/templates/pipeline-report.md`
- Produce prioritized suggestions

### ❌ Out of Scope

- Modify CI config (`.gitlab-ci.yml`, `.github/workflows/*.yml`)
- Retry / cancel / trigger pipelines
- Auto-create issues or PRs
- Push results to Slack / email / external channels
- Fix root causes

### 🚫 Do Not

- ❌ Trigger, retry, cancel, or modify any pipeline run
- ❌ Edit CI configuration files
- ❌ Auto-create issues / PR based on findings
- ❌ Send reports to external channels without explicit request
- ❌ Generate a report when `pipeline report` returns no data
- ❌ Fabricate trend numbers when data is insufficient

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Just retry the failed pipeline" | Read-only; never trigger |
| "Fix the CI config while I'm at it" | Out of scope; analysis only |
| "Auto-create an issue to track this" | Out of scope; user decides |
| "Send report to the team chat" | Out of scope; user decides |
| "User said fix it" | Redirect to `/gitflow-workflow` |
| "Data's thin — I'll extrapolate" | Widen scope; never fabricate |

## Red Flags

- 🚩 "Fix the pipeline config" — refuse; analysis only
- 🚩 "Retry all failures" — refuse; never trigger
- 🚩 "Send this to Slack" — refuse without explicit request
- 🚩 "Skip the trend analysis" — non-skippable
- 🚩 "Just give a quick verdict" — all 3 dimensions required
- 🚩 "No data, just guess" — refuse; widen scope

## Test Scenarios

### Scenario 1: Happy Path

- **Given** Branch with ≥ 1 run in range
- **When** "流水线最近老挂，帮我分析一下"
- **Then** Report with 3 dimensions + prioritized suggestions; no pipeline modified

### Scenario 2: Negative — Wants Fix

- **Given** "帮我修一下流水线配置"
- **When** Targets CI config edits
- **Then** Do NOT load; redirect — skill is read-only

### Scenario 3: Boundary — Tempted to Retry

- **Given** Report shows repeated failures
- **When** Claude considers retrying failed ones
- **Then** Refuse; cite 🚫 Do Not; stick to report

### Scenario 4: Error — Empty Report

- **Given** New branch, no runs in range
- **When** `pipeline report` returns empty
- **Then** "No data — widen `--days` or change branch"; no empty report

## Success Criteria

- [ ] All 3 dimensions analyzed (success-rate trend, failure pattern, duration)
- [ ] Report rendered from `docs/templates/pipeline-report.md`
- [ ] Grade + trend + priority follow thresholds
- [ ] No pipeline triggered, modified, or retried
- [ ] No CI config edited, no issue/PR auto-created, no external push
- [ ] Empty data handled with scope-widening prompt, not fabricated report

## See Also

- `gitflow-precommit` — pre-commit checks correlate with pipeline failures
- `gitflow-quality` — code quality gate ties into CI health
- `gitflow-regression` — regression investigation on test failures
- `gitflow-weekly-report` — cites pipeline analysis in weekly report
- `superpowers:systematic-debugging` — systematic debug for pipeline failures
- `docs/templates/pipeline-report.md` — report template
