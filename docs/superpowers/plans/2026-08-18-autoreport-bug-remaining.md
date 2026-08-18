# 主动上报 bug 功能遗留问题修复实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 解决「主动上报 bug」功能 6 项遗留问题：去重一致性、处理端日志、多报告归档、公开预览、auth_cache_ttl 实现、残留清理。

**Architecture:** 6 个独立小任务。T1（文档对齐）+ T6（清理）为轻量；T3（Rust 归档）走 TDD；T2/T4/T5（hook + skill）为脚本/文档改动。上报路径保持 `gh` CLI（边界已定）。

**Tech Stack:** Rust 2024 · bash（Stop Hook）· Claude Code skill · bats 测试

**Spec:** `docs/superpowers/specs/2026-08-18-autoreport-bug-remaining-design.md`（随计划传递）

## Global Constraints

- **上报路径用 `gh` CLI**（项目在 GitHub、上报独立于 gf）；被上报对象保留 `gf`（标题前缀/pending command）。
- skill 源在 `skills/gf-autoreport-bug/SKILL.md`；`.claude/` 副本必须同步。
- 所有 Rust 变更须过 `make test` + `cargo clippy --all-targets --all-features -- -D warnings`。
- `write_to_disk` 保持 `pub(crate)` 签名不变；归档逻辑在函数内部。
- 不实施完整队列/多消费者（覆盖前归档足够）。

---

### Task 1: P1-2 去重命令一致性

**Files:**
- Modify: `skills/gf-autoreport-bug/SKILL.md`（Workflow Step 3 加 `--state all`）
- Modify: `.claude/skills/gf-autoreport-bug/SKILL.md`（同步副本）
- Modify: `docs/references/gf-autoreport-bug-params.md`（确认一致）

**Interfaces:**
- Consumes: 无（独立）
- Produces: 去重命令统一为含 `--state all`

- [ ] **Step 1: 修改 skill Workflow Step 3**

编辑 `skills/gf-autoreport-bug/SKILL.md` Workflow Step 3，将：

```markdown
3. **Dedup** — `gh issue list --repo byx-darwin/gitflow-cli --search "[auto-report] {command} {error_code}"`. Match → clean, stop.
```

改为：

```markdown
3. **Dedup** — `gh issue list --repo byx-darwin/gitflow-cli --search "[auto-report] {command} {error_code}" --state all`. Match → clean, stop.
```

- [ ] **Step 2: 同步副本**

```bash
cp skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
```

- [ ] **Step 3: 验证一致性**

```bash
grep -n "gh issue list" skills/gf-autoreport-bug/SKILL.md docs/references/gf-autoreport-bug-params.md
# 两处均含 --state all
diff skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md && echo "✅ 副本一致"
```

- [ ] **Step 4: 提交**

```bash
git add skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
git commit -m "fix(skill): unify dedup command with --state all (P1-2)"
```

---

### Task 2: P1-3 处理端日志

**Files:**
- Modify: `hooks/auto-report-bug.sh`（hook.log）
- Modify: `skills/gf-autoreport-bug/SKILL.md`（processing.log）
- Modify: `.claude/skills/gf-autoreport-bug/SKILL.md`（同步）

**Interfaces:**
- Consumes: `$PENDING_FILE`（已有）、`$REPO_ROOT`（已有）
- Produces: `.cache/bug-reports/hook.log` + `.cache/bug-reports/processing.log`

- [ ] **Step 1: hook 写日志**

在 `hooks/auto-report-bug.sh` 中，检测到 pending.json 后（行 33 之后）插入：

```bash
# Append to hook.log for observability
HOOK_LOG="$REPO_ROOT/.cache/bug-reports/hook.log"
mkdir -p "$(dirname "$HOOK_LOG")"
log_hook() {
  echo "[$(date +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || python3 -c 'import time;print(time.strftime("%Y-%m-%dT%H:%M:%SZ",time.gmtime()))')] $1" >> "$HOOK_LOG"
}
log_hook "detect pending.json (command=${COMMAND:-unknown}, platform=${PLATFORM:-unknown})"
```

在认证失败分支（`AUTH_CHECK_FAILED=true`）后加：

```bash
    log_hook "auth failed (platform=${PLATFORM:-unknown})"
```

在 banner 输出前（行 118 附近）加：

```bash
log_hook "banner emitted (command=${COMMAND:-unknown})"
```

- [ ] **Step 2: skill 写 processing.log**

在 `skills/gf-autoreport-bug/SKILL.md` Workflow Step 5（Notify）后、Step 6（Cleanup）前插入：

```markdown
5. **Notify** — Output `✅ 已自动报告 bug: {issue_url}`.
5b. **Log** — Append `[timestamp] issue created: {issue_url}` to `.cache/bug-reports/processing.log`.
```

- [ ] **Step 3: 验证（手动模拟 hook）**

```bash
# 触发 hook，检查 hook.log 存在且含时间戳
bash hooks/auto-report-bug.sh </dev/null 2>&1 >/dev/null
test -f .cache/bug-reports/hook.log && grep -q "detect pending.json" .cache/bug-reports/hook.log && echo "✅ hook.log 记录"
```

- [ ] **Step 4: 同步 skill 副本 + 提交**

```bash
cp skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
git add hooks/auto-report-bug.sh skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
git commit -m "feat(report): add hook.log + processing.log observability (P1-3)"
```

---

### Task 3: P1-5 多报告（覆盖前归档，TDD）

**Files:**
- Modify: `apps/cli/src/error_reporter.rs`（`write_to_disk` 归档旧文件）
- Test: `apps/cli/src/error_reporter.rs` 测试模块

**Interfaces:**
- Consumes: 无（独立）
- Produces: 二次写入保留旧报告为 `pending.<毫秒时间戳>.json`

- [ ] **Step 1: 写失败测试**

在 `apps/cli/src/error_reporter.rs` 测试模块追加：

```rust
#[test]
fn test_should_archive_previous_pending_on_second_write() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let first = ErrorReport::from_error("issue list", "github", "first", "CLI_ERROR");
    first.write_to_disk(tmp.path()).expect("first write");

    let second = ErrorReport::from_error("pr list", "github", "second", "CLI_ERROR");
    second.write_to_disk(tmp.path()).expect("second write");

    let dir = tmp.path().join(".cache/bug-reports");
    let pending = dir.join("pending.json");
    assert!(pending.exists(), "pending.json must exist");

    // Old report must be preserved under a timestamped name.
    let archived: Vec<_> = std::fs::read_dir(&dir)
        .expect("read bug-reports dir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("pending.") && n.ends_with(".json"))
        .collect();
    assert_eq!(archived.len(), 1, "exactly one archived report: {archived:?}");

    let archived_content =
        std::fs::read_to_string(dir.join(&archived[0])).expect("read archived");
    assert!(archived_content.contains("first"), "archived report keeps first content");
}
```

- [ ] **Step 2: 运行测试验证失败**

```bash
cargo test -p gitflow-cli --bin gf error_reporter::tests::test_should_archive_previous_pending_on_second_write 2>&1 | tail -8
# 期望：FAIL（当前实现覆盖写，无归档文件）
```

- [ ] **Step 3: 实现归档逻辑**

编辑 `apps/cli/src/error_reporter.rs::write_to_disk`，在 `let path = dir.join("pending.json");` 之后、`serde_json::to_string_pretty` 之前插入：

```rust
        // Preserve any existing pending report so a burst of failures does
        // not silently drop earlier reports (P1-5).
        if path.exists() {
            let archived = dir.join(format!(
                "pending.{}.json",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis())
            ));
            std::fs::rename(&path, &archived)?;
        }
```

- [ ] **Step 4: 运行测试验证通过**

```bash
cargo test -p gitflow-cli --bin gf error_reporter 2>&1 | tail -5
# 期望：全部通过（含新归档测试）
```

- [ ] **Step 5: 运行完整验证**

```bash
make test
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] **Step 6: 提交**

```bash
git add apps/cli/src/error_reporter.rs
git commit -m "fix(report): archive previous pending.json before overwrite (P1-5)"
```

---

### Task 4: P2-3 公开预览（skill 交互）

**Files:**
- Modify: `skills/gf-autoreport-bug/SKILL.md`（Workflow Step 4 前插入预览）
- Modify: `.claude/skills/gf-autoreport-bug/SKILL.md`（同步）

**Interfaces:**
- Consumes: 无（独立）
- Produces: skill 创建前打印草案 + 用户选择 create/skip/modify

- [ ] **Step 1: 插入预览步骤**

在 `skills/gf-autoreport-bug/SKILL.md` Workflow Step 4（Create）之前插入：

```markdown
3b. **Preview** — Print the sanitized pending summary (command/platform/error_code/error_message) and the planned Issue title + body. Ask the user: `create / skip / modify`. Non-interactive default: create.
```

同时更新 Step 4 文案，明确「用户选择 create 后执行」：

```markdown
4. **Create** — After user confirms (or non-interactive default), analyze root cause + severity, then `gh issue create --repo byx-darwin/gitflow-cli --title "[auto-report] gf {command} — {error_code}" --label "auto-report"`. Fail → keep file + `failed.log`.
```

- [ ] **Step 2: 同步副本**

```bash
cp skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
```

- [ ] **Step 3: 验证词数 <500 且副本一致**

```bash
wc -w skills/gf-autoreport-bug/SKILL.md   # 期望 <500（超则精简）
diff skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
```

- [ ] **Step 4: 提交**

```bash
git add skills/gf-autoreport-bug/SKILL.md .claude/skills/gf-autoreport-bug/SKILL.md
git commit -m "feat(skill): preview pending + user choice before create (P2-3)"
```

---

### Task 5: B1 auth_cache_ttl 实现

**Files:**
- Modify: `hooks/auto-report-bug.sh`（读取 auth_cache_ttl）

**Interfaces:**
- Consumes: `$PENDING_CONTENT`（已有）
- Produces: `AUTH_CACHE_TTL` 由 pending.json 覆盖（缺省 86400）

- [ ] **Step 1: 读取 auth_cache_ttl**

编辑 `hooks/auto-report-bug.sh`，在 `AUTH_CACHE_TTL=86400` 之前插入：

```bash
# Optional per-report TTL override from pending.json (B1).
PENDING_TTL=$(echo "$PENDING_CONTENT" | grep -o '"auth_cache_ttl"[[:space:]]*:[[:space:]]*[0-9]*' | head -1 | sed 's/.*:[[:space:]]*//')
AUTH_CACHE_TTL=${PENDING_TTL:-86400}
```

（原 `AUTH_CACHE_TTL=86400` 行移除，由上面计算。）

- [ ] **Step 2: 验证**

```bash
# 无 auth_cache_ttl → 缺省 86400
grep -n "AUTH_CACHE_TTL=" hooks/auto-report-bug.sh
bash -n hooks/auto-report-bug.sh && echo "✅ 语法正确"
```

手动模拟：pending.json 含 `auth_cache_ttl: 3600` → 验证 `AUTH_CACHE_TTL=3600` 生效（检查 cache 过期判定边界）。

- [ ] **Step 3: 提交**

```bash
git add hooks/auto-report-bug.sh
git commit -m "fix(hook): honor auth_cache_ttl override from pending.json (B1)"
```

---

### Task 6: B2 残留清理

**Files:**
- Delete: `.cache/bug-reports/pending.json`（测试残留）

**Interfaces:**
- Consumes: 无
- Produces: 目录干净（保留日志文件）

- [ ] **Step 1: 确认并清理**

```bash
# 确认是残留（gh 已认证，11:24Z auth 错误为测试产物）
gh auth status 2>&1 | grep -q "Logged in" && echo "gh 已认证，确认残留"
rm -f .cache/bug-reports/pending.json
ls .cache/bug-reports/
```

- [ ] **Step 2: 无提交（缓存文件 gitignored）**

```bash
# 无需 git 提交；验证目录状态
git status --short | grep bug-reports || echo "✅ 缓存目录不影响 git"
```

---

## Self-Review（写后自检）

**Spec 覆盖：**
- T1 → §3.1 P1-2 去重一致性 ✅
- T2 → §3.2 P1-3 hook.log + processing.log ✅
- T3 → §3.3 P1-5 覆盖前归档（TDD）✅
- T4 → §3.4 P2-3 预览 ✅
- T5 → §3.5 B1 auth_cache_ttl ✅
- T6 → §3.6 B2 残留清理 ✅
- AC1→T1S3 · AC2→T2S3 · AC3→T3S4 · AC4→T4S3 · AC5→T5S2 · AC6→T6S1 · AC7→各任务验证 ✅

**占位符扫描：** 无 TBD/TODO；所有代码块含完整内容。

**类型一致性：** `write_to_disk` 签名不变（T3）；`$PENDING_CONTENT`/`$PENDING_FILE`/`$REPO_ROOT`/`$PLATFORM` 均在 hook 已有定义（T2/T5）；skill 步骤编号 3b/5b 为新增，不与现有 1-6 冲突（T2/T4）。

**测试注意：** T3 归档测试用 `tempfile` 目录隔离；时间戳用毫秒避免同毫秒重名冲突。T4 词数上限风险：插入预览 + Create 文案扩展可能超 500，Step 3 检查超则精简 Red Flags/Rationalization。
