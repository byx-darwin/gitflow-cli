# GEO 生成式引擎优化增强设计文档

- **日期**：2026-08-09
- **状态**：待评审
- **关联 Issue**：#98（feat(roadmap): [阶段二·第9-10周] GEO 生成式引擎优化）
- **关联工作流**：`wf-2026-08-09-098`（full 模式四阶段编排）
- **上游依据**：`docs/superpowers/specs/2026-07-31-product-evaluation-roadmap-design.md` §6.2 + §7.3

---

## 1. 背景与范围

Issue #98 是路线图阶段二（增长）第 9-10 周的工作集合，目标是让 AI 搜索引擎（Perplexity / ChatGPT / Kimi / 豆包）在回答"有哪些 Git 工作流工具 / AI 编程工程化"时准确引用 gitflow-cli。

前置 Issue #97 已建立 GEO 地基：
- `llms.txt`（23 行）+ `llms-full.txt`（61 行）
- `SoftwareApplication` JSON-LD 标注
- 官网骨架（6 个页面）

本次工作在现有地基上进行**系统化增强**，建立可持续维护的 GEO 基础设施。

### 1.1 本次范围（In Scope）

| # | 任务 | 说明 |
|---|------|------|
| 1 | `llms-full.txt` 模块化重构 | 拆分为索引 + 三个专门模块（commands / architecture / faq） |
| 2 | 实体一致性修复 | 更新 GitHub description 为规范一句话定位 |
| 3 | JSON-LD 生成系统 | 构建期从数据源自动生成 SoftwareApplication / FAQPage / HowTo |
| 4 | 对比页面 | `/compare/`（gf vs gh/glab）+ `/what-is-ai-workflow/`（概念介绍） |
| 5 | 工作流文档页面 | `/workflow/`（gf-workflow 完整参考） |
| 6 | 实体一致性守护测试 | Rust + TypeScript 双重检查 |
| 7 | 月度抽检文档 | `docs/geo-citation-check.md` 记录流程 |

### 1.2 明确不做（Out of Scope）

| 任务 | 原因 |
|------|------|
| SEO 搜索引擎收录提交 | 属 Issue #99（第 11-12 周） |
| 内容矩阵（掘金/知乎文章） | 属 Issue #99 |
| `gitflow doctor` 环境自检 | 属 Issue #100（第 13-14 周） |
| 官网整体重构 | 当前增量增强足够，避免过度工程化 |

---

## 2. 设计决策

### 2.1 架构选型：方案 B（全面重构）

**决策**：采用方案 B（全面重构），而非方案 A（增量增强）或方案 C（最小可行）。

**理由**：
- 模块化 llms 文件更易维护和扩展
- JSON-LD 生成系统避免手工维护的结构化数据错误
- 守护测试防止实体一致性退化
- 虽然工作量增加 ~2 倍，但长期收益显著

**权衡**：
- ✅ 自动化程度高，减少手工错误
- ✅ 可扩展性强，未来新增 FAQ/HowTo 只需编辑 JSON
- ❌ 需要创建新的构建脚本和测试
- ❌ 初期工作量较大

### 2.2 规范一句话定位（全渠道逐字一致）

**决策**：所有渠道使用以下规范文案，不得改写：

```
跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。
```

**使用位置**：
- GitHub 仓库 description
- `apps/cli/Cargo.toml` 的 `description` 字段（首句）
- `website/public/llms.txt` 和 `llms-full.txt`
- `website/src/lib/jsonld.ts` 的 `CANONICAL_POSITIONING` 常量
- 所有页面的 meta description（除非页面有特定描述）

**守护机制**：
- Rust 测试 `test_should_keep_canonical_positioning_in_*`
- TypeScript 测试检查 JSON-LD 生成器输出

### 2.3 llms 文件模块化

**决策**：将 `llms-full.txt` 重构为索引 + 三个专门模块。

| 文件 | 内容 | 预计行数 |
|------|------|---------|
| `llms.txt` | 摘要（保持不变） | ~23 行 |
| `llms-full.txt` | 索引 + 快速摘要 | ~50 行 |
| `llms-commands.txt` | 完整命令参考（所有命令、标志、选项） | ~200 行 |
| `llms-architecture.txt` | 架构说明（Platform trait、Crate 结构、依赖关系） | ~80 行 |
| `llms-faq.txt` | FAQ 集合（长尾提问解答） | ~70 行 |

**设计原则**：
- 索引 + 模块：`llms-full.txt` 作为索引，指向三个专门模块
- 结构化内容：每个模块有清晰的 H1/H2 层级，便于 AI 解析
- 可独立引用：AI 可直接链接到 `llms-commands.txt` 获取命令参考

### 2.4 JSON-LD 生成系统

**决策**：构建期从数据源自动生成 JSON-LD，而非硬编码。

**数据源**：

| 数据源 | 提取信息 |
|--------|---------|
| `Cargo.toml` (`[workspace.package]`) | 名称、版本、作者、许可证、仓库 URL |
| 页面 frontmatter | 页面标题、描述、类型 |
| `website/src/data/faq.json` | FAQ 问答对 |
| `website/src/data/howto.json` | 操作指南步骤 |

**生成器架构**：

```
website/
├── src/
│   ├── data/
│   │   ├── faq.json        # FAQ 数据
│   │   └── howto.json      # HowTo 数据
│   └── lib/
│       └── jsonld.ts       # JSON-LD 生成器
```

**生成的 JSON-LD 类型**：
- `SoftwareApplication`：所有页面
- `FAQPage`：首页 + FAQ 相关页面
- `HowTo`：快速上手页面 + 工作流页面

**设计原则**：
- 数据驱动：JSON-LD 从 JSON 数据文件生成
- 类型安全：TypeScript 接口定义 JSON-LD 结构
- 常量守护：规范一句话定位作为常量
- 可扩展：新增 FAQ 或 HowTo 只需编辑 JSON 文件

### 2.5 对比页面

**决策**：创建两个独立页面，各自有明确的 URL 和 SEO 目标。

| 页面 | URL | SEO 关键词 | GEO 长尾提问 |
|------|-----|-----------|-------------|
| 对比页 | `/compare/` | `gf vs gh`、`多平台 git 工具` | `gf 和 gh 有什么区别` |
| 概念页 | `/what-is-ai-workflow/` | `AI 编程工程化`、`AI coding workflow` | `什么是 AI 编程工程工作流` |

**内容形式**：技术对比表 + 叙述结合

**`/compare/` 结构**：
1. 对比表格（核心）
2. 叙述段落（差异化价值）
3. 迁移指南（从 gh 迁移到 gf）

**`/what-is-ai-workflow/` 结构**：
1. 问题陈述（AI 编程缺乏工程纪律）
2. 核心概念（AI 助手 + 结构化流程 + 质量门禁）
3. 四阶段模型（链接到 `/workflow/`）
4. 两种技能来源（superpowers / mattpocock）
5. 为什么需要工程纪律
6. 如何开始

### 2.6 工作流文档页面

**决策**：新增 `/workflow/` 页面，作为 gf-workflow 的完整参考。

**URL**：`/workflow/`  
**SEO 关键词**：`gf-workflow`、`AI 编程工作流`、`四阶段工作流`

**页面结构**：
1. 四阶段模型（需求澄清 → 计划制定 → 执行 → 交付检查）
2. 三种工作流模式（full / standard / fast）
3. 两种技能来源（superpowers / mattpocock）
4. 三种执行模式（background agent / manual new window / same-session）
5. 合同（Contract）机制
6. 快速开始

**与 `/what-is-ai-workflow/` 的关系**：
- `/what-is-ai-workflow/` 介绍概念，链接到 `/workflow/` 获取详细说明
- `/workflow/` 是完整参考，包含所有模式和技术细节

### 2.7 实体一致性守护测试

**决策**：Rust + TypeScript 双重检查。

**Rust 守护测试**：`apps/cli/tests/geo_guard_test.rs`
- 检查 `llms.txt` / `llms-full.txt` / `apps/cli/Cargo.toml` / `jsonld.ts` 中的规范定位
- 检查模板占位符（`Your Name`、`{{version}}` 等）

**TypeScript 测试**：`website/tests/geo-consistency.test.ts`
- 检查 JSON-LD 生成器输出
- 验证 FAQPage / HowTo JSON-LD 结构

**CI 集成**：`cargo test` 和 `vitest` 均会执行这些测试。

### 2.8 月度 AI 引用抽检

**决策**：创建文档化流程，而非自动化工具。

**文件**：`docs/geo-citation-check.md`

**内容**：
- 目标关键词列表（中英文各 3-5 个）
- 抽检平台（Perplexity / ChatGPT / Kimi / 豆包）
- 记录模板（日期 / 平台 / 关键词 / 是否引用 / 准确性评分）
- 执行频率（月度）

**理由**：
- 低成本，人工执行，可持续
- 自动化工具难以判断引用准确性
- 文档化流程可追溯

---

## 3. 文件清单

| 文件 | 动作 | 说明 |
|------|------|------|
| `website/public/llms-full.txt` | Modify | 重构为索引 |
| `website/public/llms-commands.txt` | Create | 完整命令参考 |
| `website/public/llms-architecture.txt` | Create | 架构说明 |
| `website/public/llms-faq.txt` | Create | FAQ 集合 |
| `website/src/data/faq.json` | Create | FAQ 数据 |
| `website/src/data/howto.json` | Create | HowTo 数据 |
| `website/src/lib/jsonld.ts` | Create | JSON-LD 生成器 |
| `website/src/layouts/Base.astro` | Modify | 集成 JSON-LD 生成器 |
| `website/src/pages/compare.astro` | Create | 对比页面 |
| `website/src/pages/what-is-ai-workflow.astro` | Create | 概念页面 |
| `website/src/pages/workflow.astro` | Create | 工作流文档页面 |
| `apps/cli/tests/geo_guard_test.rs` | Create | Rust 守护测试 |
| `website/tests/geo-consistency.test.ts` | Create | TypeScript 一致性测试 |
| `docs/geo-citation-check.md` | Create | 抽检流程文档 |
| GitHub 仓库 description | Update | 更新为规范一句话定位 |

---

## 4. 退出标准

| 标准 | 验证方式 |
|------|---------|
| `llms-full.txt` 模块化完成 | 文件存在且可访问 |
| 实体一致性修复 | GitHub description 已更新 |
| JSON-LD 生成系统工作 | 构建后页面包含 FAQPage + HowTo JSON-LD |
| 对比页面发布 | `/compare/` 和 `/what-is-ai-workflow/` 可访问 |
| 工作流文档页面发布 | `/workflow/` 可访问 |
| 守护测试通过 | `cargo test` 和 `vitest` 均通过 |
| 抽检文档完成 | `docs/geo-citation-check.md` 存在 |

---

## 5. 风险与应对

| 风险 | 概率 | 应对 |
|------|------|------|
| JSON-LD 生成器引入构建错误 | 中 | TypeScript 类型检查 + 构建前测试 |
| 模块化 llms 文件链接失效 | 低 | 守护测试检查文件存在性 |
| 规范定位拼写错误 | 低 | 常量 + 守护测试 |
| 工作量超出预期 | 中 | 优先完成核心交付物（llms + JSON-LD + 对比页），工作流页面可延期 |

---

## 6. 实施顺序

1. **llms 文件模块化**（~2 小时）
2. **JSON-LD 生成系统**（~2 小时）
3. **对比页面 + 工作流页面**（~4 小时）
4. **实体一致性守护测试**（~1 小时）
5. **GitHub description 更新**（~10 分钟）
6. **抽检文档**（~30 分钟）
7. **集成测试 + 文档更新**（~1 小时）

**总计**：~10.5 小时

---

## 7. 成功指标

| 指标 | 基线（当前） | 目标（3 个月后） |
|------|------------|-----------------|
| AI 搜索引用数（月度抽检） | 0 | ≥ 3 次/月 |
| 引用准确性评分 | N/A | ≥ 4/5 |
| llms-full.txt 行数 | 61 | ~400 |
| JSON-LD 类型 | 1（SoftwareApplication） | 3（+ FAQPage + HowTo） |
| 对比页面数 | 0 | 3（/compare/ + /what-is-ai-workflow/ + /workflow/） |

---

## 8. 参考

- 上游路线图：`docs/superpowers/specs/2026-07-31-product-evaluation-roadmap-design.md`
- Phase 1 设计文档：`docs/superpowers/specs/2026-07-31-v1.0-metadata-website-geo-design.md`
- GEO 执行清单：§7.3
- 工作流技能源码：`skills/gf-workflow/SKILL.md`
