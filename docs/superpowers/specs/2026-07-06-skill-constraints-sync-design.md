# Skill 约束自动同步机制设计

**日期**：2026-07-06
**状态**：设计完成
**作者**：gitflow-cli team

---

## 背景

当前问题：
- CLAUDE.md 中的 skill 约束需要手动维护
- skill 更新后，CLAUDE.md 容易忘记同步
- 导致 skill 定义和 CLAUDE.md 约束不一致

目标：
- 实现 skill 约束的自动检查和同步机制
- 确保 CLAUDE.md 与 skill 定义保持一致
- 保持用户对 CLAUDE.md 的控制权

---

## 设计方案

### 核心思路

采用"检查 + 提示 + 手动同步"的简单方案：
1. Skill 可以包含可选的 `CONSTRAINTS.md` 文件
2. `skills install` 检查约束一致性
3. 不一致时输出警告并提示
4. 提供 `skills sync-constraints` 命令自动更新

---

## 文件结构

### Skill 目录
```
skills/gitflow-workflow/
├── SKILL.md           ← skill 主内容
└── CONSTRAINTS.md     ← CLAUDE.md 约束片段（可选）
```

### CLAUDE.md 结构
```markdown
# CLAUDE.md

## Non-negotiables
（手动维护的内容）

<!-- SKILLS_CONSTRAINTS_START -->
## Skill Constraints (Auto-generated)

### gitflow-workflow
- 规则 1
- 规则 2

### another-skill
- 规则 3
<!-- SKILLS_CONSTRAINTS_END -->

## Other sections
（手动维护的内容）
```

---

## 安装流程

```
gitflow-cli skills install [--force]
  │
  ├─ 安装 SKILL.md 到 .claude/skills/
  │
  ├─ 安装 hook 脚本和配置
  │
  └─ 检查约束一致性
      │
      ├─ CONSTRAINTS.md 不存在 → 跳过
      │
      ├─ CLAUDE.md 不存在
      │   └─ 创建并添加标记 + 约束内容
      │
      ├─ 标记不存在
      │   └─ 追加到文件末尾
      │
      ├─ 约束一致
      │   └─ 输出 ✅ 约束已同步
      │
      └─ 约束不一致
          └─ 输出 ⚠️ 警告 + 提示运行 sync-constraints
```

---

## 同步命令

### 命令定义
```bash
gitflow-cli skills sync-constraints
```

### 执行流程
```
1. 扫描 skills/ 目录下所有 skill
2. 读取每个 skill 的 CONSTRAINTS.md（如果存在）
3. 读取 CLAUDE.md
4. 替换标记区域内的内容
5. 写回 CLAUDE.md
6. 输出更新结果
```

### 输出示例
```
✅ 已同步 3 个 skill 的约束：
  - gitflow-workflow（更新）
  - gitflow-quality（新增）
  - gitflow-pr-review（无变化）

CLAUDE.md 已更新
```

---

## 约束合并策略

### 合并规则
1. 保留 `<!-- SKILLS_CONSTRAINTS_START -->` 之前的内容
2. 替换标记区域内的内容
3. 保留 `<!-- SKILLS_CONSTRAINTS_END -->` 之后的内容

### 内容格式
```markdown
### <skill-name>
<CONSTRAINTS.md 的内容>
```

多个 skill 的约束按字母顺序排列。

---

## 错误处理

| 场景 | 处理方式 |
|------|---------|
| CONSTRAINTS.md 不存在 | 跳过，不报错 |
| CLAUDE.md 不存在 | 创建并添加标记 + 约束内容 |
| 标记不存在 | 追加到文件末尾并添加标记 |
| 读取 CONSTRAINTS.md 失败 | 输出错误，继续处理其他 skill |
| 写入 CLAUDE.md 失败 | 输出错误，回滚变更 |

---

## 实现细节

### 新增模块

**`apps/cli/src/commands/skills.rs`**
- 添加 `sync_constraints()` 函数
- 添加 `check_constraints()` 函数
- 修改 `install()` 函数，添加约束检查

**`crates/core/src/constraints.rs`**（新文件）
- 约束解析和合并逻辑
- 标记检测和内容替换

### 关键函数

```rust
/// 检查 skill 约束是否与 CLAUDE.md 一致
fn check_constraints(skill_name: &str, constraints_path: &Path) -> Result<bool>

/// 同步所有 skill 的约束到 CLAUDE.md
fn sync_constraints(skills_dir: &Path, claude_md_path: &Path) -> Result<()>

/// 合并约束内容到 CLAUDE.md
fn merge_constraints(claude_md: &str, constraints: &HashMap<String, String>) -> String
```

---

## 测试策略

### 单元测试
- ✅ 约束解析：读取 CONSTRAINTS.md
- ✅ 标记检测：识别 CLAUDE.md 中的标记
- ✅ 内容合并：替换标记区域内的内容
- ✅ 一致性检查：比较约束是否相同

### 集成测试
- ✅ 安装流程：install 时检查约束
- ✅ 同步命令：sync-constraints 更新 CLAUDE.md
- ✅ 错误处理：文件不存在、权限错误等

### 端到端测试
- ✅ 完整流程：安装 → 检查 → 同步
- ✅ 多 skill 场景：多个 skill 同时更新

---

## 使用示例

### 场景 1：安装 skill（约束已同步）
```bash
$ gitflow-cli skills install
✅ 已安装: gitflow-workflow
✅ 约束已同步: gitflow-workflow
```

### 场景 2：安装 skill（约束需要更新）
```bash
$ gitflow-cli skills install --force
♻ 已覆盖: gitflow-workflow
⚠️  约束已更新: gitflow-workflow

运行以下命令同步到 CLAUDE.md：
  gitflow-cli skills sync-constraints
```

### 场景 3：同步约束
```bash
$ gitflow-cli skills sync-constraints
✅ 已同步 2 个 skill 的约束：
  - gitflow-workflow（更新）
  - gitflow-quality（新增）

CLAUDE.md 已更新
```

---

## 未来扩展

### 可能的增强
1. **Diff 预览**：同步前显示变更内容
2. **选择性同步**：只同步部分 skill 的约束
3. **备份机制**：同步前备份 CLAUDE.md
4. **版本控制**：记录约束的版本号

### 不在当前范围
- 自动修改 CLAUDE.md（安装时）
- 交互式选择
- 约束的版本管理

---

## 验收标准

- [ ] Skill 可以包含可选的 CONSTRAINTS.md 文件
- [ ] `skills install` 检查约束一致性
- [ ] 不一致时输出警告并提示
- [ ] `skills sync-constraints` 命令可用
- [ ] 同步命令正确更新 CLAUDE.md
- [ ] 保留 CLAUDE.md 中手动维护的内容
- [ ] 完整的测试覆盖
- [ ] 文档更新

---

## 参考资料

- [CLAUDE.md 当前结构](../../../CLAUDE.md)
- [gitflow-workflow skill](../../../skills/gitflow-workflow/)
- [skills install 实现](../../../apps/cli/src/commands/skills.rs)
