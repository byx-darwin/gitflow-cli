# 走查包：新增 gf-walkthrough 交付走查技能（Issue #329）

## ① 开场叙事

这次变更给团队交付流程带来一个新的“说明书生成器”：当一次代码改动完成后，
系统能自动整理出一份给非工程背景的人也能看懂的说明，讲清楚“改了什么、
凭什么说测试过、还有什么没测”，而不需要开发者临时手写。以往交付走查靠
个人经验口头描述，验证到什么程度、哪些结论只是猜测，常常混在一起说不清；
这份说明要求每一条结论都必须标注清楚：是亲自跑过命令验证的（`Measured`）、
是看代码推断出来的（`Inferred`），还是根本没验证过的（`Unverified`）。
现在做这件事，是因为 Issue #329 明确把“证据分级造假”列为交付走查的头号
风险——本走查包本身，就是用这套新方法给它自己打的第一份体检报告，
用来证明这套方法真的可以从头到尾走通，而不是纸上谈兵。

## ② 变更摘要

```
$ git diff --stat dev..HEAD
 Makefile                                           |  61 ++-
 .../superpowers/plans/2026-09-16-gf-walkthrough.md | 423 +++++++++++++++++++++
 .../specs/2026-09-16-gf-walkthrough-design.md      | 248 ++++++++++++
 .../templates/walkthrough-report-template.md       |  55 +++
 skills/gf-walkthrough/SKILL.md                     | 149 ++++++++
 5 files changed, 935 insertions(+), 1 deletion(-)
```

按文件分组说明（6 个提交：`19d99e6` 澄清与计划文档 → `8c889bb` RED 验证器 →
`53e6a52` 验证器死代码清理 → `95189b0` GREEN SKILL.md 主体 → `c25a837`
模板外置 + 压字数 → `450fce3` 评审修复；本次 Task 4 再加第 7 个提交）：

- **`Makefile`**（`8c889bb`、`53e6a52`、`c25a837`）— 新增 `check-walkthrough-skill`
  目标（12 项硬约束校验），随后清理了其中 #3 检查用的 awk 脚本里两处从未被读取的
  死代码（`n=NR` 赋值、`buf[]` 数组），再修复了 #4 循环“只在失败时打印”导致
  通过与从未执行无法区分的问题（改为 `HEADER` 标志位，仿照 #2 的 `TIER` 模式）。
  这是唯一被本次改动触碰的可执行逻辑，其余全部是文档。
- **`docs/superpowers/plans/2026-09-16-gf-walkthrough.md`**、
  **`docs/superpowers/specs/2026-09-16-gf-walkthrough-design.md`**（`19d99e6`）—
  Issue #329 的澄清与四阶段计划文档，是本变更的需求与设计来源，不是产物本身。
- **`docs/superpowers/templates/walkthrough-report-template.md`**（`c25a837`
  新增）— 报告骨架 + 10 项自检清单，从 SKILL.md 正文中抽出来单独存放，
  为的是让 SKILL.md 正文压到 500 词硬限以内（详见 `skill-conventions.md`
  §1.3 的“外置模板”约定）。
- **`skills/gf-walkthrough/SKILL.md`**（`95189b0` 新增，`c25a837` 压缩、
  `450fce3` 修复评审发现的范围声明缺失）— 技能正文本身：证据分级三档、
  失败测试三列溯源表、只读重跑白名单、与 `gf-review`/`gf-pr-review`/`gf-smell`
  的边界划分。这是本变更真正新增的“能力”，其余文件都是围绕它的脚手架
  （测试、文档、模板）。

## ③ 验证证据

- [Measured] `make check-walkthrough-skill` 在本报告落盘前呈现预期的“部分跳过”状态
  ```
  $ ls docs/walkthrough-*.md 2>/dev/null; echo "---"; make check-walkthrough-skill; echo "exit=$?"
  (eval):1: no matches found: docs/walkthrough-*.md
  ---
  ✓ #1 正文无语言专属标识
  ✓ #2 三档标记齐备
  ✓ #4 失败测试表三列齐备
  ✓ #5 禁用词规则已声明
  ✓ #9 工具集含 Write 不含 Edit
  ✓ #10 复用既有语言探测
  ✓ #11 词数 499 ≤ 500
  ✓ #12 报告模板存在
  ✗ #8 未找到非空的 docs/walkthrough-*.md
  — #3 跳过（无报告）
  — #6 跳过（无报告）
  — #7 跳过（无报告）
  存在未通过项
  make: *** [check-walkthrough-skill] Error 1
  exit=2
  ```
  这证明 #1、#2、#4、#5、#9、#10、#11、#12 在报告落盘前已经全绿，
  #3/#6/#7/#8 是因为“报告文件还不存在”而跳过或失败，符合校验器设计
  （`Makefile:264-286`）。本报告文件写入后需要重新运行本命令确认全 12 项转绿
  （见下一条）。

- [Measured] `cargo build` 在新增 `skills/gf-walkthrough/` 后仍可正常生成嵌入清单
  ```
  $ cargo build --quiet; echo "exit=$?"
  exit=0
  ```
  以及嵌入清单确实收录了新增的技能目录：
  ```
  $ find target -name "skills_manifest.rs" | while read f; do echo "== $f =="; grep -c "gf-walkthrough" "$f"; done
  == target/debug/build/gitflow-cli-fdb4098534c7ec57/out/skills_manifest.rs ==
  48
  ```
  `apps/cli/build.rs:75` 声明了 `cargo:rerun-if-changed=skills/`，新增
  `skills/gf-walkthrough/SKILL.md` 会触发该清单重新生成；上面的 `grep -c`
  非零，说明新技能的路径条目确实被写入了生成产物，构建脚本未被破坏。

- [Measured] `skills/gf-walkthrough/SKILL.md` 正文词数在 500 词硬限内
  ```
  $ S=skills/gf-walkthrough/SKILL.md; W=`perl -0 -ne 's/^---\n.*?^---\n//ms; s/\x60\x60\x60.*?\x60\x60\x60//gs; s/\x60[^\x60]+\x60//g; print scalar(()=/\p{L}+/g)' "$S"`; echo "word_count=$W"
  word_count=499
  ```
  用的是 `Makefile:259` 里 `check-walkthrough-skill` #11 检查项的同一条命令
  （已修正 `.superpowers/sdd/2026-09-16-gf-walkthrough/task-4-brief.md` 提到的
  仓库既有缺陷：旧版 `docs/superpowers/templates/skill-conventions.md:21`
  给出的计数命令因 `scalar(/.../g)` 在标量上下文里返回布尔值而恒为 `1`，
  本命令改用 `scalar(()=/\p{L}+/g)` 取真实匹配次数）。

- [Measured] 全量测试套件在隔离 `gc`/GitCode 命令名冲突后可运行，全部通过
  ```
  $ MIRROR=$(mktemp -d); for f in /opt/homebrew/bin/*; do b=$(basename "$f"); [ "$b" = gc ] && continue; ln -s "$f" "$MIRROR/$b" 2>/dev/null; done; env PATH="$MIRROR:$HOME/.cargo/bin:/usr/bin:/bin" cargo test --workspace --quiet > /tmp/cargo_test_out2.txt 2>&1; echo "exit=$?"; grep -c "test result: ok" /tmp/cargo_test_out2.txt; grep -oE '[0-9]+ passed' /tmp/cargo_test_out2.txt | awk -F' ' '{s+=$1} END{print s}'
  exit=0
  39
  1450
  ```
  即：39 个测试组（含集成测试与合并文档测试）全部 `test result: ok`，
  逐组通过数相加共 1450，0 failed，命令整体 `exit=0`。
  Workaround 说明：本机 `/opt/homebrew/bin/gc`（Graphviz）与 GitCode CLI 的
  `gc` 命令名冲突，会让两个 `e2e-gitcode` 用例假失败；直接从 `PATH` 剔除
  `/opt/homebrew/bin` 又会连带丢掉 `gh`/`glab`，破坏 `e2e-github`/`e2e-gitlab`。
  实测可行的隔离方式：把 `/opt/homebrew/bin` 软链接镜像到一个临时目录，
  唯独跳过 `gc`，再把该临时目录连同 `~/.cargo/bin`、`/usr/bin`、`/bin`
  组成新 `PATH` 传给 `cargo test`（完整命令见上）。这条 workaround 下
  `e2e-gitcode` 未出现假失败，套件整体也没有任何失败——本次 Fix Round 1
  之前的版本曾在此处声称观察到 4 条 `crates/gitlab/src/auth.rs` 失败用例，
  经复核为编造内容，现已订正：本次实测未出现任何测试失败。

- [Inferred] AC#3（失败测试三列溯源表）本次未被触发，机制定义于代码中未被验证 —— 本次
  `cargo test --workspace` 实测 0 failed（见上一条），因此没有失败用例需要
  溯源，三列表格本节不出现。三列溯源机制本身的定义位置为
  `skills/gf-walkthrough/SKILL.md:46-52`（`## Failing Tests` 一节的表头与
  “Missing commit hash together with `unrelated` → forbidden” 规则）及
  `docs/superpowers/templates/walkthrough-report-template.md:28-36`（表格
  骨架与 ancestry check 脚本）。这条判定只依据读代码得出，未在本次运行中
  被真实的失败用例练习过，因此标 `Inferred` 而非 `Measured`：如实说明
  “本次未验证过该机制在真实失败上的运作方式”，比伪造一张失败表格更诚实。

- [Inferred] `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 豁免未运行
  —— 依据 `docs/walkthrough-feat-329-gf-walkthrough-2026-09-16.md` 本节所述
  变更集：本次 6+1 个提交只修改了 `Makefile`（shell 脚本片段）和 5 个
  Markdown 文件，未新增或修改任何 `.rs` 文件（`git diff --stat dev..HEAD`
  见 ② 节，逐项确认后缀均非 `.rs`），对照
  `CLAUDE.md:50`「When touching production Rust code, run `cargo clippy`…」
  与 `CLAUDE.md:49`「Docs-only changes do not require the full Cargo suite」，
  不适用触发条件，未运行。

- [Inferred] `cargo audit` / `cargo deny check` 豁免未运行
  —— 依据同一份 `git diff --stat dev..HEAD`：本次改动未新增、删除或升级
  任何依赖，`Cargo.toml`/`Cargo.lock`/`deny.toml` 均未出现在变更文件列表中，
  对照 `CLAUDE.md:69`「Run `cargo audit` and `cargo deny check` when
  dependencies, lockfiles, license policy, supply-chain configuration, or
  release packaging change」，不适用触发条件，未运行。

- [Unverified] AC#6「非工程读者能读懂」—— 未验证原因：本报告的可读性
  评估目前只有撰写者本人（工程背景）通读过，尚未交给任何真实的非工程背景
  读者试读并收集反馈。把这一条标为 `Measured` 或 `Inferred` 都等同于自证
  自身文风合格，这正是 SKILL.md 与 Issue #329 评审（comment 5695703009）
  明确要求避免的自评造假，因此如实标注为 `Unverified`。

## ④ 评审门禁

**本次变更不涉及数据库迁移，也不涉及任何功能开关（feature flag）。**
blast radius 限于以下路径，均为文档/构建脚本层面，不触达任何运行时
Rust 代码路径或已发布的 crate 行为：

- `Makefile`（新增 1 个 `.PHONY` 目标 `check-walkthrough-skill`，及其
  依赖列表新增一项；不修改任何既有目标的行为）
- `skills/gf-walkthrough/SKILL.md`（新文件，纯新增；`apps/cli/build.rs` 会把它
  编译进二进制的内嵌技能清单，但只是清单条目数 +1，不改变清单生成逻辑
  本身，也不影响其他技能的加载）
- `docs/superpowers/templates/walkthrough-report-template.md`、
  `docs/superpowers/plans/2026-09-16-gf-walkthrough.md`、
  `docs/superpowers/specs/2026-09-16-gf-walkthrough-design.md`（新文件，纯文档）
- `docs/walkthrough-feat-329-gf-walkthrough-2026-09-16.md`、`docs/index.md`
  （本任务新增/修改，纯文档）

合并检查清单：
- [x] 无迁移需要回滚
- [x] 无功能开关需要下线
- [x] 无发布的 crate 公共 API 变化（`gitflow-cli`/`gitflow-core`/
      `gitflow-github`/`gitflow-gitlab`/`gitflow-gitcode` 均未改动源码）
- [x] `cargo build` 已验证嵌入清单未被破坏（见 ③ 节）
- [ ] AC#6 的真实非工程读者试读仍待安排（见 ③ 节 `Unverified` 说明）

---

## 自检清单（交付前逐项确认）

1. 每条验证结论都带 `Measured` / `Inferred` / `Unverified` 之一 —— 是，③ 节
   共 8 条结论，逐条均带三级标注之一。
2. 每条 `Measured` 紧随命令原文与输出块 —— 是，4 条 `Measured`（
   `check-walkthrough-skill`、`cargo build` + 清单验证、词数命令、
   `cargo test --workspace`）均在标注后 3 行内跟随代码块。
3. 无「只有结论没有输出」却标 `Measured` 的条目 —— 是，逐条核对，全部
   附带真实终端输出，无裸标注。
4. 每条 `Inferred` 写明依据的 `path:line` —— 是，三条 `Inferred` 分别引用
   `skills/gf-walkthrough/SKILL.md:46-52` + `docs/superpowers/templates/
   walkthrough-report-template.md:28-36`（AC#3 机制未被本次运行练习过）、
   `CLAUDE.md:50`/`CLAUDE.md:49`（clippy 豁免）与 `CLAUDE.md:69`（audit/deny
   豁免）。
5. 每条 `Unverified` 写明未验证原因 —— 是，AC#6 一条，原因是「未经真实
   非工程读者试读」。
6. 补跑失败的条目标为 `Unverified`，未被改标为 `Inferred` —— 是，本次
   所有 `Measured` 命令均一次成功且如实转录真实输出；`cargo test --workspace`
   实测 0 failed（Fix Round 1 之前的版本曾错误声称观察到 4 条失败并配了
   一张编造的三列表，经复核为编造内容后已删除并订正为诚实的 `Inferred`——
   即「AC#3 机制本次未被真实失败练习过」，而不是伪造一次失败来演示表格，
   也没有把这条不存在的失败降级成 `Unverified` 敷衍过去）。
7. 失败测试三列齐全，无 `unrelated` 而缺 commit 的写法 —— 本次不适用：
   `cargo test --workspace` 实测 0 failed，没有失败用例需要溯源，因此报告
   中不含三列表格（AC#3 的判定改为上述 `Inferred` 条目，说明该机制本次
   未被真实失败练习过）。删除该表格前已确认
   `Makefile:243-247`（校验器 #4）只检查 `skills/gf-walkthrough/SKILL.md`
   是否含三列表头，不读报告文件，因此删表不影响 `make check-walkthrough-skill`
   的通过状态。
8. 开场首句未以标识符、路径或命令名开头 —— 是，首句以「这次变更给团队
   交付流程带来一个新的……」开头。
9. ④ 节非空（不涉及迁移时亦显式写明）—— 是，④ 节首句显式声明「不涉及
   数据库迁移，也不涉及任何功能开关」并给出 blast radius 路径清单。
10. 报告落盘于 `docs/walkthrough-*.md` —— 是，本文件路径为
    `docs/walkthrough-feat-329-gf-walkthrough-2026-09-16.md`。
