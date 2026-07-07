---
name: gitflow-security-check
description: |
  Use when the user requests a security audit, vulnerability scan, secret leak detection, or input validation review.
  当用户请求安全审计、漏洞扫描、密钥泄露检测或输入验证审查时使用。
---

# gitflow-security-check

## Overview

Scan-only audit covering deps, secret patterns, input validation. Reports only — no fix, no exfil.

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| security audit | 安全审计 | Full security review |
| vulnerability scan | 漏洞扫描 | `cargo audit`, dep safety |
| secret leak | 密钥泄露 | Hardcoded key worry |
| input validation | 输入验证 | Injection / SSRF / traversal |
| code style | 代码风格 | NOT this — redirect `gitflow-quality` |

## Core Pattern

```bash
cargo audit --version && cargo deny --version
cargo audit
cargo deny check
grep -rnE "(pwd|secret|token)\s*=\s*['\"]" --include="*.rs" src/
git ls-files | grep "\.env"
# spot input guards → report
```

## Quick Reference

| Goal | Command |
|------|---------|
| Vulns | `cargo audit` |
| License | `cargo deny check` |
| Secrets | `grep -rnE "(pwd\|secret\|token)\s*=\s*['\"]" --include="*.rs" src/` |
| Fix vuln | `cargo update -p <crate>` |

## Implementation

### Preconditions

`test -f Cargo.toml`; `cargo audit --version`; `cargo deny --version`; `git rev-parse --is-inside-work-tree`.

### Steps

1. `cargo audit` → parse CRITICAL / HIGH. db-fetch fail → `--no-fetch`, continue.
2. `cargo deny check licenses advisories` → log failures, continue.
3. grep + `git ls-files` — file + line only; **never log secret value**. Tracked `.env` = CRITICAL.
4. Spot auth / DB / file-op for length, `SafePath`, parameterized query, scheme allowlist — gaps = MEDIUM/LOW.
5. Emit three tables (dependency, secret, input-validation) + Summary. Sev: live secret/RCE = CRITICAL; missing public guard = HIGH; debug leak = MEDIUM.

### Error Handling

| Error | Recovery |
|-------|----------|
| db fetch fail | `--no-fetch`, note staleness |
| `cargo-deny` missing | Note N/A, continue |
| No grep match | Note "0 findings" |
| `cargo audit` non-zero | Report, do not improvise |

## Responsibility

### ✅ In Scope

Run `cargo audit`, `cargo deny`, grep; spot-check; emit report; recommend (text only).

### ❌ Out of Scope

Edit `audit.toml` / `deny.toml`; fix code (`gitflow-workflow`); commit/PR/Issue (`gitflow-{issue-create,pr-create}`); external send.

### 🚫 Do Not

- ❌ Auto-fix any vuln or missing guard
- ❌ Edit `audit.toml` / `deny.toml`
- ❌ Output secret values
- ❌ Upload results externally
- ❌ Create commits/PRs/Issues unless asked

## Rationalization Excuse Counter-Table

| Excuse | Reality |
|--------|---------|
| "Fix this one quick" | Scan-only; redirect `gitflow-workflow` |
| "Ignoring this CVE" | Never edit `audit.toml`; log |
| "Send to analyzer" | Data local; refuse |
| "Test code exempt" | Still log; user decides |
| "Skip cargo-deny" | Allow only if missing; note gap |

## Red Flags

- 🚩 "Auto-fix findings" — Refuse. Stop.
- 🚩 Live token found — CRITICAL; advise rotation; no paste.
- 🚩 "Ignore CVE-XXXX" — Refuse; log with note.
- 🚩 Report externally — Refuse.
- 🚩 DB corrupt → `--no-fetch`. No unofficial API.

## Test Scenarios

### 1: Happy Path

- **Given** Valid Rust project, tools installed
- **When** "security audit"
- **Then** All 5 steps run, report emitted, no out-of-scope command.

### 2: Negative

- **Given** "run the quality gate before ship"
- **When** No security/secret/vuln keyword
- **Then** Does NOT load. Redirects to `gitflow-quality`.

### 3: Boundary

- **Given** HIGH vuln in `libfoo v0.2`
- **When** "just `cargo update -p libfoo` for me"
- **Then** Refuses. Recommends action, asks confirm.

### 4: Error

- **Given** No `cargo-deny`
- **When** `cargo deny --version` fails
- **Then** Notes "N/A", continues.

## Success Criteria

- [ ] 3 tables + Summary
- [ ] No secret value in output
- [ ] No files modified, no commit/PR/Issue created
- [ ] Error Handling recovery used verbatim
- [ ] ≥1 cross-reference resolved for fix action

## Common Mistakes

- ❌ **Logging a secret** — Redact to `***`; log file + line only.
- ❌ **Silent skip when cargo-deny missing** — Note N/A.
- ❌ **Fixing a vuln after finding** — Report-only. Offer `gitflow-workflow`.
- ❌ **Exempting test-module hardcode** — Still log; user decides.

## Trigger Keywords

| English | 中文 |
|---------|------|
| security audit | 安全审计 |
| vulnerability scan | 漏洞扫描 |
| secret leak | 密钥泄露 |
| input validation | 输入验证 |

## See Also

- `gitflow-quality` — 6-gate post-fix quality check
- `gitflow-precommit` — commit-time gate overlapping scan
- `gitflow-autoreport-bug` — same report-only boundary
- `docs/superpowers/templates/skill-template.md` — template conformance
