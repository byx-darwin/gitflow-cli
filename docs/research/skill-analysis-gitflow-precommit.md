# gitflow-precommit Skill 分析报告

> **分析日期：** 2026-07-07
> **分析目标：** `skills/gitflow-precommit/SKILL.md`
> **对应 Issue：** #24
> **分析维度：** 4 个维度（结构规范、职责边界、可测试性、Superpowers 最佳实践）

---

## 一、当前状态总览

| 维度 | 评分 | 说明 |
|------|------|------|
| 维度 1：Skill 结构和文档规范 | ⚠️ 需改进 | description 为功能描述而非触发条件；缺少 Overview / When to Use / Core Pattern / Quick Reference 章节；可写操作步骤（hook 配置）缺少边界声明；token 数量超标（885 词） |
| 维度 2：职责边界清晰度 | ❌ 不合格 | 完全缺失职责边界声明、禁止行为清单和红旗列表；且 skill 涉及文件写入操作（在 `.git/hooks/` 下创建文件），边界缺失风险为 🟡 中 |
| 维度 3：可测试性 | ❌ 不合格 | 完全缺失测试场景、基线测试和成功标准 |
| 维度 4：与 Superpowers 最佳实践的差距 | ⚠️ 需改进 | 工作流程步骤明确可执行；但 description 不合规、无关键词覆盖、无跨引用、未遵循 writing-skills 方法论 |

**总体评估：** gitflow-precommit 是一个"操作教程型 skill"——它告诉 Claude "按这些步骤运行这些命令"，提供了高质量的检查工作流和 hook 脚本模板。但与 Superpowers 要求的 skill 形态差距在于：无法自动判断何时触发、无边界约束（尤其是涉及文件写入操作）、无测试验证机制。

---

## 二、维度 1 分析 — Skill 结构和文档规范

### 2.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| YAML frontmatter 含 name 字段 | ✅ | `name: gitflow-precommit` |
| YAML frontmatter 含 description 字段 | ✅ | 存在 description |
| description 以 "Use when..." 开头 | ❌ | "Pre-commit 检查工作流 — 在提交前运行格式化..."
| description 只描述触发条件 | ❌ | 混合了功能描述（"确保代码质量达标"）和流程描述（"并可选配置 Git pre-commit hook"） |
| 含 Overview 章节 | ⚠️ | 有 H1 标题和简短介绍，但缺少结构化 Overview |
| 含 When to Use 章节 | ❌ | 无触发条件说明 |
| 含 Core Pattern 章节 | ❌ | 无核心模式/算法骨架 |
| 含 Quick Reference | ❌ | 工作流步骤缺少"触发→执行→输出"快速对照表 |
| 含 Implementation 章节 | ✅ | 步骤 1-7 的工作流明确可执行（高质量） |
| 含 Common Mistakes | ⚠️ | 有"注意事项"段落（9 条），接近 Common Mistakes |
| Token 效率 < 500 词 | ❌ | 885 词（`wc -w`），超标约 385 词（77%）|
| 无叙事性示例反模式 | ⚠️ | 步骤 6 嵌入完整 hook 脚本和完整 `.pre-commit-config.yaml`，更接近"复制黏贴模板"而非"指导执行" |
| 无多语言稀释 | ⚠️ | 全中文，未提供英文 description |
| 无流程图中嵌入代码 | ✅ | 无 Mermaid/AST 流程图嵌入 |

### 2.2 具体问题

1. **description 违反 Superpowers 规范**：
   - 当前：`Pre-commit 检查工作流 — 在提交前运行格式化、静态分析和测试，确保代码质量达标，并可选配置 Git pre-commit hook`
   - 问题：这是功能描述 + 流程描述 + 效果承诺，不是触发条件
   - 应为：`Use when the user wants to run pre-commit code quality checks (formatting, linting, testing) before committing, or set up/verify a Git pre-commit hook.`
   - 后果：Claude 无法基于自然语言请求准确判断是否加载此 skill（"帮跑一下检查" vs "提交前应该做什么" vs "配置一下 hook" 都可能是触发）

2. **Token 严重超标（885 词 vs 500 词上限）**：
   - 主因：步骤 6 中嵌入完整 hook 脚本（约 35 行代码）和完整 `.pre-commit-config.yaml`（含注释，约 30 行）
   - 技能设计考虑：skill 中嵌入完整脚本以便 Claude 直接生成，但与"触发→运行→输出"的 skill 职责不符——这些脚本应是 skill 执行时动态选择生成的数据源，而非硬编码在 skill 中
   - 建议：skill 保留 3 行命令速查（`cargo fmt -- --check`、`cargo clippy`、`cargo test --workspace`），hook 模板移至 `docs/templates/pre-commit-hook.sh` 和 `templates/pre-commit-config.yaml`

3. **缺少结构化快速导航**：
   - Superpowers 技能推荐 When to Use + Core Pattern + Quick Reference + Common Mistakes 的组合
   - 当前用户期待的是"告诉我提交前要做什么"的速查，但文档结构是 7 步骤线性教程
   - 建议：添加 Quick Reference 速查表（3 核心命令 + 故障修复命令 + hook 配置命令）

4. **使用示例偏 happy-path**：
   - 步骤 2-4 中失败处理仅在旁边列出命令，未说明"自动修复后重新检查"的标准流程判断
   - `cargo clippy --fix --allow-dirty` 有实际风险（可能引入无关变更），应由作者/用户审查
   - 缺少明确分支：自动修复后应提示用户 diff review

5. **"步骤 6"的内容定位模糊**：
   - 标题"配置 Git pre-commit hook（可选）"看起来可选，但这是一个独立的完整并行场景
   - 它与步骤 1-5（运行检查）不在同一调用路径——用户可能只想运行检查，或只想配置 hook
   - 应在 skill 中明确分岔：运行时检查场景 vs hook 配置场景

### 2.3 评分：⚠️ 需改进

---

## 三、维度 2 分析 — 职责边界清晰度

### 3.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 有明确的职责边界声明 | ❌ | 完全缺失 |
| 有禁止行为清单（🚫 不得...） | ❌ | 缺失 |
| 有职责范围说明（✅ 负责... / ❌ 不负责...） | ❌ | 缺失 |
| 有"合理化借口"反制表格 | ❌ | 缺失 |
| 有红旗列表（Red Flags） | ❌ | 缺失 |

### 3.2 具体问题

1. **无职责边界 → 涉及文件写入操作风险**：
   - 步骤 6 涉及写入 `.git/hooks/pre-commit` 文件和 `pip install` 操作——这些是实际修改用户环境的副作用操作
   - 没有边界声明时，Claude 可能在以下场景过度执行：
     - 用户仅要求"跑一下检查"时，Claude 自动写入 hook 文件并修改 `.pre-commit-config.yaml`
     - 自动将 `--no-verify` 的跳过路径判断为"不需要修复"
     - 在未确认的情况下运行 `cargo clippy --fix --allow-dirty`（修改代码）
     - 将 `pip install pre-commit` 视为理所当然（可能用户环境禁止 pip）

2. **缺少禁止行为清单 — 应明确**：
   - 🚫 不得在用户未明确要求时自动配置 Git hook
   - 🚫 不得在未获得用户批准前运行 `cargo clippy --fix`（修改用户代码）
   - 🚫 不得在未安装前运行 `pip install pre-commit`
   - 🚫 不得为用户提交代码（`git add` / `git commit`）— 仅运行检查
   - 🚫 不得修改 `.pre-commit-config.yaml`、`rustfmt.toml`、`clippy.toml` 等配置文件

3. **缺少红旗信号 — 应标识**：
   - 用户要求"自动修复所有 lint 警告"
   - 用户要求"配置 hook 跳过某些文件的检查"
   - 在 CI 环境中暗示使用此 skill（hook 不适合 CI）
   - 用户说"紧急提交，跳过检查然后用 hook 自动通过"

4. **缺少职责范围说明**：
   - ✅ 负责：解析项目配置、运行 fmt/clippy/test 检查、汇总检查结果、生成报告、按需配置 hook
   - ❌ 不负责：修复代码问题、提交代码、修改 `.git/hooks/`（除非用户明确请求）、安装系统级依赖

5. **"合理化借口"反制（针对 hook 场景）**：
   - "just this once" → "顺带把 hook 写了，以后就方便了" → 无需默认配置
   - "standard practice" → "大家都配了就是好的" → 需用户确认
   - "auto-fix" → "cargo clippy --fix 能自动修，不麻烦" → 需 review diff

### 3.3 评分：❌ 不合格

**对比参考（gitflow-autoreport-bug）：** 该 skill 通过"职责边界声明"章节明确了"修复建议 ≠ 自动修复"的边界。gitflow-precommit 涉及更实际的副作用操作（写入 hook 文件、修改代码的 `--fix`、安装 pip 包），职责边界比 autoreport-bug（仅创建 Issue）更复杂，更需要声明。

---

## 四、维度 3 分析 — 可测试性

### 4.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 定义了测试场景 | ❌ | 无测试场景定义 |
| 有基线测试（无 skill 时的行为） | ❌ | 缺失 |
| 有压力测试场景 | ❌ | 缺失 |
| 有成功标准 | ❌ | 仅有格式化的"结论"输出，未定义为可验证标准 |
| 可使用 writing-skills 方法论测试 | ❌ | 无测试钩子 |

### 4.2 具体问题

1. **零测试覆盖 — 无法验证 Claude 执行质量**：
   - 如何判断 "Claude 正确识别了应触发 pre-commit 检查的场景"？（"提交前帮我检查一下" vs "下一个任务是什么"的区别）
   - 如何判断 "Claude 在未确认时未执行 cargo clippy --fix"？
   - 如何判断 "Claude 在项目不是 Rust 项目时正确拒绝/降级"？
   - 如何判断 "Claude 正确解析了 .pre-commit-config.yaml 并成功复用已有配置"？

2. **缺少基线对比**：
   - 基线行为：用户说"help me prepare to commit" → Claude 可能仅帮助 git add/commit，不知道也要运行 fmt/clippy/test
   - 基线行为：用户说"run the checks" → Claude 可能只运行 `cargo test`（遗漏 fmt 和 clippy）
   - 基线行为：用户说"set up a hook" → Claude 可能提供从零编写的建议，未复用 `.pre-commit-config.yaml`

3. **无压力测试场景**：
   - 超大 workspace（>10 个 crate）运行 `cargo test --workspace` 超时
   - 网络隔离环境无法使用 `pre-commit` Python 包
   - `.pre-commit-config.yaml` 存在但 hook ID 已被修改（自定义配置）
   - 项目同时包含 Rust 和 TypeScript/Go 等多语言项目

4. **无成功标准**：
   - 应定义：完整检查应覆盖哪些工具？（fmt + clippy + test 三维度）
   - 应定义：hook 配置成功的验证标志？（执行后 `.git/hooks/pre-commit` 可执行文件存在 + 手动运行触发）
   - 应定义：检查报告的最低字段（检查项名称、状态、失败原因、修复建议）

5. **未使用 writing-skills 方法论验证**：
   - skill 未经历 baseline → gap analysis → write skill → verify 的 TDD 循环
   - skill 创建时未记录"不用 skill 时的 Claude 差距是什么"，无法衡量 skill 解决了什么问题

### 4.3 评分：❌ 不合格

---

## 五、维度 4 分析 — 与 Superpowers 最佳实践的差距

### 5.1 检查清单

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 遵循 TDD for skills（RED-GREEN-REFACTOR） | ❌ | 无 TDD 流程记录 |
| description 只描述触发条件，不描述流程 | ❌ | description 混合功能描述与流程描述 |
| 关键词覆盖（错误信息、症状、同义词、工具） | ⚠️ | 提到了 `cargo fmt`、`cargo clippy`、`cargo test`、`.git/hooks/pre-commit`、`.pre-commit-config.yaml`、`pip install pre-commit`、`--no-verify` 等工具/文件和选项 |
| 跨引用其他 skills | ❌ | 无 See Also / 相关 Skills |
| 必要时使用 flowchart | ⚠️ | 当前无流程图，但 7 步骤工作流+2 并行路径（运行检查 vs 配置 hook）其实需要流程图指导 |

### 5.2 具体问题

1. **description 应为触发条件，当前是功能描述+流程描述**：
   - ❌ 当前：`Pre-commit 检查工作流 — 在提交前运行格式化、静态分析和测试，确保代码质量达标，并可选配置 Git pre-commit hook`
   - ✅ 应为：`Use when the user runs pre-commit quality checks (fmt, clippy, test), sets up/verifies their pre-commit hook, or their commit is rejected by quality gate checks.`
   - 建议关键词覆盖：用户可能表达为"提交前检查"、"质量检查"、"pre-commit"、"跑 lint"、"cargo fmt check"、"cargo clippy"、"setup pre-commit hook"、"commit failed checks"、"配置 git hook"、"hook 检查失败"

2. **缺少跨引用**：应明确引用：
   - `gitflow-commit`（commit 操作与 pre-commit 检查的对齐）
   - `gitflow-quality`（quality 包含更多维度检查）
   - `gitflow-security-check`（安全也是质量的一部分，可共享环境）
   - `superpowers:test-driven-development`（TDD 循环与 pre-commit 检查的关联）

3. **内容组织偏向"教程文档"而非"执行指令"**：
   - 步骤 1 是"解析 Cargo.toml"——这不是 Claude 需要的指令，而是在描述"Claude 不需要做什么"
   - Superpowers skill 应该用命令式和条件判断，而非旁白式文档
   - 当前结构读起来像给用户阅读的操作手册，而非给 Claude 遵循的执行指令

4. **缺少"双流"分支提示**：
   - 场景 A：执行检查（步骤 1-5 → 输出报告）
   - 场景 B：配置 hook（步骤 6 → 验证可执行权限）
   - 两个场景应使用 if/else 分支进行区分，避免每次加载时给用户展示不相关路径

5. **技术命令的质量优势**：
   - 命令本身准确且经过验证：`cargo fmt -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --workspace`
   - `--allow-dirty` 等高级 flag 显示了作者对 Rust 生态的深入了解
   - 脚本模板实用：`set -euo pipefail`、结构化输出、明确退出码
   - 这些优质内容是重构时应保留的核心价值

### 5.3 评分：⚠️ 需改进

---

## 六、改进建议

### P0（必须修复 — 阻断性问题）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P0-1 | 重写 description 为触发条件 | D1, D4 | description 必须改为 "Use when..." 格式，仅包含触发关键词 |
| P0-2 | 添加职责边界声明章节 | D2 | 明确 hook 配置和 `--fix` 操作需要用户确认，禁止自动修改文件/安装依赖 |
| P0-3 | 添加红旗列表 | D2 | 标识敏感场景：用户要求自动修复 lint、跳过 hook 的 CI 环境、紧急提交等 |
| P0-4 | 添加禁止行为清单 | D2 | 🚫 不得未经确认写入 .git/hooks/pre-commit；🚫 不得运行 cargo clippy --fix；🚫 不得 pip install；🚫 不得为用户 git add/commit |
| P0-5 | 降低 token 数至 < 500 | D1 | 将完整脚本模板移至 `docs/templates/`，skill 仅保留 3 行核心命令 + 分支逻辑 |

### P1（建议修复 — 提升质量）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P1-1 | 重构为结构化模板 | D1 | 添加 Overview / When to Use / Core Pattern / Quick Reference / Implementation / Common Mistakes 章节 |
| P1-1b | 添加 Quick Reference 速查 | D1 | 3 核心命令（fmt + clippy + test）+ 失败修复命令 + hook 配置命令速查表 |
| P1-2 | 添加双流分支结构 | D1, D4 | 场景 A（运行检查）/ 场景 B（配置 hook）使用明确的 if/else 逻辑 |
| P1-3 | 添加关键词覆盖 | D4 | 覆盖中文触发词："提交前检查"、"质量检查"、"跑 lint"、"cargo fmt check"、"cargo clippy"、"配置 git hook"、"pre-commit hook"；覆盖英文触发词："pre-commit checks"、"setup hook"、"git hook" |
| P1-4 | 添加跨引用 | D4 | 引用 gitflow-commit、gitflow-quality、gitflow-security-check、superpowers:test-driven-development |
| P1-5 | 添加基线测试场景 | D3 | 定义不用 skill 时 Claude 的基线行为（可能只运行 cargo test，遗漏 fmt 和 clippy） |
| P1-6 | 补充成功标准 | D3 | 完整检查的 3 维度、hook 验证标志、报告最低字段 |

### P2（可选改进 — 锦上添花）

| # | 改进项 | 维度 | 说明 |
|---|--------|------|------|
| P2-1 | 添加压力测试场景 | D3 | 大型 workspace 超时、pip 不可用、多语言项目混合、自定义 .pre-commit-config.yaml |
| P2-2 | 提供英文版 description | D1 | Superpowers 主流语言为英文，description 改用英文可提高国际兼容性 |
| P2-3 | 添加 TDD for skills 验证记录 | D3, D4 | 记录 baseline → 编写 → 验证迭代的过程 |
| P2-4 | 添加 workflow 流程图 | D4 | 7 步骤 + 2 并行路径用 Mermaid flowchart 简化阅读 |
| P2-5 | 前置 Rust 项目检测 | D1 | 明确：非 Rust 项目时的 graceful degradation（降级至 .pre-commit-config.yaml 运行） |

---

## 七、验收标准

- [ ] description 以 "Use when..." 开头，仅描述触发条件（推荐英文）
- [ ] 含职责边界声明章节（含 🚫 禁止行为、✅/❌ 职责范围）
- [ ] 含红旗列表（要求自动修复 lint、CI 环境、紧急跳过等）
- [ ] 含关键词覆盖（中英触发词、工具名、错误信息）
- [ ] 含跨引用（至少引用 3 个相关 skill）
- [ ] 文档结构包含 Overview / When to Use / Core Pattern / Quick Reference / Common Mistakes
- [ ] 快速加载时 token 数 < 500 词（详细脚本模板移至 docs/templates/）
- [ ] 含双流分支结构（运行检查 vs 配置 hook）
- [ ] 含成功标准定义（3 维度检查清单、hook 验证标志）
- [ ] 含前置条件检查（Rust 非 rust 场景 graceful degradation）
- [ ] 不修改用户文件、不安装依赖、不为用户 commit 未经明确授权

---

## 八、与同类 Skill 对比

| 对比项 | gitflow-precommit | gitflow-autoreport-bug | gitflow-security-check |
|--------|-------------------|----------------------|----------------------|
| 边界风险等级 | 🟡 中（涉及文件写入） | 🟡 中（涉及 Issue 创建） | 🔴 高（涉及敏感数据） |
| 职责边界 | ❌ 缺失 | ✅ 完整 | ❌ 缺失 |
| 测试场景 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 触发条件 | ❌ 功能描述 | ⚠️ 描述流程 | ❌ 功能描述 |
| 跨引用 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| 命令质量 | ✅ 高（多 flag 精确，含高级用法） | ⚠️ 中 | ✅ 高（维度多） |
| description 位置 | ⚠️ 混合功能+流程 | ⚠️ 描述流程 | ❌ 纯功能 |
| Token 数 | ⚠️ 高（885 词） | ⚠️ 中 | ⚠️ 中（~500） |
| 结构化程度 | ⚠️ 线性步骤完整 | ✅ 结构化最佳 | ⚠️ checklist 为主 |

**关键发现：** gitflow-precommit 在工作流步骤的可执行性上是所有 skill 中最强的（步骤 7 步明确，命令精确，含高级 flag 如 `--allow-dirty`），但其结构形态也是"教程文档"与"执行指令"差距最大的——优秀的技术内容需要通过 Superpowers 标准结构才能被 Claude 有效消费。

---

## 九、总结

gitflow-precommit 当前的定位是"操作教程 + 模板库"混合体——它有清晰的工作流步骤和高质量的脚本模板，但缺乏 Superpowers skill 所需的结构性要素。

核心差距：
1. **缺乏触发条件** → Claude 何时应加载此 skill？（用户说"提交前检查"？"配置 hook"？"跑 lint"？）
2. **缺乏职责边界** → 涉及文件写入（hook 配置）和代码修改（`--fix`），边界缺失比 read-only skill 更危险
3. **缺乏可测试性** → 如何验证 Claude 在正确场景触发、在边界内执行、正确生成分支？
4. **token 超标** → 885 词中包含大量脚本模板，应分离到项目模板库

重构方向：保留高质量工作流步骤和命令（重构为 Quick Reference + 分支结构），将脚本模板移至 `docs/templates/`，添加职责边界声明和红旗列表，重写 description 为触发条件，添加跨引用和 Success Criteria。重构后预期 token 从 885 词降至 ~300 词（不含模板文件），大幅提升加载效率。
