# 批次7设计：dogfooding checklist 参数修正 + noauth 测试环境依赖澄清（#361 #356）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-20-004

## #361：dogfooding checklist 命令参数修正

### 现状核实

`gf release create --help` / `gf issue create --help` 实测：

| 清单原文 | 实际正确形式 |
|---|---|
| `gf release create v0.x.x --notes "test release"` | `gf release create --tag-name v0.x.x --body "test release"`（`--tag-name` 是必填标志非位置参数；`--body` 不是 `--notes`） |
| `gf issue create --title "..." --labels "测试标签"` | `gf issue create --title "..." --label "测试标签"`（单数，可重复指定） |

### 方案

1. 改 `docs/specs/phase4-dogfooding-checklist.md` 里这两处命令
2. 头部"最后更新: 2026-07-10"改为本次日期，"适用版本"核对当前 CLI 版本
3. **防漂移手段**：不扩展 `scripts/validate-skill-commands.sh`——该脚本当前作用域是
   `skills/*/SKILL.md`，且只验证命令名存在性，不验证参数正确性；要覆盖本文档需要
   同时扩大扫描范围和加深验证粒度，工作量与本次纯文档修正不成比例（YAGNI）。改为
   在清单头部补一条说明：命令随 CLI 版本升级需人工核对，标注本次验证所用的
   CLI 版本与日期，作为下次漂移时的参照基线

## #356：noauth 测试环境依赖澄清

### 现状核实

- `crates/e2e-github/tests/noauth.rs:1` 模块文档声称"任何环境均可运行"，不成立——
  隐含要求 `gh` 已安装，未安装时输出安装引导而非登录引导，测试用
  `assert!(combined.contains("gh auth login"))` 断言登录引导文案，因而失败
- `crates/e2e-gitlab/tests/noauth.rs:1` 模块文档**已经准确**（"前提是运行环境已安装
  `glab` CLI"），但运行时同样没有区分"未安装"与"已安装未登录"两种失败，同款问题
- `crates/e2e-gitcode/tests/noauth.rs` 已经修复过（模块文档已准确声明前提，且已用
  `GC_TOKEN` 注入方案处理认证态歧义）——**不在本次范围内**

### 方案

两个文件（`e2e-github`、`e2e-gitlab`）的两条测试都加一段 skip 逻辑：检查 `gf` 的
实际输出是否包含对应平台的"未检测到 CLI"安装引导文案（如"未检测到 gh"），命中则
`eprintln!("skipped: ...")` + `return`，不再往下断言登录引导。不命中才继续原有的
"必须包含登录引导"断言。

这个判定复用 `gf` 自己的检测路径产生的实际输出，不用 `which` 单独探测 PATH，
保证测试断言和真实运行时行为一致；也是本代码库
`crates/e2e-core/tests/harness.rs`（`gf` 二进制缺失时 skip）已经在用的既有约定，
风格统一，不引入新模式。

`e2e-github`：模块文档同步改为准确表述（去掉"任何环境均可运行"）。
`e2e-gitlab`：模块文档已经准确，不用改文案，只加运行时 skip 逻辑。

## 测试策略

- `#361`：纯文档改动，人工核对新命令能实跑（已用 `--help` 验证参数存在，交付前再跑
  一次真实命令确认）
- `#356`：Rust 测试代码改动，跑 `cargo test -p e2e-github -p e2e-gitlab --test noauth`
  验证正常路径不受影响（本机已装 gh/glab，测试应仍走原有断言路径，不会误 skip）；
  用受限 PATH（去掉 gh/glab）人工验证新 skip 逻辑生效，不产生误导性失败
