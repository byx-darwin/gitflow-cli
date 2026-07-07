---
name: gitflow-security-check
description: |
  Use when the user wants to audit the codebase for hardcoded secrets, dependency vulnerabilities, unsafe code, or license compliance.
  当用户需要检查密钥硬编码、依赖漏洞、unsafe 代码、或许可证合规时使用。
---

# gitflow-security-check

Security audit checklist: dependency vulnerabilities, hardcoded secrets, unsafe code, license compliance. **Detection only — never auto-fix.**

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| security audit | 安全审计 | scan for vulnerabilities |
| dependency vulns | 依赖漏洞 | cargo audit |
| hardcoded secrets | 密钥硬编码 | grep patterns |
| license compliance | 许可证合规 | cargo deny |
| unsafe code check | unsafe 代码检查 | grep unsafe |

## Core Pattern

```bash
cargo audit                                              # 1. dependency vulns
cargo deny check                                         # 2. license compliance
grep -rn "password\|secret\|api_key\|token\s*=" src/     # 3. hardcoded secrets
grep -rn "unsafe" --include="*.rs" src/                  # 4. unsafe code
```

## Quick Reference

| Goal | Command |
|------|---------|
| Dependency audit | `cargo audit` |
| License check | `cargo deny check` |
| Find secrets | `grep -rn "password\|secret\|api_key" src/` |
| Find unsafe | `grep -rn "unsafe" --include="*.rs" src/` |

## Implementation

### Preconditions

- `cargo-audit` installed
- `cargo-deny` installed
- Advisory DB up to date

### Step 1: Run Scans

Execute the 4 scan commands from Core Pattern. Capture output.

### Step 2: Triage Findings

Classify by severity: `CRITICAL` > `HIGH` > `MEDIUM` > `LOW`.

### Step 3: Report

Produce a Security Audit Report with sections per scan type. Suggest fix commands (e.g., `cargo update -p <crate>`) — do NOT execute them.

### Error Handling

| Error | Recovery |
|-------|----------|
| `cargo-audit` not installed | Suggest `cargo install cargo-audit`. Do not improvise. |
| Advisory DB stale | `cargo audit` auto-updates. Persist otherwise. |
| `cargo deny` unavailable | Skip license section. Note in report. |
| No Rust src/ | Stop. Skill applicable to Rust only. |

## Responsibility

### ✅ In Scope

- Run `cargo audit`, `cargo deny`, pattern greps
- Produce Security Audit Report
- Suggest fix commands (in report, not executed)

### ❌ Out of Scope

- Auto-fixing vulnerabilities
- Modifying `audit.toml` ignore list
- Patching source code — `/gitflow-workflow`
- Reporting vulns to Issue — `/gitflow-autoreport-bug`

### 🚫 Do Not

- ❌ Run `cargo update` or apply patches
- ❌ Modify `.gitignore`, `audit.toml`, or source
- ❌ Report vulns to Issue without user confirmation
- ❌ Skip severity triage — CRITICAL must be called out

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Just patch it quickly" | Detection only; patching is out of scope. |
| "The test-only secret is harmless" | Flag it; let the user decide. |
| "Skip cargo deny if not installed" | Skip is OK — but note it in the report. |

## Red Flags

- 🚩 "Fix all the vulns now" — Refuse. Detection only.
- 🚩 "Add to audit.toml to silence" — Refuse. User decides.
- 🚩 "Ignore CRITICAL because it's transitive" — Refuse. Triage honestly.
- 🚩 "Report vulns to Issue automatically" — Refuse. Confirm with user first.

## Common Mistakes

- ❌ **Running `cargo update -p <crate>` to "help"** — never auto-fix.
- ❌ **Marking `MEDIUM` as "ok to skip"** — present findings neutrally.

## Trigger Keywords

| English | 中文 |
|---------|------|
| security audit | 安全审计 |
| cargo audit | 依赖漏洞 |
| hardcoded secrets | 密钥硬编码 |
| unsafe code | unsafe 代码 |
| license check | 许可证合规 |

## Test Scenarios

### 1: Happy Path
- **Given** Rust workspace with `cargo-audit` + `cargo-deny` — **When** "run security audit"
- **Then** 4 scans run → report produced → fix suggestions (not executed)

### 2: Negative
- **Given** "fix the unsafe code in src/foo.rs" — **When** user asks for fix
- **Then** skill NOT loaded — redirect `/gitflow-workflow`

### 3: Boundary
- **Given** CRITICAL vuln found — **When** "just patch it quickly"
- **Then** refuse; cite Out of Scope

### 4: Error
- **Given** `cargo-audit` not installed — **When** skill runs
- **Then** suggest install; do not improvise with raw `curl`/`wget`

## Success Criteria

- [ ] 4 scans attempted
- [ ] Findings classified by severity
- [ ] Fix commands suggested, not executed
- [ ] No source/config modifications

## See Also

- `/gitflow-quality` — 6-gate pre-delivery check
- `/gitflow-precommit` — pre-commit security hook
- `/gitflow-pipeline-analyzer` — CI/CD security gates
- `/gitflow-autoreport-bug` — file vuln as Issue
