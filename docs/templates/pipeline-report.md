# Pipeline Analysis Report Template

Use this template when rendering the output of `gitflow-pipeline-analyzer`. Fill placeholders with live data from `gitflow-cli pipeline report`, `pipeline status`, and `pipeline jobs`.

```markdown
## Pipeline Analysis Report

**Branch:** <branch>
**Period:** last <days> days
**Generated:** <timestamp>

### Overview

| Metric | Value | Note |
|--------|-------|------|
| Total runs | <n> | |
| Success rate | <x%> | 🟢 / 🟡 / 🔴 |
| Avg duration | <duration> | |
| Max duration | <duration> | Pipeline #<id> |
| Trend | 📈 / 📉 / ➡️ | <brief> |

### Success-Rate Trend

<!-- Compare first-half vs second-half success rate. State delta and interpretation. -->

### Failure Patterns

| Rank | Job | Count | Type | Inferred Root Cause |
|------|-----|-------|------|---------------------|
| 1 | <job> | <n> | build / test / lint / deploy / timeout | <cause> |

### Duration Analysis

| Rank | Job | Avg | Max | Note |
|------|-----|-----|-----|------|
| 1 | <job> | <duration> | <duration> | bottleneck |

### Recommendations

1. 🔴 **Urgent** — <action>
2. 🟠 **High** — <action>
3. 🟡 **Medium** — <action>
4. 🟢 **Low** — <action>
```

## Quality Grades

| Success Rate | Grade |
|--------------|-------|
| ≥ 95% | 🟢 Healthy |
| 80%–94% | 🟡 Watch |
| < 80% | 🔴 Alert |

## Trend Arrows

| Condition | Indicator |
|-----------|-----------|
| Latter half higher | 📈 Improving |
| Latter half lower | 📉 Degrading |
| Delta < 5% | ➡️ Stable |

## Priority Levels

| Priority | Condition |
|----------|-----------|
| 🔴 Urgent | success rate < 80% or build持续性失败 |
| 🟠 High | 80%–94% or flaky test present |
| 🟡 Medium | duration ↑ > 20% or lint repeats |
| 🟢 Low | ≥ 95% but room to optimize |

## Failure Type Classification

| Type | Common Causes | Severity |
|------|---------------|----------|
| build | compile error, dependency breakage | 🔴 blocking |
| test | unit/integration failure, timeout | 🔴 blocking |
| lint | format, clippy warning | 🟡 non-blocking |
| deploy | env config, permission | 🔴 blocking |
| timeout | resource starvation, deadlock | 🟠 intermittent |

## Recurring Pattern Detection

- Same job fails ≥ 3 consecutive runs → persistent failure
- Same job fails intermittently → flaky / env instability
- Multiple jobs fail in same stage → shared root cause (e.g., dependency change)

## Duration Optimization Hints

| Symptom | Direction |
|---------|-----------|
| Single job slow | Split, parallelize |
| Dependency download slow | Cache dependency directory |
| Test suite slow | Split tests, skip slow ones |
| Build slow | Incremental compile, build cache |
