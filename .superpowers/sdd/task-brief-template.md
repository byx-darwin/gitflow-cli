### Task N: 分析 SKILL_NAME skill

**Files:**
- Analyze: `skills/SKILL_DIR/SKILL.md`
- Create: GitHub Issue #ISSUE_NUM
- Create: `docs/research/skill-analysis-SKILL_DIR.md`

**Global Constraints:**
- 每个 skill 必须创建独立的 GitHub Issue
- 分析必须覆盖 4 个维度，每个维度给出 ✅/⚠️/❌ 评分
- 改进建议必须分为 P0（必须）、P1（建议）、P2（可选）三个优先级
- 所有文档必须使用 Markdown 格式

- [ ] **Step 1: 读取 skill 文档**

读取 `skills/SKILL_DIR/SKILL.md` 的完整内容。

- [ ] **Step 2: 维度 1 分析 — Skill 结构和文档规范**

检查以下项目并记录：
- YAML frontmatter 是否符合规范（name, description 字段）
- description 是否以 "Use when..." 开头，只描述触发条件
- 文档结构是否完整（Overview, When to Use, Core Pattern, Quick Reference, Implementation, Common Mistakes）
- Token 效率（是否 < 500 词，频繁加载的是否 < 200 词）
- 是否使用了反模式（叙事性示例、多语言稀释、流程图中的代码等）

评分：✅ 优秀 / ⚠️ 需改进 / ❌ 不合格

- [ ] **Step 3: 维度 2 分析 — 职责边界清晰度**

检查以下项目并记录：
- 是否有明确的职责边界声明
- 是否有禁止行为清单（🚫 不得...）
- 是否有职责范围说明（✅ 负责... / ❌ 不负责...）
- 是否有"合理化借口"反制表格
- 是否有红旗列表（Red Flags）

评分：✅ 优秀 / ⚠️ 需改进 / ❌ 不合格

- [ ] **Step 4: 维度 3 分析 — 可测试性**

检查以下项目并记录：
- 是否定义了测试场景
- 是否有基线测试（无 skill 时的行为）
- 是否有压力测试场景
- 是否有成功标准
- 是否可以使用 writing-skills 方法论进行测试

评分：✅ 优秀 / ⚠️ 需改进 / ❌ 不合格

- [ ] **Step 5: 维度 4 分析 — 与 Superpowers 最佳实践的差距**

检查以下项目并记录：
- 是否遵循 TDD for skills（RED-GREEN-REFACTOR）
- description 是否只描述触发条件，不描述流程
- 是否有关键词覆盖（错误信息、症状、同义词、工具）
- 是否有跨引用其他 skills
- 是否有 flowchart（仅在必要时）

评分：✅ 优秀 / ⚠️ 需改进 / ❌ 不合格

- [ ] **Step 6: 创建 GitHub Issue**

使用 gitflow-cli issue create 创建 Issue：

Title: `refactor(skill): SKILL_DIR — 符合 Superpowers 最佳实践`
Labels: `enhancement`, `skill-refactor`

Issue body 必须包含：
- 当前状态评估（4 个维度，每个维度评分 + 问题清单）
- 改进建议（P0/P1/P2 分级）
- 验收标准

- [ ] **Step 7: 提交分析结果**

```bash
git add docs/research/skill-analysis-SKILL_DIR.md
git commit -m "docs: analyze SKILL_DIR skill (#ISSUE_NUM)"
```
