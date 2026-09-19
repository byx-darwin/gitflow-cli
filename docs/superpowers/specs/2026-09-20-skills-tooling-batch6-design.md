# 批次6：三个决策落地设计（#369 #339 #352）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-20-003

## 背景

三个此前需要用户拍板的决策，均已确认：

| Issue | 决策 |
|---|---|
| #369 | `cargo-fmt` pre-commit hook 改为自动写回（去掉 `--check`） |
| #339 | `_common.sh` 整体删除（含 wrapper 与 install.sh 分发逻辑） |
| #352 | `make install-skills` 同步整个 `docs/` 目录到安装目录 |

## 方案

### #369：cargo-fmt hook 去掉 `--check`

`.pre-commit-config.yaml` 里 `cargo-fmt` hook 的 `entry` 从
`bash -c 'cargo +nightly fmt -- --check'` 改为 `bash -c 'cargo +nightly fmt'`。

**AC2 实测验证**（用 `end-of-file-fixer` 当活体样本，机制与本次改动完全一致）：
制造一个无结尾换行的探针文件，跑 `pre-commit run --files <probe> end-of-file-fixer`：

```
fix end of files.........................................................Failed
- hook id: end-of-file-fixer
- exit code: 1
- files were modified by this hook
Fixing docs/eof_probe_TEMP.txt
```

`git status` 显示 `AM`（已改但未重新 add）。**结论**：改成自动写回后，提交流程从
"❌ → 手动跑 fmt → 重新 add → 重新提交"（三步）变成
"❌（文件已被自动改好）→ 重新 add → 重新提交"（两步）——去掉了"手动跑 fmt"这一步，
但仍需要重新 add + 重新提交，不是一次性成功。这与 issue 原文的权衡分析完全一致。

**AC4 同文件内其余 hook 检查**：`cargo-check`、`cargo-deny`、`typos`、`cargo-clippy`
（`stages: [pre-push]`）、`cargo-test` 均为只读检查，不修改工作区文件，无同类体验问题。

### #339：`_common.sh` 整体删除

删除三处：
1. `skills/_common.sh`（91 行，4 个函数：`json_escape`/`detect_platform`/
   `cd_to_git_root`/`check_prerequisites`，均确认零调用方）
2. `scripts/_common.sh`（thin wrapper，只 source 上面那个文件，同样零调用方）
3. `scripts/install.sh:362-382` 的分发逻辑：移除"复制 `_common.sh` 共享库"代码块
   （4 行）；同时移除紧邻的一条已经不准确的旧注释（`# 跳过非 gf 前缀的目录（如
   _common.sh 所在的父目录不会被遍历）`——这条注释描述的行为在当前代码里根本不存在，
   属于历史遗留的误导性注释，一并清理）

**范围外**：`CHANGELOG.md`、`docs/issue-triage-report-*.md`、
`docs/superpowers/plans/2026-07-*.md` 等历史记录文档里提到 `_common.sh` 的地方
不动——它们是过去事件的记录，不是活文档。

### #352：`install-skills` 同步整个 `docs/` 目录

现状核实：9 个受影响 skill 里，真正会在安装后**断链**的是用 `../../docs/...`
相对路径链接引用的那些（如 `gf-pr-review`、`gf-label-stats`）——因为
`~/.claude/skills/<skill>/SKILL.md` 装好后，`../../docs/...` 恰好还是从
`~/.claude/skills/<skill>/` 出发向上两级，如果 `~/.claude/` 下也有一份 `docs/`，
链接自然就通了。其余"裸路径"提法（如 `gf-pr/SKILL.md` 里的
`docs/references/gf-pr-params.md`，`#338` 已挪到 See Also 的那条）是给阅读源码仓库
的人看的说明文字，不是需要在安装后可解析的相对链接，不受影响也不需要处理。

**方案**：`make install-skills` 在 `cp -r skills/* "$$D"/` 之后，追加
`cp -r docs "$$(dirname "$$D")"/`——把整个 `docs/` 目录复制到安装目录的**上一级**
（即 `~/.claude/docs/`，与 `~/.claude/skills/` 同级），镜像源码仓库里
`skills/` 与 `docs/` 同级的布局，让所有 `../../docs/...` 相对链接在安装后与在源码
仓库里的解析结果完全一致。

**为什么不逐个 skill 建映射表**：`docs/` 4.8M、318 个文件，整体同步比维护一份
"哪个 skill 引用了哪个 docs 路径"的映射表更简单、更 DRY，也自动覆盖未来新增的
同类引用，不需要每次新增一个 skill 就更新映射表。这也是
`skill-conventions.md` §1.3 本身鼓励的"外置到 docs/ 控制 token 预算"这一模式
应有的、不该处处例外的基础设施支持。

## 测试策略

三处均为 Makefile/bash/yaml 配置改动，无 Rust 单测。验证方式：
- `#369`：已用 `end-of-file-fixer` 实测验证同款机制（见上）；改完后跑一次真实的
  `cargo +nightly fmt` 制造格式问题再提交，确认新行为符合预期
- `#339`：删除后跑 `grep -rn "_common.sh"` 确认仅历史文档命中，代码/脚本零命中；
  `make install-skills`、`scripts/install.sh` 正常跑通不报错
- `#352`：`make install-skills`（或等效探针）后，人工核对 `~/.claude/docs/` 存在
  且 `gf-pr-review` 的 `../../docs/references/pr-review-checklist.md` 从安装后的
  `SKILL.md` 位置出发能正确解析到文件
