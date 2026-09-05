# Pipeline 分析报告 — PR #304

> **PR：** [#304 test(e2e): add e2e-gitlab and e2e-gitcode coverage](https://github.com/byx-darwin/gitflow-cli/pull/304)
> **分支：** `feat/291-e2e-gitlab-gitcode-coverage` → `dev`（对应 Issue #291）
> **分析日期：** 2026-09-03
> **模式：** 只读（CLI: `gf`，版本 `1.9.0`）
> **背景：** 任务下发时描述为"刚 push 并通过 `gf pr merge 304 --auto` 加入合并队列"。实测 `gf pr view 304` 显示 `state: closed`、`mergedAt: 2026-09-03T02:20:21Z`——PR 在下发分析任务前已完成合并。本报告基于合并前该分支触发的 3 个 workflow run 的**最终终态**撰写（采集时其中 2 个仍 `running`，已轮询至全部收尾）。

## 零、核心结论先行

PR #304（新增 `crates/e2e-gitlab`、`crates/e2e-gitcode`，并在 `.github/workflows/e2e-tests.yml` 新增 `e2e-gitlab`/`e2e-gitcode` 两个 job）触发的 3 个 workflow run 中，**2 个失败、仅 1 个成功**：`gf pipeline report` 最终收敛为 `successRate: 0.333`、`avgDurationSecs: 193.67s`。13 个 job 中 **5 个失败**：主 CI workflow 的 `Test (ubuntu/macos/windows-latest)` 三平台均失败（各 2 个测试失败，位于新增的 `crates/e2e-gitcode/tests/noauth.rs`），E2E Tests workflow 新增的 `e2e-gitlab` job 失败（4 个测试失败）、`e2e-gitcode` job 失败（2 个测试失败）。**全部失败均为确定性失败（非 flaky）**，根因可归纳为 3 类环境/断言缺陷（见「二、失败归因」）。**更严重的是一项流程发现**：两个失败 workflow run 的完成时间（`02:23:27`–`02:23:35`）**晚于** PR 的 `mergedAt`（`02:20:21`）——即 `gf pr merge 304 --auto` 在这些新增 job 的结果尚未产出前就已完成合并，导致这 8 个已知失败的测试当前已进入 `dev` 分支且未被任何后续验证捕获（`e2e-tests.yml` 的 `push` 触发仅限 `branches: [main]`，`dev` 分支本身无保护、无重跑机制）。**总体判定：5 项技术发现 + 1 项流程发现，需人工介入修复并评估是否需要对已合并的 `dev` 分支采取补救措施。**

## 一、PR #304 关联流水线实测

`feat/291-e2e-gitlab-gitcode-coverage` 分支触发 3 个 workflow run（均创建于 `2026-09-03T02:19:14Z`）：

| Run ID | Workflow（按 job 内容归属） | 状态（采集时→最终） | 结论 | 耗时 |
|--------|------|----------------------|------|------|
| 33707190537 | Smoke Test 跨平台 | success（采集时即为终态） | ✅ success（3 job 全部成功） | ~65s |
| 33707190436 | 主 CI workflow（Check/MSRV/Lint/Smoke Test/Test×3） | running → completed（约 4 分钟后收尾，02:23:28） | ❌ **failure**（Test×3 平台全部失败，其余 4 job 成功） | ~250s（最慢 job） |
| 33707190475 | E2E Tests（GitHub/GitLab/GitCode） | running → completed（约 4 分钟后收尾，02:23:35） | ❌ **failure**（GitHub 成功，GitLab/GitCode 均失败） | ~257s（最慢 job） |

`gf pipeline report --branch feat/291-e2e-gitlab-gitcode-coverage --days 30`（全部 run 终态后复采）：

```json
{
  "totalRuns": 3,
  "successRate": 0.3333333333333333,
  "avgDurationSecs": 193.66666666666666,
  "topFailures": ["failure"]
}
```

**关键时间线**：`gf pr view 304` 显示 `mergedAt: 2026-09-03T02:20:21Z`；而两个失败 run 的 `updatedAt`（完成时间）分别为 `2026-09-03T02:23:28Z`（主 CI）与 `2026-09-03T02:23:35Z`（E2E Tests）——**均晚于合并时间约 3 分钟**。也就是说，`gf pr merge 304 --auto` 是在这两个 workflow 仍处于 `running` 状态、结果尚未产出的情况下完成的合并（详见「六、流程发现」）。

已收尾 job 明细（13 个 job：8 成功、5 失败）：

| Job | Workflow run | 耗时 | 结论 |
|-----|--------------|------|------|
| Check | 33707190436 | 32s | ✅ success |
| MSRV | 33707190436 | 56s | ✅ success |
| Smoke Test | 33707190436 | 77s | ✅ success |
| Lint | 33707190436 | 131s | ✅ success |
| **Test (ubuntu-latest)** | 33707190436 | 140s | ❌ **failure**（2 个测试失败） |
| **Test (macos-latest)** | 33707190436 | 126s | ❌ **failure**（2 个测试失败） |
| **Test (windows-latest)** | 33707190436 | **247s** | ❌ **failure**（2 个测试失败，且本轮最慢 job） |
| E2E Tests (GitHub) | 33707190475 | 58s | ✅ success |
| **E2E Tests (GitLab)** | 33707190475 | **257s** | ❌ **failure**（4 个测试失败） |
| **E2E Tests (GitCode)** | 33707190475 | 193s | ❌ **failure**（2 个测试失败） |
| Smoke Test (gitcode) | 33707190537 | 45s | ✅ success |
| Smoke Test (github) | 33707190537 | 58s | ✅ success |
| Smoke Test (gitlab) | 33707190537 | 61s | ✅ success |

## 二、失败归因

全部 5 个失败 job、8 个失败测试用例均为**确定性失败**（同一根因在对应环境下 100% 复现，非间歇性），可归为 3 类：

### 归因 A：主 CI `Test` job 缺少 `gc` CLI（影响 ubuntu/macos/windows 三平台，各 2 个测试）

`crates/e2e-gitcode/tests/noauth.rs` 的 `test_should_fail_with_login_guidance_when_listing_issues_unauthenticated`（line 61）与 `test_should_fail_with_login_guidance_when_status_checked_unauthenticated`（line 40）在 `.github/workflows/ci.yml` 的 `Test` job（跨 3 平台跑 workspace 级 `cargo test`）中失败：

```
expected login guidance in output, got:   × [gitcode] 未检测到 gc。
  │ 📦 安装：pip install gitcode-cli
  ...
```

**根因**：`crates/e2e-gitcode` 作为普通 workspace 成员，其测试会被 `ci.yml` 的 `Test` job（跨 ubuntu/macos/windows 的通用 `cargo test`）无差别拾取执行；但只有 `e2e-tests.yml` 的专属 `e2e-gitcode` job 才通过 `pip install gitcode-cli` 安装了 `gc` CLI。`ci.yml` 的 `Test` job 环境中 `gc` 不存在，测试断言"应输出登录引导文案"实际收到的是"未检测到 gc 二进制"的安装引导文案——两者不同，断言失败。**这是 PR 引入的真实环境隔离缺陷**：新增测试 crate 未做"仅在已知安装 `gc` 的 job 中运行"的隔离（如通过 feature gate、`#[ignore]` + 显式 include，或将 `crates/e2e-gitcode`/`crates/e2e-gitlab` 从 `ci.yml` workspace 范围排除）。PR 描述中"`cargo test -p e2e-core -p e2e-github -p e2e-gitlab -p e2e-gitcode`（all green）"的本地验证之所以通过，大概率是因为本地开发机已安装 `gc`/`glab`，掩盖了这一 CI 环境差异。

### 归因 B：`e2e-gitlab`/`e2e-gitcode` 的 `issue.rs` 依赖未配置的真实测试仓库（各 1 个测试）

`test_should_list_open_issues_with_valid_schema`（`crates/e2e-gitlab/tests/issue.rs:51` 与 `crates/e2e-gitcode/tests/issue.rs:51`）均报错：

```
stderr:   × Unable to parse owner/repo from URL: https://gitlab.com/.git
stderr:   × Unable to parse owner/repo from URL: https://gitcode.com/.git
```

**根因**：PR 描述"Scope notes"已知承认 `E2E_TEST_REPO_GITLAB`/`E2E_TEST_REPO_GITCODE` 两个 secret 尚未配置，"real-credential scenarios skip gracefully"——但实测该测试**并未优雅跳过**，而是用空字符串拼出了 `https://gitlab.com/.git`（`scratch_repo_dir()` 用未设置的仓库路径拼接 origin URL），导致 URL 解析阶段直接 panic 失败，而非按预期的"无凭据→跳过"路径。**这是 PR 承诺的跳过逻辑与实际实现不一致的缺陷**：`has_gitlab_auth()`/`has_gitcode_auth()` 一类的判定未覆盖到 `issue.rs` 这条路径，或该路径根本没有调用判定就直接构造了仓库 URL。

### 归因 C：`e2e-gitlab` 的 `auth.rs`/`noauth.rs` 因 `go install glab@latest` 产出的二进制版本探测失败（4 个测试）

`test_should_report_logged_in_with_real_credentials`（`auth.rs:33`）与两个 `noauth.rs` 用例（line 33、54）均报错：

```
mode Interactive: stderr:   × [GitLab] glab 版本信息解析失败。
  │ 📦 重新安装：brew install glab
```

**根因**：`e2e-tests.yml` 新增的 `e2e-gitlab` job 通过 `go install gitlab.com/gitlab-org/cli/cmd/glab@latest` 安装 `glab`（而非 `e2e-github` 沿用的包管理器/预装方式），该安装方式产出的二进制 `glab version` 输出格式与 `gitflow-gitlab` crate 的版本解析器不兼容，导致**版本探测阶段就失败**，测试根本走不到"未登录引导"这条预期路径，直接在更早的阶段抛错。这是**安装方式选型问题**：`go install @latest` 编译出的二进制缺少 `ldflags` 注入的版本信息（`glab version` 命令依赖构建时注入的版本字符串），与官方发行的预编译二进制或 Homebrew 版本行为不同。

（对比：`e2e-gitcode` job 用 `pip install gitcode-cli` 安装，未出现同类版本解析问题，说明 `go install @latest` 是本次新增的、`e2e-gitlab` 特有的安装脚本缺陷。）

`test_should_report_logged_in_with_real_credentials`（`crates/e2e-gitcode/tests/auth.rs:53`）额外报错：

```
assertion `left == right` failed: mode Interactive: expected logged-in, stdout: {"success":true,"data":{"loggedIn":false,"scopes":[]},"platform":"gitcode",...}
```

与归因 B 类似——该测试假设"存在真实凭据即应报告已登录"，但由于 `E2E_GITCODE_TOKEN` 未配置（PR 描述已知的 gap），实际拿到 `loggedIn: false`，测试未做"无凭据→跳过"防护，直接断言失败。

## 三、dev / main 基线（7 天 / 30 天，PR #304 合并前采集）

| 分支 | 周期 | Total runs | Success rate | Avg duration | 评级 |
|------|------|-----------:|--------------:|--------------:|------|
| `dev` | 7 天 | 100 | 95.0% | 149.2s | 🟢 Healthy |
| `dev` | 30 天 | 100 | 95.0% | 149.2s | 🟢 Healthy |
| `main` | 30 天 | 100 | 100.0% | 159.59s | 🟢 Healthy |

两个周期样本量均已达窗口上限 100。基线本身健康，PR #304 是本次采集到的**唯一新增失败样本**；由于合并已经发生且 `dev` push 不重跑 `e2e-tests.yml`，这 3 个失败 run 何时会被计入 `dev` 30 天基线取决于平台如何统计——`gf pipeline report --branch dev` 目前仍反映合并前的健康水位，尚未捕获本次合并引入的失败信号。

## 四、耗时分析

| 排名 | Job | 耗时 | 说明 |
|------|-----|------|------|
| 1 | **E2E Tests (GitLab)**（新增 job） | **257s** | ⚠️ 全流水线最慢；且已失败（见归因 C）。相较 `E2E Tests (GitHub)` 基线 58s，**耗时是其 4.4 倍** |
| 2 | **Test (windows-latest)** | 247s | ⚠️ 已失败（见归因 A）；较 PR #302 报告记录的历史基线（172s/185s/288s）量级相近，非本次新增的耗时异常 |
| 3 | **E2E Tests (GitCode)**（新增 job） | **193s** | ⚠️ 已失败（见归因 B/C）。相较 GitHub 基线 58s，**耗时是其 3.3 倍** |
| 4 | Lint | 131s | 正常范围 |
| 5 | Test (ubuntu-latest) | 140s | 已失败（见归因 A），耗时本身在正常区间 |
| 6 | Test (macos-latest) | 126s | 已失败（见归因 A），耗时本身在正常区间 |

**新增 job 耗时瓶颈**：`e2e-gitlab` job 的主要耗时来自 `go install gitlab.com/gitlab-org/cli/cmd/glab@latest`（从源码编译，实测日志显示仅依赖下载阶段就跨越约 10 秒级别的多个 Go module 拉取）叠加独立的 `cargo build --release` 全量构建（未与其余 job 共享增量缓存的场景下，`Swatinem/rust-cache@v2` 命中率如何尚待观察）。`e2e-gitcode` job 用 `pip install gitcode-cli`（预编译 wheel）安装更快，但整体耗时仍是 GitHub 基线的 3.3 倍，主要瓶颈同样是独立的 release 构建而非 CLI 安装本身。**建议**：两个新 job 的 `cargo build --release` 步骤可考虑与 `e2e-github` job 共享构建产物（如上传/下载 artifact），而非各自独立全量构建。

## 五、Flaky 信号

**未发现 flaky test**。本轮 5 个失败 job、8 个失败测试用例全部为**确定性失败**——即在给定环境（缺 `gc`/未配置测试仓库/`go install` 产出的 `glab` 版本解析异常）下会 100% 复现，与代码变更（新增测试文件的环境假设）直接相关，而非间歇性、非确定性的偶发问题。因此本轮判定为**回归缺陷**而非 flaky 信号，优先级应高于 flaky test 的观测性处理。

## 六、流程发现（非本次三维度分析常规项，但影响判断）

`gf pr view 304` 显示 `mergedAt: 2026-09-03T02:20:21Z`；本报告实测的两个失败 workflow run（主 CI、E2E Tests）分别在 `02:23:28Z`、`02:23:35Z` 才完成——**均晚于合并时间约 3 分钟**。这说明：

1. `gf pr merge 304 --auto` 触发的自动合并**没有等待新增的 `e2e-gitlab`/`e2e-gitcode` job（连同同批的 `Test` job 三平台）产出结果**就已经完成合并动作。
2. `.github/workflows/e2e-tests.yml` 头部注释明确写明"`dev` 无保护"（`push` 触发仅限 `branches: [main]`），意味着**这 8 个已知失败的测试目前已合入 `dev` 分支，且不会被任何后续自动化重新验证**，要等到下一次向 `main` 推送（发布前）才会重新触发 `e2e-tests.yml` 走一遍这些 job。
3. 这与 branch protection 的必要检查列表滞后于新增 job 有关——刚新增的 `Test`（因新测试文件产生的失败）与 `e2e-gitlab`/`e2e-gitcode` job 大概率尚未被加入 `dev` 分支的 required status checks 列表，因此自动合并逻辑判定"无阻塞检查"即可合并，而不管这些新 job 的真实运行结果。

## 七、结论

- PR #304 相关的 3 个 workflow run 中 2 个失败、5/13 个 job 失败、8 个测试用例失败，全部为确定性回归（非 flaky），根因分为 3 类：（A）`ci.yml` 的 `Test` job 缺少 `gc` CLI 导致新增的 `e2e-gitcode/noauth.rs` 断言失败（3 平台×2 = 6 处）；（B）`issue.rs` 未对缺失的测试仓库配置做优雅降级，直接崩在 URL 解析（2 处）；（C）`e2e-gitlab` job 用 `go install glab@latest` 安装的二进制版本探测失败，导致 4 个 `auth.rs`/`noauth.rs` 用例失败；另有 `e2e-gitcode/auth.rs` 因未对缺失凭据做保护而失败（1 处，可与 B 归为同类"跳过逻辑未覆盖"问题）。
- `dev`/`main` 历史基线均为 🟢 Healthy（95.0%/100.0%，均 100 次运行样本），PR #304 是本次采集到的唯一失败样本，尚未反映进 30 天基线统计。
- **流程发现（六）：PR 已在 CI 结果产出前完成自动合并**，8 个已知失败的测试目前已进入 `dev` 分支且无后续自动重验证机制——这是比测试本身缺陷更值得立即关注的问题。
- 耗时侧：新增的 `e2e-gitlab`（257s）、`e2e-gitcode`（193s）job 分别是 `e2e-github` 基线（58s）的 4.4×/3.3×，主要瓶颈是独立的 release 全量构建叠加 `go install` 从源码编译 CLI。

## 八、Recommendations

1. 🔴 **High** — 立即修复归因 A：将 `crates/e2e-gitcode`（及 `crates/e2e-gitlab`）的 workspace 测试范围从 `ci.yml` 的通用 `Test` job 中隔离（`--exclude` 或 feature gate），使其只在已安装对应 CLI 的 `e2e-tests.yml` 专属 job 中运行；否则每次向 `dev`/`main` 提交都会在 3 个平台上重复触发同一失败。
2. 🔴 **High** — 修复归因 B：`scratch_repo_dir()`/`issue.rs` 应在拼接仓库 URL 前显式调用 `has_gitlab_auth()`/`has_gitcode_auth()`（或等价的"测试仓库是否配置"判定），未配置时直接 `return`（跳过）而非继续拼出空 owner/repo 的 URL 导致 panic。`e2e-gitcode/tests/auth.rs` 的 `test_should_report_logged_in_with_real_credentials` 同理需要补上"无凭据则跳过"防护。
3. 🟡 **Medium** — 修复归因 C：`e2e-gitlab` job 的 `go install gitlab.com/gitlab-org/cli/cmd/glab@latest` 改为与 `e2e-gitcode` 一致的预编译安装方式（如官方 release 二进制或包管理器），避免源码编译产物的版本字符串与 `gitflow-gitlab` 版本解析器不兼容。
4. 🔴 **High（流程）** — 核查 `dev` 分支的 branch protection required status checks 是否已包含 `e2e-tests.yml` 新增的 `e2e-gitlab`/`e2e-gitcode` job 及 `ci.yml` 的 `Test` 系列 job；若未包含，应补充，避免"自动合并早于 CI 结果产出"的时序问题再次发生。同时建议评估是否需要对已合入 `dev` 的这批已知失败测试采取 hotfix（例如立即提交上述修复 1–3 的 follow-up PR），因为下一次 `main` push 前这些失败不会被任何自动化重新捕获。
5. 🟢 **Low** — 耗时优化：`e2e-gitlab`/`e2e-gitcode` job 各自独立执行 `cargo build --release`，可评估与 `e2e-github` job 共享构建产物（如 artifact 上传/下载或合并为单一构建 job + 矩阵测试 job），以降低新增 job 相对 GitHub 基线 3.3×–4.4× 的耗时倍数。
