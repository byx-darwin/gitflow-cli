# Phase 4 Dogfooding Checklist — Design Spec

**Issue:** #73
**Date:** 2026-07-10
**Status:** Approved
**适用版本:** v0.6.x+

## Background

v0.6.0 全量测试暴露了一个关键问题：测试环境与真实使用场景脱节。`pr merge` 的 bug 是在实际使用（Dogfooding）中发现的，而不是在测试中（参见 Issue #70）。

本设计将 Dogfooding 纳入 Phase 4 回归测试标准，确保每次发布前都用真实 workflow 场景验证核心命令。

## Goals

- 将 Dogfooding 纳入 Phase 4 回归测试标准
- 每次发布前用真实 workflow 场景验证核心命令
- 覆盖三平台（GitLab/GitHub/GitCode）各至少一个完整 workflow
- 验证交互模式和非交互模式两种场景
- Dogfooding bug 自动记录到 `.cache/bug-reports/pending.json`

## Design Decisions

### 执行模式：手动 Checklist

提供一个 Markdown checklist 文档，由用户手动执行每个 workflow 步骤并勾选确认。适合发布前的手动验证场景。

**理由：** Phase 4 是发布前的最终验证环节，手动执行可以确保每个步骤都被仔细检查，避免自动化脚本可能忽略的边缘情况。

### 测试范围：风险驱动

每个平台根据历史 bug 选择测试重点，而非执行完整的 workflow。

**理由：** 完整的 workflow 耗时较长，而风险驱动策略可以更高效地覆盖最可能出问题的区域。

### Bug 处理：仅记录到 pending.json

发现 bug 时自动追加到 `.cache/bug-reports/pending.json`，并在 workflow 结束时汇总报告。不自动创建 Issue。

**理由：** 给用户决定权，避免自动创建过多 Issue 造成噪音。

### Gate 策略：阻断发布

任何 dogfooding 检查项失败都会阻断 Phase 4 完成，必须修复后重新执行。

**理由：** Dogfooding 的目的是在发布前发现真实场景中的问题，如果发现问题仍然允许发布，就失去了 dogfooding 的意义。

## Architecture

### 文件结构

```
docs/specs/phase4-dogfooding-checklist.md  # 主 checklist 文档
```

### 文档结构

```markdown
# Phase 4 Dogfooding Checklist

## 前置条件
- [ ] 确认当前版本已完成 Phase 1-3
- [ ] 确认三个平台的认证状态正常

## GitHub Dogfooding
### 风险项：release 命令
- [ ] 执行 `gitflow-cli release create v0.x.x`
- [ ] 执行 `gitflow-cli release delete v0.x.x`
- [ ] 验证非交互模式：`echo "y" | gitflow-cli release create v0.x.x`

## GitLab Dogfooding
### 风险项：issue label 中文编码
- [ ] 创建带中文标签的 issue
- [ ] 验证 label CRUD 操作

## GitCode Dogfooding
### 风险项：pr merge 非交互模式（Issue #70）
- [ ] 执行 `gitflow-cli pr merge <n>` 在非交互 shell
- [ ] 验证 `--yes` 标志正确传递

## Bug 记录模板
（发现 bug 时追加到 .cache/bug-reports/pending.json）

## 汇总报告
（执行完成后生成摘要）
```

### Bug 记录流程

当 dogfooding 过程中发现 bug 时：

1. **立即记录**：将 bug 信息追加到 `.cache/bug-reports/pending.json`
2. **记录格式**：
```json
{
  "id": "<uuid>",
  "source": "dogfooding",
  "platform": "gitcode|github|gitlab",
  "command": "pr merge",
  "phase4_checklist_item": "GitCode Dogfooding > pr merge 非交互模式",
  "error_message": "...",
  "steps_to_reproduce": "...",
  "timestamp": "2026-07-10T00:00:00Z",
  "dogfooding_version": "0.6.x"
}
```

3. **不自动创建 Issue**：仅记录到 pending.json，由用户在 Phase 4 结束后决定是否创建 Issue
4. **汇总报告**：Phase 4 结束时，读取 pending.json 中 `source: "dogfooding"` 的条目，生成 dogfooding 专项报告

### Phase 4 集成

**Phase 4 执行顺序：**

```
Phase 4: Post-Delivery Checks
├── Step 1: gitflow-pipeline-analyzer (现有)
├── Step 2: gitflow-issue-triage (现有)
├── Step 3: gitflow-review (现有)
└── Step 4: Dogfooding Checklist (新增) ← 最后执行
```

**Gate 策略：**

- **阻断发布**：任何 dogfooding 检查项失败都会阻断 Phase 4 完成
- **失败处理流程**：
  1. 记录 bug 到 `pending.json`
  2. 标记失败的检查项为 `[✗]`
  3. **停止 Phase 4**，不归档 contract
  4. 提示用户："Dogfooding 发现 N 个 bug，需要修复后重新执行"
  5. 用户修复后，重新执行失败的检查项

**通过条件：**

- 所有平台的 checklist 项都标记为 `[✓]`
- `pending.json` 中没有新增的 `source: "dogfooding"` 且未解决的 bug
- 生成 dogfooding 汇总报告并保存

**与 gitflow-workflow 的集成：**

在 `skills/gitflow-workflow/SKILL.md` 的 Phase 4 步骤中新增引用：

```markdown
4. **[AUTO]** Execute Dogfooding Checklist
   - Reference: `docs/specs/phase4-dogfooding-checklist.md`
   - All items must pass
   - Output: `dogfooding_passed = true/false`
```

Gate 3→4 的条件不变（`pr_url` + `tests_passed`），Phase 4 完成条件新增：`dogfooding_passed = true`。

## Testing Strategy

由于这是流程文档而非代码功能，测试方式为：

1. **文档验证**：
   - 所有 checklist 项的命令语法正确
   - 链接到相关 Issue 和文档有效
   - JSON 模板格式符合 `pending.json` schema

2. **试运行验证**：
   - 在下一次发布前，实际执行一次完整 dogfooding
   - 验证 checklist 的可操作性和完整性
   - 根据执行结果调整 checklist 内容

3. **回归测试**：
   - 每次发现新的 dogfooding bug，将其验证步骤添加到 checklist
   - 定期回顾 `pending.json` 中的 dogfooding bug，更新风险项

## Documentation Maintenance

### 更新时机

- 发现新的平台特定 bug → 添加对应风险项
- 平台命令行为变更 → 更新相关检查项
- 新增平台支持 → 添加新的平台分节

### 版本管理

- 文档头部标注适用版本范围（如 `适用版本: v0.6.x+`）
- 重大变更时更新版本号

### 关联文档

- 链接到 `docs/specs/phase4-coverage-tdd-design.md`（Phase 4 覆盖率设计）
- 链接到 `.cache/bug-reports/pending.json`（bug 记录位置）
- 链接到相关 Issue（如 #70、#73）

## Deliverables

- [ ] `docs/specs/phase4-dogfooding-checklist.md` — 主 checklist 文档
- [ ] 更新 `skills/gitflow-workflow/SKILL.md` — Phase 4 步骤新增 dogfooding
- [ ] 更新 `docs/index.md` — 添加文档索引

## Related

- Issue #70: `pr merge` 非交互式模式失败（Dogfooding 发现）
- Issue #71: E2E 非交互测试框架
- Issue #73: Phase 4 Dogfooding 常态化
- `docs/test-report-gitflow-cli-full-test-2026-07-08.md` — 全量测试报告
