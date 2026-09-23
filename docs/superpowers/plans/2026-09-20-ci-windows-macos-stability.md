# CI Windows/macOS Test Job Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 给 `.github/workflows/ci.yml` 的 `test` job 加 job 级 timeout 与失败时诊断信息上传，为 #373 排查结论中"没有可复现路径的孤立 flake"留下未来的诊断证据。

**Architecture:** 纯 CI YAML 配置改动，单文件两处修改：`timeout-minutes` 声明 + 一个 `if: failure()` 诊断步骤。不涉及 Rust 代码，不涉及单元测试。

**Tech Stack:** GitHub Actions YAML

**Spec:** `docs/superpowers/specs/2026-09-20-ci-windows-macos-stability-design.md`

## Global Constraints

- 不修改 `test` job 矩阵本身（`os: [ubuntu-latest, macos-latest, windows-latest]`）或既有的 `cargo test`/`cargo nextest run` 步骤
- `actions/upload-artifact` 使用仓库既有约定版本 `@v4`（`.github/workflows/e2e-tests.yml`、`release.yml` 已用此版本）
- 不追加超出设计文档范围的改动（不碰 `gf pipeline report` 本身——那是 Issue #378 的范围）

---

### Task 1: 给 `test` job 加 timeout 与失败诊断上传

**Files:**
- Modify: `.github/workflows/ci.yml:35-63`（`test` job）

**Interfaces:**
- 无跨任务接口——单文件单 job 的自包含改动

- [ ] **Step 1: 确认当前 `test` job 的确切内容（作为改动基线）**

Run: `sed -n '35,63p' .github/workflows/ci.yml`

Expected 输出应与下方"改动前"完全一致：

```yaml
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest
      - name: Build release binary (for e2e tests)
        run: cargo build --release
      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH
      # e2e-gitlab/e2e-gitcode need the real `glab`/`gc` CLI binaries installed to
      # exercise their noauth.rs error-path assertions correctly (see
      # .github/workflows/e2e-tests.yml's dedicated jobs, which install them).
      # This generic 3-OS matrix job doesn't install them, so exclude those two
      # crates here — e2e-github stays included since `gh` ships preinstalled on
      # all three GitHub-hosted runner images.
      - name: cargo test
        run: cargo nextest run --all-features --workspace --exclude e2e-gitlab --exclude e2e-gitcode
      # nextest does not run doctests at all, so without this step the `# Examples`
      # blocks in public API docs are compiled and executed by nothing in CI.
      # Same crate exclusions as above, for the same reason.
      - name: cargo test --doc
        run: cargo test --doc --all-features --workspace --exclude e2e-gitlab --exclude e2e-gitcode
```

若基线不一致（文件在本计划写完后被其他改动改过），先跟维护者确认再继续，不要盲目套用下面的替换。

- [ ] **Step 2: 应用改动——加 `timeout-minutes` + 失败诊断步骤**

把上面那段整体替换为：

```yaml
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest
      - name: Build release binary (for e2e tests)
        run: cargo build --release
      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH
      # e2e-gitlab/e2e-gitcode need the real `glab`/`gc` CLI binaries installed to
      # exercise their noauth.rs error-path assertions correctly (see
      # .github/workflows/e2e-tests.yml's dedicated jobs, which install them).
      # This generic 3-OS matrix job doesn't install them, so exclude those two
      # crates here — e2e-github stays included since `gh` ships preinstalled on
      # all three GitHub-hosted runner images.
      - name: cargo test
        run: cargo nextest run --all-features --workspace --exclude e2e-gitlab --exclude e2e-gitcode
      # nextest does not run doctests at all, so without this step the `# Examples`
      # blocks in public API docs are compiled and executed by nothing in CI.
      # Same crate exclusions as above, for the same reason.
      - name: cargo test --doc
        run: cargo test --doc --all-features --workspace --exclude e2e-gitlab --exclude e2e-gitcode
      # Issue #373: 唯一一次真实的 Windows 独立失败（2026-08-31）没有可复现路径，
      # 也没有留下能区分"资源限制 vs 测试本身"的证据。这一步只在失败时跑，采集
      # 当时的资源状态，供下次万一复现时判别根因。
      - name: Collect failure diagnostics
        if: failure()
        shell: bash
        run: |
          {
            echo "## Failure diagnostics (${{ matrix.os }})"
            echo
            echo "### Toolchain"
            rustc --version
            cargo --version
            echo
            echo "### Disk space"
            df -h . || true
            echo
            echo "### Memory"
            if [ "${{ runner.os }}" = "Linux" ]; then
              free -h || true
            elif [ "${{ runner.os }}" = "macOS" ]; then
              vm_stat || true
            else
              wmic OS get FreePhysicalMemory,TotalVisibleMemorySize 2>/dev/null || \
                systeminfo | grep -i "memory" || true
            fi
          } > failure-diagnostics.txt
      - name: Upload failure diagnostics
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: test-diagnostics-${{ matrix.os }}
          path: failure-diagnostics.txt
          retention-days: 7
```

（改动点：第 3 行新增 `timeout-minutes: 15`；`steps` 列表末尾新增两个 `if: failure()` 步骤。既有步骤内容与顺序一字不改。）

- [ ] **Step 3: YAML 语法检查**

Run: `pre-commit run check-yaml --files .github/workflows/ci.yml`
Expected: `Passed`

若本地没有安装 `pre-commit` 或该 hook 不可用，退而求其次用 Python 校验语法：

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo "YAML syntax OK"`
Expected: `YAML syntax OK`

- [ ] **Step 4: 人工核对 diff 范围**

Run: `git diff .github/workflows/ci.yml`
Expected: diff 只涉及 `test` job 这一段，新增 1 行 `timeout-minutes: 15` + 2 个新 step（约 25 行新增），没有改动其他 job（`check`/`lint`/`msrv`/`smoke-test` 等）或既有 `test` job 的任何原有内容。

- [ ] **Step 5: 提交**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add timeout and failure diagnostics to Test job matrix (#373)"
```

---

## Final Verification

- [ ] **Step 1: 全量 pre-commit（含既有 hook 全跑一遍，确认没有引入其他问题）**

Run: `pre-commit run --all-files`
Expected: 全部 `Passed`（`cargo fmt`/`cargo check` 因本次未改 Rust 代码应为 `Skipped`，`check yaml` 应为 `Passed`）
