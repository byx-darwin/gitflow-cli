# 2026-08-04 Skill 目录重命名 `gitflow-*` → `gf-*` 设计

**Issue**: [#126](https://github.com/byx-darwin/gitflow-cli/issues/126) · **工作流**: wf-2026-08-04-001 · **模式**: full

## 背景

skills 目录名称 `gitflow-*` 与二进制名 `gf` 不一致。二进制已从 `gitflow-cli` 重命名为 `gf`（见 PR #124/#125），skills 应统一为 `gf-*` 前缀。

## 目标

将所有 `gitflow-*` skill 重命名为 `gf-*`：

- `.claude/skills/gitflow-*` → `.claude/skills/gf-*`（26 个）
- `skills/gitflow-*` → `skills/gf-*`（26 个）
- `docs/references/gitflow-*-params.md` → `docs/references/gf-*-params.md`
- `docs/research/skill-analysis-gitflow-*.md` → `docs/research/skill-analysis-gf-*.md`
- `docs/superpowers/tests/skills/gitflow-*-test.md` → `docs/superpowers/tests/skills/gf-*-test.md`
- 所有内部交叉引用更新（164 个文件，2,279 处引用）
- CLAUDE.md、docs/、specs/、Rust 源码、hooks、scripts 中的 skill 引用更新

## 范围决定

用户确认采用**全量方案**：所有 `gitflow-` skill 引用（包括 `docs/research/`、`docs/superpowers/plans/`、`docs/superpowers/specs/` 历史文档）全部重命名。CHANGELOG.md 中二进制旧名 `gitflow-cli` 保留为历史记录。

## 执行策略

**方案 A：批量重命名 + 一次性审查**（用户确认）

### 步骤

1. **批量重命名目录**（`git mv` 保留历史）
   - `.claude/skills/gitflow-*` → `gf-*`（26 个）
   - `skills/gitflow-*` → `gf-*`（26 个）
   - `docs/references/` 参数文件
   - `docs/research/` 分析报告
   - `docs/superpowers/tests/skills/` 测试文档

2. **全局内容替换**（排除 `.cache/`、`target/`、`.git/`）
   - `sed -i 's/gitflow-<skillname>/gf-<skillname>/g'`，使用 skill 名称白名单模式
   - 应用于 `.md`/`.rs`/`.sh`/`.toml`/`.yaml`/`.yml`/`.json`

3. **手工修复特例**
   - `gitflow-cli`（二进制旧名）在 CHANGELOG/历史记录中保留
   - `specs/gitflow-cli-design.md` 等历史档案文件不重命名
   - SKILL.md 的 `description:` 触发词字段确认更新

4. **验证**
   - `make build` + `make test`
   - `gf skills list`（退出标准）
   - `grep -r "gitflow-"` 确认无残留（历史记录除外）

### 防误伤规则

| 模式 | 处理 |
|------|------|
| `gitflow-<skillname>`（26 个白名单 skill 名） | ✅ 替换为 `gf-<skillname>` |
| `gitflow-cli`（GitHub 仓库 URL / 二进制旧名） | ⚠️ **绝不改动** —— 在 README、Rust 源码、CHANGELOG 中是 `byx-darwin/gitflow-cli` 仓库地址或历史发版记录 |
| `specs/gitflow-cli-design.md` 等历史档案文件 | 不重命名 |
| JSON fixture 测试数据 | ✅ 随内容替换，保持测试一致性 |

**Rust 源码现状**：`crates/` 与 `apps/cli/src/` 中所有 `gitflow-` 引用均为 `gitflow-cli` 仓库 URL，无 skill 名引用，**无需改动**。skill 名引用集中在 SKILL.md、docs/、hooks/、CLAUDE.md、scripts/ 等文档与配置文件中。

## 验证（退出标准映射）

| 退出标准 | 验证方式 |
|----------|----------|
| `gf skills list` 正常 | 运行命令，确认列出 26 个 `gf-*` skill |
| 所有 skill 触发词正常工作 | 检查 SKILL.md 的 `description` 字段 |
| 文档无残留 `gitflow-` skill 名 | `grep -r "gitflow-"` 检查（允许 CHANGELOG 等历史记录） |

## 风险与回滚

- **风险**：sed 误伤非 skill 引用 → 用 skill 名称白名单模式规避
- **回滚**：目录用 `git mv`（保留历史），文件替换用 `git checkout` 还原

## 测试

- Rust 测试：`apps/cli/tests/workflow_*_test.rs` 引用 skill 名称需同步更新
- CLI 测试：`apps/cli/tests/common/mod.rs`
- E2E：`gf skills list` 验证
- 文档验证：`make check-agent-sync`（AGENTS/CLAUDE/skill 编辑）
