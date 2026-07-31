# 设计文档:e2e 实化 + 发布流水线加固(Issue #96)

- **状态**:已批准(brainstorming 三段评审通过)
- **日期**:2026-07-31
- **关联**:Issue #96 · 路线图 #93 阶段一第 5-6 周(`docs/superpowers/specs/2026-07-31-product-evaluation-roadmap-design.md` §6.1)· 工作流合同 `wf-2026-07-31-003`
- **前置成果**:#95/#104(契约测试 + 兼容性矩阵 + 版本护栏)已合并

## 1. 背景与问题

路线图评估识别两个结构性风险:

1. **e2e 流水线空转**:`e2e-tests.yml` 存在且每日触发,但 `e2e-github` 的测试是空泛断言——
   `assert!(success || 输出非空 || contains("error") || contains("login"))` 覆盖一切可能,
   恒绿,什么都不验证。且测试未向 `gh` 子进程注入 `GH_TOKEN`,真实凭据从未生效。
2. **发布模板事故**:`release.toml` 使用 cargo-release 旧语法 `{{version}}`(新版为 `{version}`),
   模板展开失败,历史提交 `9331bfa`/`0b0e9d7` 字面生成 `chore: release v{{version}}`;
   发布脚本无事后校验,无法拦截此类产物。

## 2. 目标与退出标准

| Issue #96 任务 | 退出标准 | 本设计组件 |
|----------------|----------|-----------|
| e2e-core 与 e2e-github 各 ≥3 只读实测场景 | nextest 报告含严格断言的真实测试 | A |
| 每周定时 e2e 回归(真实凭据) | `e2e-tests.yml` 周一 cron 定时运行且全绿 | B |
| 上游 CLI 新版本巡检预警 | nightly workflow 产生/更新预警 Issue | C |
| 发布加固(模板校验 + dry-run 强制清单) | `make release-rehearse` 演练通过 | D |

## 3. 总体架构

四个组件 + 一条主线:**让 CI 在真实环境中验证真实行为,让发布流程拦截畸形产物**。

| 组件 | 交付物 | 边界 |
|------|--------|------|
| A:e2e 实化 | 增强 `e2e-core`(双层模式支持)+ 实化 `e2e-github` 测试 | 仅 Rust 测试代码 + 设置指南 |
| B:每周定时回归 | 更新 `.github/workflows/e2e-tests.yml` | 仅 workflow 配置 |
| C:上游巡检 | 新增 `.github/workflows/upstream-patrol.yml` + `upstream-drift` label | 仅 workflow + 标签 |
| D:发布加固 | 修复 `release.toml`、加固 `scripts/release.sh`、新增 `make release-rehearse`、更新文档 | 不动 cd.yml / release.yml |

## 4. 组件 A:e2e 实化

### 4.1 e2e-core 增强

**双层模式支持**(支撑组件 B):

- `TestConfig::from_env()` 新增派生字段 `mode: TestMode { Authenticated, Unauthenticated }`:
  当 `github_token.is_some()` 时为 `Authenticated`,否则 `Unauthenticated`。
- `TestConfig` 新增方法:
  - `gh_env() -> Vec<(&str, String)>`:返回 `[("GH_TOKEN", token)]`(有凭据时)或空,
    供 `TtyRunner::env()` 批量注入——**修复"凭据从未传给 gh 子进程"的根因**。
  - 测试侧跳过约定:实测测试开头以 `if config.mode == TestMode::Unauthenticated { eprintln!("skipped: no credentials"); return; }` 守卫(无需宏,nextest 正常通过);
- `TtyRunner` 现有 `.env(k, v)` 接口足够,不新增接口。

**e2e-core 自测(≥3 个,无需网络,所有路径均运行)**:

| # | 场景 | 严格断言 |
|---|------|----------|
| 1 | 二进制发现与启动 | Interactive/NonInteractive 两种模式跑 `--help`:exit 0 且 stdout 含 `gitflow` |
| 2 | `from_env` 解析 | 有 token → `Authenticated`;无 token → `Unauthenticated`;`E2E_TEST_REPO` 缺失 → `Err(MissingEnvVar)` |
| 3 | fixture 生命周期 | `TestFixture`/`TestResource` 创建与清理(drop 后资源消失) |
| 4 | 错误传播 | 未知子命令:exit ≠ 0 且 stderr 非空 |

### 4.2 e2e-github 实化

**前置(e2e-test-repo 种子)**:在 `byx-darwin/e2e-test-repo` 预置固定 fixture
Issue(标题 `e2e-fixture-issue`,长期不关闭)与 ≥1 个 PR;seed 步骤写入
`docs/e2e-test-setup-guide.md`;断言失败信息直接指向重新 seed 的步骤。

**实测场景(≥3 个只读,严格断言,无凭据自动跳过)**:

| # | 命令 | 严格断言(取代空泛断言) |
|---|------|--------------------------|
| 1 | `auth status --platform github` | exit 0,输出含已认证标识 |
| 2 | `issue list --repo $E2E_TEST_REPO` | exit 0,输出含 `e2e-fixture-issue` |
| 3 | `pr list --repo $E2E_TEST_REPO` | exit 0,输出含已知 fixture PR 标识或结构化表头 |

所有实测测试通过 `config.gh_env()` 注入 `GH_TOKEN`。

**无凭据错误路径(PR 路径运行)**:

| # | 场景 | 严格断言 |
|---|------|----------|
| 1 | 未认证 `issue list`(不注入 token) | exit ≠ 0,stderr 含中文登录引导(验证 #95 中文优先错误 UX) |
| 2 | 未认证 `auth status` | 输出明确指示未登录状态 |

## 5. 组件 B:`e2e-tests.yml` 每周定时回归

```yaml
on:
  push: { branches: [main] }
  pull_request:
    branches: [main]
    paths: ['crates/**', 'apps/**', '.github/workflows/e2e-tests.yml']
  schedule:
    - cron: '0 2 * * 1'   # 每周一 02:00 UTC(从每日改为每周)
  workflow_dispatch:
```

**模式判定完全交给测试层**:fork PR 取不到 secrets → `E2E_GITHUB_TOKEN` 为空 →
实测测试自动 skip、无凭据错误路径与 e2e-core 自测正常运行。workflow 保持单 job,
无 if 分支逻辑:

- 步骤沿用现状:checkout / rust-toolchain / rust-cache / nextest / release build / PATH
- 测试命令:`cargo nextest run -p e2e-core -p e2e-github --all-features`
  (新增 `-p e2e-core`,让 harness 自测进入 CI)
- 新增 mode 汇报步骤:`if: always()`,将 authenticated/unauthenticated 写入
  `$GITHUB_STEP_SUMMARY`
- 保留 artifact 上传

**绿色语义**:PR 绿 = 错误路径 + harness 自测通过;schedule/main/dispatch 绿 = 实测全过。

## 6. 组件 C:`upstream-patrol.yml` nightly 巡检

```yaml
on:
  schedule: [{ cron: '0 3 * * *' }]   # 每天 03:00 UTC
  workflow_dispatch:
permissions: { contents: read, issues: write }
```

### 6.1 Job `version-check`(无需凭据)

1. 安装最新版 gh / glab / gitcode(gitcode 安装失败 → `::warning::` 记录并跳过,不 fail)
2. `jq` 读取 `docs/compatibility-matrix.json` 的 `min_version` / `tested_versions`
3. 比较:installed > tested → 判定"新版本出现"
4. 预警:创建/更新 Issue(见 6.3)

### 6.2 Job `github-live-check`(需 `E2E_GITHUB_TOKEN`)

1. 安装**最新版** gh
2. `GH_TOKEN=$E2E_GITHUB_TOKEN scripts/smoke-test.sh --platform github --read-only`
3. 失败 → 判定"新版本破坏"(行动级),创建/更新 Issue

### 6.3 Issue 预警规则(防 nightly 刷屏)

- 标签 `upstream-drift`(随本 PR 创建,颜色沿用现有标签风格)
- 标题分级:
  - 信息级:`upstream CLI 新版本: <binary> <version>`
  - 行动级:`upstream CLI 破坏: <binary> <version>`
- 去重:创建前 `gh issue list --label upstream-drift --state open --search "in:title <binary>"`
  → 命中则 `gh issue comment` 追加本次巡检结果,否则创建
- Issue 正文:当前版本 / tested 版本 / 失败摘录 / 兼容性矩阵链接 / 建议动作
  (跑 e2e 验证 → 更新矩阵 tested_versions → 必要时提升 min_version)

**边界**:live-check 仅覆盖 github(gitlab/gitcode 无 CI 凭据,与 #96 范围一致)。

## 7. 组件 D:发布加固

### 7.1 根因修复 — `release.toml`

```toml
tag-name = "v{version}"
tag-message = "Release v{version}"
pre-release-commit-message = "chore: release v{version}"
```

保留 `verify = true`、`allow-branch = ["main"]`。

### 7.2 事后校验闸门 — `scripts/release.sh`

新增 `validate_release_artifacts()` 函数组(**纯函数,接受字符串参数,不依赖 cargo**,可独立测试):

| 时机 | 校验规则 | 失败动作 |
|------|----------|----------|
| `cargo release commit` 后 | commit subject 匹配 `^chore: release v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$` 且不含 `{{` | 中止 + 输出恢复步骤(`git reset --hard HEAD~1`),exit 1 |
| `cargo release tag` 后 | tag 名匹配 `^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$` 且不含 `{{` | 中止 + `git tag -d <bad-tag>`,exit 1 |
| `git cliff` 后 | `grep -q '{{.\+}}' CHANGELOG.md` 为假 | 中止,exit 1 |

### 7.3 强制清单 — `preflight_checklist()`

清单项:① 在 `main` 分支 ② 工作区干净 ③ `cargo nextest` 全绿 ④ clippy 全绿
⑤ 版本预览确认 ⑥ dry-run 通过。

**行为变更(核心)**:`--quick` 仅跳过交互确认,**不再跳过 dry-run**——
现行 `if ! $QUICK_MODE; then dry_run; fi` 改为无条件执行。

### 7.4 演练模式

- 新增 `release.sh --rehearse`:完整 dry 链路(preflight + 清单 + `cargo release --dry-run`
  + changelog 预览 + 用模拟字符串过校验函数),输出 ✅/❌ 清单报告;
  绝不产生变更(不 bump / commit / tag / push / publish);任一失败 → exit 1
- 新增 `make release-rehearse` 目标(加入 `.PHONY`)
- 退出标准"1.0 发布 dry-run 演练成功"= 演练一次 `make release-rehearse` 并附输出摘录至 Issue #96

### 7.5 文档

`docs/release-workflow.md` 追加:`v{{version}}` 事故复盘(根因 + 修复)、校验闸门说明、
演练流程、受验证的 cargo-release 版本。

## 8. 测试策略(TDD:RED → GREEN → REFACTOR)

| 层 | 测试 | 位置 |
|----|------|------|
| e2e-core harness | ≥4 自测(见 4.1) | `crates/e2e-core/tests/` + `#[cfg(test)]` |
| e2e-github | ≥3 实测 + 2 无凭据错误路径(见 4.2) | `crates/e2e-github/tests/` |
| 发布校验 | `release.sh --self-test`:好坏字符串过纯校验函数(正常 / `v{{version}}` / 坏 tag / CHANGELOG 残留) | 脚本内自测模式 |
| workflow | `actionlint`(若本地可用)+ dispatch 触发验证 | — |

## 9. 风险与缓解

| 风险 | 缓解 |
|------|------|
| gitcode CLI 在 GitHub runner 安装不稳 | version-check 仅警告不阻断 |
| e2e-test-repo fixture Issue 被误删 | setup guide 记录 seed 步骤;断言失败信息指向重新 seed |
| cargo-release 版本差异再致模板异常 | 演练 + 事后校验双保险;文档固化受验证版本 |
| 上游新版本 ≠ 破坏,误报噪音 | Issue 标题"新版本"(信息级)与"破坏"(行动级)分级 |
| weekly cron 时间窗口内 secrets 过期 | setup guide 已有 90 天轮换提示;实测失败本身即暴露过期 |

## 10. 明确不做(YAGNI)

- e2e-gitlab / e2e-gitcode crate(本 Issue 未要求)
- CI 演练 workflow(第 7-8 周 1.0 准备期按需再加)
- Homebrew / crates.io 元数据变更(第 7-8 周任务)
- 兼容性矩阵自动更新(预警交人工判断)

## 11. 决策记录(brainstorming 评审)

| 决策点 | 选择 | 备选 |
|--------|------|------|
| 范围组织 | 一份设计文档,4 组件 | 拆两份 / 拆四份 |
| 凭据策略 | 双层模式(PR 无凭据错误路径 + 定时真实凭据) | 仅定时 / PR skip 兜底 |
| 巡检机制 | nightly + Issue 预警(去重) | dependabot 式自动 PR / 每周搭车 |
| 发布加固 | 本地强制闸门(校验 + 清单 + 演练) | 本地 + CI 演练 workflow / 仅修复 + 文档 |
