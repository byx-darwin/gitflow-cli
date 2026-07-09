# TOON 输出格式设计文档

**日期**: 2026-07-09
**状态**: 待实现

## 1. 背景与目标

### 问题
gitflow-cli 当前支持 JSON 和 Text 两种输出格式。JSON 格式虽然通用，但存在冗余（引号、大括号、重复键名），在 LLM 场景下消耗更多 token。

### 目标
新增 TOON（Token-Oriented Object Notation）输出格式，实现：
- 结构化数据 token 减少 15-40%
- 均匀数组场景 token 减少 50-60%
- 保持向后兼容

## 2. 需求澄清结果

| 决策项 | 选择 |
|--------|------|
| TOON 实现 | 使用外部 crate `toon-format` |
| 集成方式 | `--output toon` + `--output auto` |
| 编码范围 | 保留整个信封结构（CliOutput<T>） |
| auto 策略 | 启发式规则（数据结构分析） |

## 3. 架构设计

### 3.1 模块结构

```
crates/core/src/toon.rs (新增)
  ├── JsonShape 结构体
  ├── TopLevel 枚举
  ├── ToonStrategy 枚举
  ├── analyze() 函数
  ├── select_strategy() 函数
  └── encode() 函数

apps/cli/src/main.rs (修改)
  └── OutputFormat 枚举新增 Toon, Auto 变体

apps/cli/src/commands/output.rs (修改)
  └── print_output 新增 Toon, Auto 分支
  └── 新增 print_toon(), print_auto() 函数

Cargo.toml (修改)
  └── 新增 toon-format 依赖
```

### 3.2 核心类型

```rust
/// JSON 数据的结构特征
pub struct JsonShape {
    /// 顶层类型（Object / Array / Scalar）
    pub top_level: TopLevel,
    /// 数组元素数量（如果顶层是数组）
    pub item_count: usize,
    /// 数组中对象是否都有相同的 key（均匀数组）
    pub is_uniform_array: bool,
    /// 最大嵌套深度
    pub max_depth: usize,
    /// 总字符数（用于判断是否值得转换）
    pub char_count: usize,
}

pub enum TopLevel {
    Object,
    Array,
    Scalar,
}

/// TOON 编码策略
pub enum ToonStrategy {
    /// 使用 TOON 格式（均匀数组或深度嵌套）
    Toon,
    /// 保持 JSON 格式（小数据或不适合 TOON 的结构）
    Json,
}
```

### 3.3 核心函数

#### 结构分析器

```rust
/// 分析 JSON 数据的结构特征（O(n) 单次遍历）
pub fn analyze(value: &serde_json::Value) -> JsonShape
```

遍历 JSON，收集：
- 顶层类型
- 数组元素数量和均匀性
- 嵌套深度
- 总字符数

#### 策略选择器

```rust
/// 根据结构特征选择编码策略
pub fn select_strategy(shape: &JsonShape) -> ToonStrategy
```

选择规则：
1. `char_count < 200` → Json（不值得转换）
2. 顶层是数组 && 均匀 && `item_count >= 5` → Toon
3. `max_depth > 3` → Toon
4. 其他 → Json

#### TOON 编码

```rust
/// 将 serde_json::Value 编码为 TOON 格式
pub fn encode(value: &serde_json::Value) -> Result<String, ToonError>
```

使用 `toon-format` crate 的 `encode_default` 函数。

### 3.4 CLI 集成

#### OutputFormat 枚举扩展

```rust
pub enum OutputFormat {
    Json,   // 现有：JSON 格式
    Text,   // 现有：文本格式
    Toon,   // 新增：强制 TOON 格式
    Auto,   // 新增：根据数据自动选择
}
```

#### print_output 扩展

```rust
pub fn print_output<T: serde::Serialize>(
    value: &T,
    format: &OutputFormat
) -> miette::Result<()> {
    match format {
        OutputFormat::Json => print_json(value),
        OutputFormat::Text => print_text(value),
        OutputFormat::Toon => print_toon(value),
        OutputFormat::Auto => print_auto(value),
    }
}
```

## 4. TOON 格式示例

### 4.1 均匀数组（HRV 格式）

**输入 JSON**:
```json
{
  "success": true,
  "data": [
    {"number": 1, "title": "修复bug", "state": "open"},
    {"number": 2, "title": "新功能", "state": "closed"},
    {"number": 3, "title": "优化性能", "state": "open"},
    {"number": 4, "title": "更新文档", "state": "open"},
    {"number": 5, "title": "添加测试", "state": "open"}
  ],
  "platform": "github",
  "command": "issue list"
}
```

**输出 TOON**:
```
success: true
platform: github
command: issue list
data: items[5]{number,title,state}:
  1,\修复bug,open
  2,\新功能,closed
  3,\优化性能,open
  4,\更新文档,open
  5,\添加测试,open
```

### 4.2 嵌套对象

**输入 JSON**:
```json
{
  "success": true,
  "data": {
    "number": 42,
    "title": "重构模块",
    "author": {
      "login": "alice",
      "email": "alice@example.com"
    }
  },
  "platform": "github",
  "command": "pr view"
}
```

**输出 TOON**:
```
success: true
platform: github
command: pr view
data:
  number: 42
  title: \重构模块
  author:
    login: alice
    email: alice@example.com
```

## 5. 测试策略

### 5.1 单元测试（`crates/core/src/toon.rs`）

| 测试名称 | 验证内容 |
|---------|---------|
| `test_analyze_uniform_array` | 均匀数组检测 |
| `test_analyze_mixed_array` | 混合数组检测 |
| `test_analyze_deep_nesting` | 深层嵌套检测 |
| `test_analyze_small_data` | 小数据检测 |
| `test_strategy_small_data_json` | < 200 字符 → JSON |
| `test_strategy_uniform_array` | 均匀数组 ≥5 → TOON |
| `test_strategy_deep_nesting` | 深度 > 3 → TOON |
| `test_strategy_default_json` | 其他 → JSON |
| `test_toon_encode_simple` | 简单对象编码 |
| `test_toon_encode_array` | 数组编码（HRV 格式） |

### 5.2 集成测试（`apps/cli/tests/toon_output_test.rs`）

| 测试名称 | 验证内容 |
|---------|---------|
| `test_issue_list_toon_output` | `--output toon` 输出验证 |
| `test_issue_list_auto_output` | `--output auto` 自动选择 |
| `test_pr_view_toon_output` | PR 查看 TOON 输出 |

## 6. 依赖变更

### 新增依赖

```toml
# crates/core/Cargo.toml
[dependencies]
toon-format = "0.5"
```

## 7. 实现步骤

1. **添加依赖**: 在 `crates/core/Cargo.toml` 添加 `toon-format`
2. **实现核心模块**: 创建 `crates/core/src/toon.rs`
   - 实现 `JsonShape`, `TopLevel`, `ToonStrategy`
   - 实现 `analyze()`, `select_strategy()`, `encode()`
   - 编写单元测试
3. **扩展 CLI**: 修改 `apps/cli/src/main.rs`
   - `OutputFormat` 枚举新增 `Toon`, `Auto`
4. **集成输出**: 修改 `apps/cli/src/commands/output.rs`
   - 新增 `print_toon()`, `print_auto()`
   - 更新 `print_output()` match 分支
5. **编写集成测试**: 创建 `apps/cli/tests/toon_output_test.rs`
6. **验证**: 运行 `make test`, `make clippy`, `make fmt`

## 8. Skills 关联

### 8.1 受影响的 Skills

以下 skills 调用 `gitflow-cli` 命令，**默认使用 `--output auto`** 以优化 token 消耗：

| Skill | 当前用法 | 更新后 |
|-------|---------|--------|
| `gitflow-workflow` | `issue list --state open --output json` | `issue list --state open --output auto` |
| `gitflow-issue` | `issue list/view` 默认 JSON | `issue list/view --output auto` |
| `gitflow-pr` | `pr view/list` 默认 JSON | `pr view/list --output auto` |
| `gitflow-release` | `release list/view` 默认 JSON | `release list/view --output auto` |
| `gitflow-issue-triage` | `issue list` 默认 JSON | `issue list --output auto` |

### 8.2 文档更新策略

**Phase 1: CLI 层面**
- 所有 `gitflow-cli --help` 自动包含新的 `--output toon/auto` 选项
- 更新所有 skills 的命令示例，将 `--output json` 改为 `--output auto`

**Phase 2: Skill 文档**
- 在 skills 的 SKILL.md 中更新命令示例：
  ```markdown
  ## 命令示例

  gitflow-cli issue list --state open --output auto  # 自动选择最优格式
  ```

### 8.3 向后兼容

- CLI 默认仍为 `--output json`（保持向后兼容）
- **Skills 层面默认使用 `--output auto`**（优化 token）
- 用户可显式指定 `--output json/toon/text` 覆盖

## 9. 风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| `toon-format` crate 不稳定 | 版本锁定 0.5，测试覆盖充分 |
| TOON 解码不兼容 | 仅用于输出，不要求可逆 |
| 性能开销 | 结构分析 O(n)，编码由 crate 优化 |

## 10. 验收标准

- [ ] `gitflow-cli issue list --output toon` 输出 TOON 格式
- [ ] `gitflow-cli issue list --output auto` 自动选择最优格式
- [ ] 均匀数组（≥5 元素）使用 TOON HRV，token 减少 ≥50%
- [ ] 小数据（<200 字符）使用 JSON，避免转换开销
- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] `cargo clippy` 无警告
- [ ] `cargo fmt` 格式化通过
- [ ] `gitflow-cli --help` 显示新的 `--output toon/auto` 选项
- [ ] Skills 文档更新：所有命令示例使用 `--output auto`
- [ ] gitflow-workflow SKILL.md 更新 `--output json` → `--output auto`
