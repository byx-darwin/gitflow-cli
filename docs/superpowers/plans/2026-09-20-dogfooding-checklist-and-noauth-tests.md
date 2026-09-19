# Dogfooding Checklist Fix + Noauth Test Env Clarity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 dogfooding checklist 里对不上的 CLI 参数（#361），并让 `e2e-github`/`e2e-gitlab` 的 noauth 测试在 `gh`/`glab` 未安装时优雅 skip 而不是产生误导性失败（#356）。

**Architecture:** Task 1 是纯文档改动（`docs/specs/phase4-dogfooding-checklist.md`）。Task 2 是两个独立 crate 里各自的 `tests/noauth.rs` 文件（`e2e-github`、`e2e-gitlab`），两条测试各加一段 skip 逻辑，互不依赖。

**Tech Stack:** Markdown / Rust 2024（tokio async test）

**Spec:** `docs/superpowers/specs/2026-09-20-dogfooding-checklist-and-noauth-tests-design.md`

## Global Constraints

- skip 判定复用 `gf` 实际输出里"未检测到 {binary}"这个安装引导文案（`apps/cli/src/commands/prerequisites.rs:186` 的格式），不用 `which` 单独探测 PATH
- `crates/e2e-gitcode/tests/noauth.rs` 已经修复过，不在本次范围内，不要动
- `crates/e2e-gitlab/tests/noauth.rs` 的模块文档已经准确，不要改文案，只加运行时 skip 逻辑

---

### Task 1: 修正 dogfooding checklist 命令参数（#361）

**Files:**
- Modify: `docs/specs/phase4-dogfooding-checklist.md`

**Interfaces:**
- 无跨任务接口

- [ ] **Step 1: 改头部日期/版本**

把：

```markdown
**适用版本:** v0.6.x+
**最后更新:** 2026-07-10
```

替换为（用 `gf --version` 实测填入当前版本号）：

```markdown
**适用版本:** v0.6.x+
**最后更新:** 2026-09-20（本次已逐条实测下方命令，验证所用 CLI 版本见 `gf --version`）
```

- [ ] **Step 2: 改 release create 命令**

把：

```markdown
- [ ] 创建 release：`gf release create v0.x.x --notes "test release"`
```

替换为：

```markdown
- [ ] 创建 release：`gf release create --tag-name v0.x.x --body "test release"`
```

把：

```markdown
- [ ] 非交互模式验证：`echo "y" | gf release create v0.x.x --notes "test"`
```

替换为：

```markdown
- [ ] 非交互模式验证：`echo "y" | gf release create --tag-name v0.x.x --body "test"`
```

- [ ] **Step 3: 改 issue create 命令**

把：

```markdown
- [ ] 创建带中文标签的 issue：`gf issue create --title "Dogfooding test" --labels "测试标签"`
```

替换为：

```markdown
- [ ] 创建带中文标签的 issue：`gf issue create --title "Dogfooding test" --label "测试标签"`
```

- [ ] **Step 4: 补防漂移说明**

在文档头部（"适用版本"/"最后更新"那两行）之后，新增一段：

```markdown
> **命令随 CLI 版本升级可能漂移**：本清单里的每条命令在"最后更新"日期当天都用
> `gf <子命令> --help` 逐一核对过参数形式。CLI 版本升级后如果某条命令报参数错误，
> 先用 `--help` 核对最新参数形式再更新本清单，而不是照抄旧命令排查半天。
```

- [ ] **Step 5: 人工验证新命令可执行**

Run: `./target/debug/gf release create --help 2>&1 | grep -E "tag-name|--body"`
Expected: 输出包含 `--tag-name <TAG_NAME>` 和 `--body <BODY>`，确认无 `--notes` 项

Run: `./target/debug/gf issue create --help 2>&1 | grep -E "label|title"`
Expected: 输出包含 `--label <LABEL>`（单数），确认无 `--labels`

- [ ] **Step 6: 提交**

```bash
git add docs/specs/phase4-dogfooding-checklist.md
git commit -m "fix(docs): correct dogfooding checklist command params to match actual CLI (#361)"
```

---

### Task 2: noauth 测试加 skip 逻辑（#356）

**Files:**
- Modify: `crates/e2e-github/tests/noauth.rs`
- Modify: `crates/e2e-gitlab/tests/noauth.rs`

**Interfaces:**
- 无跨任务接口，两个文件各自独立

- [ ] **Step 1: RED — 用受限 PATH 复现当前的误导性失败**

Run:
```bash
MIRROR=$(mktemp -d)
for f in /opt/homebrew/bin/* /usr/bin/* /bin/*; do
  [ -e "$f" ] || continue
  b=$(basename "$f")
  [ "$b" = "gh" ] && continue
  [ -e "$MIRROR/$b" ] && continue
  ln -s "$f" "$MIRROR/$b" 2>/dev/null || true
done
env PATH="$MIRROR:$HOME/.cargo/bin" cargo test -p e2e-github --test noauth 2>&1 | tail -20
rm -rf "$MIRROR"
```
Expected: 至少一条测试 FAILED，输出里能看到 `expected login guidance in output, got:` 后面跟的是"未检测到 gh"的安装引导文案而不是登录引导——这就是 #356 描述的误导性失败，实测复现

- [ ] **Step 2: 改 `crates/e2e-github/tests/noauth.rs` 模块文档**

把第 1 行：

```rust
//! 未认证错误路径 E2E 测试(无需凭据,任何环境均可运行)
```

替换为：

```rust
//! 未认证错误路径 E2E 测试(无需凭据,前提是运行环境已安装 `gh` CLI；未安装时 skip
//! 而非 fail，见下方 skip 逻辑)
```

- [ ] **Step 3: 改 `crates/e2e-github/tests/noauth.rs` 两条测试，加 skip 逻辑**

把：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

替换为：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    let combined = format!("{}{}", output.stdout, output.stderr);
    if combined.contains("未检测到 gh") {
        eprintln!("skipped: gh CLI not installed in this environment");
        return;
    }

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero"
    );
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

把：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero"
    );
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

替换为：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "github", "--output", "json"])
        .await
        .unwrap();

    let combined = format!("{}{}", output.stdout, output.stderr);
    if combined.contains("未检测到 gh") {
        eprintln!("skipped: gh CLI not installed in this environment");
        return;
    }

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero"
    );
    assert!(
        combined.contains("gh auth login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

- [ ] **Step 4: 改 `crates/e2e-gitlab/tests/noauth.rs` 两条测试，加同款 skip 逻辑（不改模块文档）**

把：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

替换为：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_status_checked_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["auth", "status", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    let combined_raw = format!("{}{}", output.stdout, output.stderr);
    if combined_raw.contains("未检测到 glab") {
        eprintln!("skipped: glab CLI not installed in this environment");
        return;
    }

    assert!(
        !output.status.success(),
        "unauthenticated auth status must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = combined_raw.to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

把：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = format!("{}{}", output.stdout, output.stderr).to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

替换为：

```rust
#[tokio::test]
async fn test_should_fail_with_login_guidance_when_listing_issues_unauthenticated() {
    let runner = scrubbed_runner();
    let output = runner
        .run(&["issue", "list", "--platform", "gitlab", "--output", "json"])
        .await
        .unwrap();

    let combined_raw = format!("{}{}", output.stdout, output.stderr);
    if combined_raw.contains("未检测到 glab") {
        eprintln!("skipped: glab CLI not installed in this environment");
        return;
    }

    assert!(
        !output.status.success(),
        "unauthenticated issue list must exit non-zero, stdout: {}, stderr: {}",
        output.stdout,
        output.stderr
    );
    let combined = combined_raw.to_lowercase();
    assert!(
        combined.contains("auth login") || combined.contains("login"),
        "expected login guidance in output, got: {combined}"
    );
}
```

- [ ] **Step 5: GREEN — 正常路径不受影响**

Run: `cargo test -p e2e-github -p e2e-gitlab --test noauth 2>&1 | tail -15`
Expected: 全部 4 条测试 PASS（本机已装 `gh`/`glab`，走原有断言路径，不会误 skip）

- [ ] **Step 6: GREEN — 受限 PATH 下验证新 skip 逻辑生效**

Run:
```bash
MIRROR=$(mktemp -d)
for f in /opt/homebrew/bin/* /usr/bin/* /bin/*; do
  [ -e "$f" ] || continue
  b=$(basename "$f")
  [ "$b" = "gh" ] && continue
  [ -e "$MIRROR/$b" ] && continue
  ln -s "$f" "$MIRROR/$b" 2>/dev/null || true
done
env PATH="$MIRROR:$HOME/.cargo/bin" cargo test -p e2e-github --test noauth -- --nocapture 2>&1 | tail -20
rm -rf "$MIRROR"
```
Expected: 测试 PASS（不再 FAILED），`--nocapture` 输出里能看到 `skipped: gh CLI not installed in this environment`

- [ ] **Step 7: 提交**

```bash
git add crates/e2e-github/tests/noauth.rs crates/e2e-gitlab/tests/noauth.rs
git commit -m "fix(e2e): noauth tests skip gracefully when gh/glab CLI not installed instead of failing misleadingly (#356)"
```

---

## Final Verification

- [ ] **Step 1: 全量测试**

Run: `cargo test -p e2e-github -p e2e-gitlab --test noauth`
Expected: 全部 PASS

- [ ] **Step 2: pedantic clippy（改动的两个 crate）**

Run: `cargo clippy -p e2e-github -p e2e-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 0 警告

- [ ] **Step 3: pre-commit 全量**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed`
