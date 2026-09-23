# Code Review Report — Issue #378

> **交付方式：** `local_merge`（`git merge --no-ff`），无 PR 编号可挂载正式 GitHub review verdict。本报告是 gf-workflow Phase 4 的书面存档，`gf review approve/request-changes` 的 CLI 提交步骤因无 open PR 而不适用（`gf-review` skill 的 precondition 是 PR open）。
> **Merge commit:** `a9da23f70ac134e6a1d74544835bd95f41775ce4`
> **Diff range:** `5a2f955..a9da23f`
> **分析日期：** 2026-09-20
> **工作流：** `wf-2026-09-20-005`

## 背景

Issue #378 报告 `gf pipeline report` 在 run 未收尾时把 in_progress job 误计为失败，且此前一次修复提交（`5051966`）被十次实测认定无效。Phase 1 排查（见 Issue #378 评论 5749481258）确认：成功率计算逻辑本身已经正确（`crates/github/src/pipeline.rs` 的 `report()` 早已按 `status == "completed"` 正确过滤非终态 run，且有回归测试覆盖），真正根因是 `gf --version` 不带 git commit 信息，导致无法在生成快照时确认所用二进制是否包含目标修复。本分支的修复范围是：给 `gf --version` 嵌入 git short SHA，并在 `gf-pipeline-analyzer` skill 中补一条核对提示——不改动、也未新增任何计算逻辑或回归测试。

## 独立复核（不复用 Phase 3 结论，重新读 diff 独立判断）

改动文件：`apps/cli/build.rs`（+54/-6）、`apps/cli/src/main.rs`（+31/-11）、`apps/cli/tests/version_test.rs`（新增 36 行）、`skills/gf-pipeline-analyzer/SKILL.md`（+5）。

| 检查点 | 结论 |
|---|---|
| `crates/github/src/pipeline.rs` 是否被改动 | 未改动，符合 Global Constraint |
| 是否新增依赖 | 无 `Cargo.toml`/`Cargo.lock` 变化 |
| `write_git_sha` 是否可能 panic | 否——两处 `git` 调用全部走 `if let Ok(...) &&`/`.ok().filter().and_then().unwrap_or_else(\|\| "unknown")` 容错链；唯一 `.expect()` 落在 `fs::write` 写 `OUT_DIR` 失败上，属于 build script 应当直接失败的场景（build.rs 文件头已有带理由的 `clippy::expect_used` 允许） |
| `cargo:rerun-if-changed` 是否覆盖 `HEAD` 与 `logs/HEAD`，且用 `--git-path` 正确处理 worktree | 是，第 108-119 行按此实现；这是修复本身是否"不会再陈旧"的关键点，逻辑正确 |
| `gf --version` 输出格式 | `format!("{} ({})", built_info::PKG_VERSION, build_git::GIT_SHA)`，符合要求 |
| `clap::Command::version` 需要 `'static` 字符串，工作区未启用 clap `string` feature 的应对 | `Box::leak` 一次性泄漏，仅在 `main()` 顶部执行一次，注释清楚说明原因，字符串极小——可接受的权衡，非缺陷 |
| `built_info` 文档注释此前声称暴露 `GIT_COMMIT_HASH`（实为不存在，因 `built` 未启用 `git2` feature） | 已在本次改动中一并订正为准确描述 |
| 新增集成测试是否真的验证行为 | `apps/cli/tests/version_test.rs` 通过 `env!("CARGO_BIN_EXE_gf")` 跑真实二进制，断言 `starts_with("gf ")` 且含括号后缀——对着修复前的 `"gf 1.9.0"`（无括号）会失败，是真实回归防护而非重言式 |
| SKILL.md 新增说明是否符合仓库语言约定 | 是（English 正文 + Issue #378 引用），初版为中文，已在 Phase 3 fix round 中改为英文并复核通过 |

## 本地验证

```
cargo test -p gitflow-cli                                                        # 全绿
cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic   # 无警告
cargo +nightly fmt -- --check                                                    # 无差异
make test（合并后，dev 分支全量）                                                    # 1560/1560 passed
```

## 真实场景验证（AC4，见 Issue #378 评论 5749906008）

因 `dev` push 不触发 CI（`ci.yml` 明确排除），另开临时 PR #379（`chore/378-ci-verification-scratch` → `dev`，验证后已关闭不合并）取得一次真实的 in-progress 快照：`gf pipeline report` 在两个 run 均为 `running` 时返回 `totalRuns: 0`，未误计为失败，同时复现并纠正了「未重新编译的二进制报告陈旧 sha」的现场（`a9da23f` → 重新编译后变为真实 HEAD `c8a1ebd`）。

## 遗留 / 不阻塞项

- `version_test.rs` 的断言形状略宽松（sha 为空串也会通过括号形状检查），但 `build.rs` 已用 `.filter(|s| !s.is_empty())` 兜底，实际不可达。
- 无 reflog 的仓库会让 `logs/HEAD` 的 `rerun-if-changed` 监视路径不存在，导致每次构建都判定为 dirty——仅影响构建耗时，不影响正确性。
- `skills/gf-pipeline-analyzer/SKILL.md` 中 `## Preconditions` 与 `## Overview` 之间缺一个空行——确认为改动前就存在的既有格式问题，不属于本次改动引入，按"最小改动"原则不在本次顺手修，留给专门的格式化处理（如与 #350 同类的收尾）。

## 结论

**Approve.** 无 Critical/Important 发现。改动范围精确匹配计划声明的文件清单，无死代码、无 TODO、无范围蔓延，测试与实测证据齐全。Issue #378 的四项 AC 已全部收口（AC1/AC2 见 Phase 1 排查、AC3 由既有测试满足、AC4 见上文真实验证记录）。

因交付方式为 `local_merge`，本报告即为正式书面存档，不额外调用 `gf review` CLI 提交动作。
