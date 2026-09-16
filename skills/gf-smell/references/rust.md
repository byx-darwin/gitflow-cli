# Rust Detection Layer

**Detection:** `Cargo.toml` at project root (per `gf-quality/references/detector.md`).

## 检测命令

**单次运行，捕获输出后复用。** 重复调用会命中 cargo 缓存并返回空输出，把「缓存」
误报成「干净」——本仓库实测踩中过。

```bash
OUT=$(mktemp -t gf-smell-rust)
cargo clippy --workspace --all-targets 2>&1 -- \
  --force-warn clippy::too_many_lines \
  --force-warn clippy::cognitive_complexity \
  --force-warn clippy::excessive_nesting \
  --force-warn clippy::too_many_arguments \
  --force-warn clippy::type_complexity \
  --force-warn dead_code \
  > "$OUT" 2>&1
```

**必须用 `--force-warn`，不能用 `-W`。** `-W` 无法穿透源码里的 `#[allow(...)]`，
而被 allow 掉的复杂度诊断恰恰是最需要看见的——「已被 allow」不等于问题消失，
只等于有人决定不看它。实测：本仓库以 `-W` 运行全工作区零命中，改 `--force-warn`
后命中 5 条。

结构扫描（clippy 覆盖不到的文件级与模块级信号），排除目录与 Stage 0 一致：

```bash
FILES=$(find . -name '*.rs' \
  -not -path './target/*' -not -path './.worktree/*' \
  -not -path './.claude/worktrees/*' -not -path './vendor/*')

# 文件行数
echo "$FILES" | xargs wc -l | sort -rn | head -20

# 每个 impl 块的函数数量（God Structure 信号）
echo "$FILES" | xargs grep -c '^\s*\(pub \)\?\(async \)\?fn ' | sort -t: -k2 -rn | head -20

# 模块扇出（一个文件引入多少个 crate 内部模块）
echo "$FILES" | xargs grep -c '^use crate::' | sort -t: -k2 -rn | head -20

# 无理由的抑制（前一行无注释的 allow）
echo "$FILES" | xargs grep -n -B1 '#\[allow(clippy::' | grep -A1 -- '--' | head -40

# 缩进嵌套深度（Deep Nesting 结构信号——clippy::excessive_nesting 未配置阈值时的替代）
echo "$FILES" | xargs -I{} awk '
  { depth += gsub(/\{/, "{") - gsub(/\}/, "}"); if (depth > max) max = depth }
  END { print max, FILENAME }
' {} | sort -rn | head -20
```

## 阈值

| 信号 | 阈值 | 来源 |
|---|---|---|
| `too_many_lines` | 100 行 | 工具默认 |
| `cognitive_complexity` | 25 | 工具默认 |
| `excessive_nesting` | **本仓库不生效**：需在 `clippy.toml` 显式设置 `excessive-nesting-threshold` 才启用，工具默认为 **0（即关闭）**；本仓库未设置该项 | 工具默认为禁用，非本仓库可用阈值 |
| `too_many_arguments` | 7 个 | 工具默认 |
| `type_complexity` | 250 | 工具默认 |
| 文件行数 | 800 行 | 本文件定义 |
| 单文件函数数 | 40 个 | 本文件定义 |
| 模块扇出 | 15 个内部模块 | 本文件定义 |

阈值来自工具默认时**不得**通过修改 `clippy.toml` 调整——该文件属项目策略配置，
变更需用户确认。需要更严的阈值时，用结构扫描补充，不动配置。

`clippy::excessive_nesting` 是特例：它不是「默认阈值可用但不得调整」，而是「默认
即关闭，且开启该阈值本身就需要改 `clippy.toml`」。本仓库不改该文件，因此该 lint
在本仓库**永远不产出信号**——`--force-warn` 对它无效，因为它不是被 allow 压制，
而是阈值判定本身从未触发。实测：本仓库以文档命令跑全工作区，`excessive_nesting`
命中 0 条。Deep Nesting 类目改用结构扫描的缩进/花括号嵌套深度作为替代信号，见下节。

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `clippy::too_many_lines` | **Measured** |
| Deep Nesting | 结构扫描：缩进/花括号嵌套深度（本仓库使用）；`clippy::excessive_nesting`（**仅当**项目在 `clippy.toml` 显式配置 `excessive-nesting-threshold` 时才是 Measured 来源，本仓库未配置） | **Observed**（本仓库）／条件性 **Measured**（已配置阈值的项目） |
| Excessive Parameters | `clippy::too_many_arguments` | **Measured** |
| Dead Code | `dead_code` | **Measured** |
| Primitive Obsession | `clippy::type_complexity` | **Measured** |
| God Structure | 结构扫描：文件行数 · 单文件函数数 | **Observed** |
| Shotgun Surgery | 结构扫描：模块扇出 + 同形改动分布 | **Observed** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| Feature Envy | 阅读代码 | **Inferred** |
| Cyclic Dependency | 阅读模块依赖 | **Inferred** |

**结构扫描的产出只能标 Observed，不得标 Measured。** 花括号计数会被 format
string 的 `{}` 与 raw string 欺骗——本仓库实测中，一个花括号计数脚本把一个测试
函数误报为 401 行。逐条位置须人工复核后方可进报告。

`cognitive_complexity` 不单独映射类目：它是判定置信度的辅助量。一个
`too_many_lines` 越界但 `cognitive_complexity` 未越界的函数，通常是扁平分发表，
证据强度 Measured 而置信度 Low。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `clippy` 组件未安装 | 提示 `rustup component add clippy`；跳过全部 Measured 类目，报告标注「Measured 层未执行」；结构扫描照常 |
| 工作区编译失败 | 停止。clippy 需要通过类型检查才能出诊断；报告编译错误，不改代码 |
| `find` / `grep` 不可用 | 停止，不改用其他实现 |

不得自行安装工具，不得用 `cargo clean` 规避编译问题。
