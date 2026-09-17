# 覆盖率度量口径统一 — Design

- Date: 2026-09-17
- Workflow: `wf-2026-09-17-001`
- Issues: #340（工具分裂）、#348（references 三处缺陷）、#354（Gate 3 口径不一致）
- Milestone: #2「覆盖率度量口径统一」

## 背景

`gf-quality` skill 的覆盖率闸门存在三层互相纠缠的缺陷，三个 Issue 分别只看到了其中一面：

1. **工具分裂**（#340）：`skills/gf-quality/references/rust.md` 共 6 处指向 `cargo-tarpaulin`，而 `Makefile:158` 实际执行 `cargo llvm-cov`。按文档安装 tarpaulin 的人，跑 `make coverage` 用的是另一套工具。
2. **口径名实不符**（#354、#348 缺陷 3）：`SKILL.md:101` 把 Gate 3 定义为 `Incremental coverage ≥ 80%`，但五个语言层给出的命令**全部产出总覆盖率**，没有任何一处实现「与基线比较」。
3. **references 自身缺陷**（#348 缺陷 1、2）：`detector.md:26` 把 `Gemfile` 映射到不存在的 `references/ruby.md`；`go.md` 与 `python.md` 授权格式化工具自动修复，与 `SKILL.md:193`「Report only. No auto-fix.」直接对立（#348 认为该冲突仅在 `go.md`，实测不止，详见第 5 节）。

三者触碰同一批文件且互相约束——先定工具、再定口径、再同步语言层，顺序反了就要返工。故合并为单次变更处理。

## 调研结论（实测，非文档推断）

### 两个 Rust 覆盖率工具都没有原生增量模式

| 工具 | 版本 | 阈值能力 | 差异/增量模式 |
|---|---|---|---|
| `cargo-llvm-cov` | 0.9.0 | `--fail-under-lines` / `--fail-under-regions` / `--fail-under-functions` / `--fail-under-file-lines` | **无** |
| `cargo-tarpaulin` | 0.37.2 | `--fail-under` | **无** |

#354 猜测的 `cargo tarpaulin --workspace --diff <base>` **不存在**。真增量覆盖率必须导出 lcov/cobertura 再喂给 `diff-cover` 之类的外部工具，且五种语言各需配一套，并与 skill「不自动安装工具」的约束冲突（大量 SKIPPED）。故**放弃 incremental 语义**。

### tarpaulin 的 37.55% 是漏计产物

同一仓库、同为 `--workspace` 的对照实测：

| 工具 | 口径 | 覆盖率 | 分母（总行数） |
|---|---|---|---|
| tarpaulin（#354 引用值） | 行 | 37.55% | 6,634 |
| llvm-cov | **行** | **85.78%** | **21,717** |
| llvm-cov | 区域 | 85.49% | 31,322 |
| llvm-cov | 函数 | 81.13% | 2,983 |

分母相差 3.3 倍：tarpaulin 在 macOS / Apple Silicon 上大面积漏计（历史上依赖 ptrace，非 Linux 平台支持弱）。

**这推翻了 #354 的一条验收标准前提**：「若保留 80% 阈值而本仓库实测 37.55%，需给出处置方案（分批提升、按 crate 设阈值、或显式记录为已知缺口）」。换到 llvm-cov 后三个口径全部越过 80%，恒判负的闸门随工具统一一并消失，无需处置方案。关闭 #354 时需说明该条因前提证伪而不适用，而非声称已实现。

### CI 当前不采集覆盖率

`.github/workflows/` 全部 7 个文件（`ci.yml` / `cd.yml` / `e2e-tests.yml` / `release.yml` / `smoke-test.yml` / `upstream-patrol.yml` / `website.yml`）零命中 `coverage|codecov|llvm-cov|tarpaulin`。故 #340 验收标准末条「CI 若采集覆盖率，使用的工具与上述一致」以**空条件成立**判定——本次不新增 CI 覆盖率采集。

## 决策

| # | 决策 | 依据 |
|---|---|---|
| D1 | 覆盖率工具统一到 **cargo-llvm-cov** | Makefile 既有实现；LLVM source-based 为 Rust 工具链原生能力；macOS 上计量正确 |
| D2 | Gate 3 口径改为**总行覆盖率**，放弃 incremental | 两工具均无原生 diff；引入 diff-cover 会给五种语言各加一个跨生态外部依赖 |
| D3 | 阈值保持 **80%**，口径为**行覆盖** | 实测 85.78%，余量 5.8 点；行覆盖是五种语言工具链共有的概念，区域覆盖仅 LLVM 有 |
| D4 | 本次变更未触及该语言源码 → Gate 3 判 **N/A 跳过** | 消除 #329 那类纯文档变更靠人工豁免放行的情形；同时省掉一次全量插桩编译 |
| D5 | **补写** `references/ruby.md`，而非移除 detector 映射 | 保留 skill 对下游 Ruby 项目的支持能力 |
| D6 | tarpaulin 残留**按规范性/史料分类处理** | 改写已归档报告与历史计划会让记录与当时实际发生的事不符 |

## 变更设计

### 1. Rust Gate 3 命令

`skills/gf-quality/references/rust.md:11`：

```
| 3 | coverage | `cargo llvm-cov --workspace --fail-under-lines ${COV_THRESHOLD:-80}` | exit 0 |
```

阈值由工具原生强制（低于阈值退出码非 0），不再解析 `tail -3` 的文本输出。

### 2. 口径声明同步

`skills/gf-quality/SKILL.md:101`：

```
| 3 | **coverage** | Total line coverage ≥ 80% (`COV_THRESHOLD` overrides); N/A when the change touches no source file of that language |
```

五个语言层的 `incremental ≥ 80%` 一律改为总行覆盖，并各自用工具原生方式强制阈值：

| 文件 | 行 | 阈值强制方式 |
|---|---|---|
| `rust.md` | 11 | `--fail-under-lines ${COV_THRESHOLD:-80}` |
| `python.md` | 11 | `--cov-fail-under=${COV_THRESHOLD:-80}` |
| `node.md` | 26 / 37 | bun `--coverage-threshold` / jest `--coverageThreshold` / vitest `coverage.thresholds.lines` |
| `java.md` | 13 / 24 | JaCoCo `check` goal 的 `LINE` COVEREDRATIO rule |
| `go.md` | 11 | `go tool cover -func` 输出接 `awk` 与阈值比较（无原生支持） |
| `ruby.md` | 新建 | SimpleCov `minimum_coverage` |

同时删除 `go.md:27` 与 `:89` 已失效的 `compare against previous run`。

### 3. 空变更的 N/A 判定

Gate 3 执行前先取本次变更集，不含该语言源文件则判 N/A 并在报告中写明理由。Rust 判据：

```bash
git diff --name-only "$(git merge-base HEAD "${BASE_REF:-origin/main}")" | grep -qE '\.rs$'
```

各语言的扩展名判据分别为 `\.go$` / `\.(js|jsx|ts|tsx)$` / `\.py$` / `\.java$` / `\.rb$`。

N/A 在报告中与「工具缺失导致的 SKIPPED」区分表述，避免两种跳过混为一谈。

### 4. `references/ruby.md` 补写

按现有五份的结构写全 6 个 Gate：

| # | Gate | 命令 |
|---|---|---|
| 1 | build | `bundle install --quiet` |
| 2 | test | `bundle exec rspec` |
| 3 | coverage | SimpleCov `minimum_coverage ${COV_THRESHOLD:-80}` |
| 4 | format | `bundle exec rubocop --only Layout` |
| 5 | static | `bundle exec rubocop` |
| 6 | pre-commit | `pre-commit run --all-files`（无配置则 N/A） |

包含与其余五份一致的 Tool Installation、Environment Variables、Forbidden Actions（含 `❌ Never auto-fix with rubocop -a`）、Troubleshooting 段落。

### 5. auto-fix 冲突去除（范围大于 #348 所述）

#348 称该冲突「仅存在于 go.md」。实测不成立——`python.md` 有完全同形的表述，且三份 reference 的 Forbidden Actions 用了会反向授权的措辞：

| 文件:行 | 现有表述 | 问题 |
|---|---|---|
| `go.md:28` / `:90` | `Gate 4: auto-fix with gofmt -w . only after user confirmation` | #348 已指出 |
| `python.md:30` / `:96` | `Gate 4: auto-fix with ruff format . or black . only after user confirmation` | 同形冲突，#348 遗漏 |
| `go.md:34`、`python.md:36`、`node.md:66` | `Never auto-fix **without showing diff first**` | 暗示「出示 diff 即可自动修复」，与 `SKILL.md:193` 的绝对禁止冲突 |

#348 的验收标准是「**全部 6 个** reference 中不存在与 SKILL.md『Report only. No auto-fix.』冲突的表述」，故三份文件均需处理，只改 `go.md` 会使该条判负。

统一改为 `rust.md:34-35` 的句式：

```
- ❌ Never auto-fix with `<formatter>` — report only
```

`java.md` 与 `detector.md` 经核查无 auto-fix 表述，无需改动。

### 6. 新增 `scripts/validate-skill-links.sh`

扫描 `skills/` 下全部 Markdown，校验 skill 运行时**真正会加载的引用目标**，断链非零退出。

沿用 `verify-skills-when-not-to-use.sh` 的形态（`set -euo pipefail`、逐项输出、末尾汇总），接进 `Makefile` 的 `check-agent-sync` 目标——该目标已挂了两个同类脚本。

**不扩展 `validate-skill-commands.sh`**：后者的职责是校验 `gf` 子命令引用是否存在（需要 `gf` 二进制），与文件链接校验是两件事，合并会让它同时依赖二进制与文件系统。

#### 校验范围：只查加载目标，不查被提及的路径

朴素的「路径解析不到即判负」会误报。实测 `skills/` 下存在两类**并非断链**的路径提及：

| 路径 | 引用处 | 实质 |
|---|---|---|
| `docs/ONBOARDING.md` | `gf-repo-onboarding/SKILL.md:108`、`:131` | skill 可能**写出**的输出路径，出现在 Red Flag 与测试场景中 |
| `docs/agents/issue-tracker.md` | `gf-workflow/references.md:410` | mattpocock 路径的**条件前置**，原文紧接着就写了「Missing → ask the user: run `setup-mat-pocock-skills`」 |

故校验范围限定为两类导航性引用：

1. **Markdown 链接语法** `[text](path)` 的相对路径
2. **反引号包裹的 `references/*.md`** —— 渐进披露的加载目标

排除：绝对路径、`http(s)`/`mailto`/锚点、含 `<占位符>`、含 `*` 通配、含空格（命令行片段）。

#### 解析根：容器目录 + skill 根，任一命中即通过

`detector.md` 自身位于 `skills/gf-quality/references/`，但它引用的 `references/rust.md` 是相对 **skill 根**（`skills/gf-quality/`）解析的，而非相对自身目录。只按容器目录解析会产生 6 条误报。

故对每条引用依次尝试：容器文件所在目录、skill 根目录（`skills/<skill-name>/`），任一存在即通过。

#### 实测结果：恰好命中 3 处真断链，零误报

| 文件:行 | 引用 | 解析到 | 真实位置 |
|---|---|---|---|
| `gf-quality/references/detector.md:26` | `references/ruby.md` | 不存在 | —（本次补写） |
| `gf-label-stats/SKILL.md:12` | `../references/gf-label-stats-taxonomy.md` | `skills/references/…` | **`docs/references/…`** |
| `gf-pr-review/SKILL.md:65`、`:79`、`:83` | `../references/pr-review-checklist.md` | `skills/references/…` | **`docs/references/…`** |

后两条是 **#348 未发现的同类缺陷**：`../` 少了一级，从 `skills/<skill>/` 出发到达 `docs/references/` 需要 `../../docs/references/`。本次一并修复——否则脚本接进 `check-agent-sync` 会立即把该目标打红。

这也构成新脚本价值的即时验证：上线即多揪出 2 处 #348 漏掉的断链。

### 7. tarpaulin 残留的分类处理

| 类别 | 文件 | 处数 | 处理 |
|---|---|---|---|
| **规范性**（现行生效） | `skills/gf-quality/references/rust.md` | 6 | 改 |
| | `docs/integration-guide.md:100` | 1 | 改 |
| | `docs/references/gf-quality-params.md:161` | 1 | 改 |
| | `docs/superpowers/tests/skills/gf-quality-test.md` | 7 | 改 |
| **史料**（已发生的记录） | `docs/reports-archive/2026-Q3/code-review-report-pr105-2026-07-31.md` | 1 | 不改 |
| | `docs/superpowers/plans/`（2 个文件） | 3 | 不改 |
| | `docs/superpowers/specs/2026-07-06-skill-constraints-sync-design.md` | 1 | 不改 |
| | `docs/research/`（2 个文件） | 6 | 不改 |
| | `docs/issue-triage-report-2026-09-16.md` | 2 | 不改 |

#340 的验收标准「仓库内只存在一套覆盖率工具」按**现行生效**口径判定。

顺带修正 `docs/integration-guide.md:100` 所在的框：它写「5 项检查」但 `SKILL.md` 定义了 6 个 Gate（漏列 pre-commit）。

## 验证

| 项 | 方式 |
|---|---|
| 新脚本 | TDD：先造断链夹具使其红，再实现使其绿 |
| 链接完整性 | `make check-agent-sync` 绿（含新脚本） |
| Rust Gate 3 | `cargo llvm-cov --workspace --fail-under-lines 80` 退出码 0 |
| N/A 行为 | 纯文档变更场景下 Gate 3 判 N/A 而非失败 |
| 口径一致性 | 六份 reference 中不再出现 `incremental`；不再出现与「Report only」冲突的表述 |
| 工具唯一性 | 规范性文件中 `grep -c tarpaulin` 为 0 |

## 范围外的发现

`crates/gitlab/src/auth.rs:347` 的 `temp_env::with_var("GL_TOKEN", Some("test_token"), …)` 修改的是**进程级**环境变量。同进程内并行运行的其他 async 测试会读到它，导致 `token()` 取到 `test_token` 而非用例期望值。

实测 `cargo test -p gitflow-gitlab --lib` 在 `dev` 分支稳定失败，失败条数随调度在 3～4 之间抖动：

```
thread 'auth::tests::test_should_extract_token_from_auth_status_show_token' panicked:
  left: "test_token"
 right: "glpat-abcdef"
```

这是既有缺陷，与本 milestone 无关，**不在本次变更范围内**。Phase 4 单独提 Issue。

同类环境噪声还有 `crates/e2e-gitcode/tests/noauth.rs` 的恒红（已知：Graphviz 的 `gc` 与 GitCode CLI 撞名）。两者叠加导致本次基线测量需 `--ignore-run-fail` 才能取得。
