# Node.js / TypeScript Detection Layer

**Detection:** `package.json` (per `gf-quality/references/detector.md`), runtime
resolved by lock file per that same reference.

## 检测命令

**单次运行，捕获输出后复用。**

```bash
OUT=$(mktemp -t gf-smell-node)
npx eslint . \
  --no-eslintrc \
  --ext .js,.jsx,.ts,.tsx \
  --ignore-pattern 'node_modules/**' \
  --ignore-pattern 'dist/**' \
  --ignore-pattern 'build/**' \
  --rule '{"complexity":["warn",15]}' \
  --rule '{"max-depth":["warn",4]}' \
  --rule '{"max-lines-per-function":["warn",100]}' \
  --rule '{"max-params":["warn",5]}' \
  --rule '{"max-lines":["warn",800]}' \
  --format unix \
  > "$OUT" 2>&1
```

`--no-eslintrc` 是刻意的：项目自身的 eslint 配置可能已把这些规则关掉，那属于
「有人决定不看」，与 Rust 层用 `--force-warn` 穿透 `#[allow]` 是同一件事。项目
配置本身不被修改。

结构扫描：

```bash
FILES=$(find . \( -name '*.ts' -o -name '*.tsx' -o -name '*.js' -o -name '*.jsx' \) \
  -not -path './node_modules/*' -not -path './dist/*' -not -path './build/*' \
  -not -path './.worktree/*' -not -path './.claude/worktrees/*')

echo "$FILES" | xargs wc -l | sort -rn | head -20
echo "$FILES" | xargs grep -c "^import " | sort -t: -k2 -rn | head -20
```

## 阈值

| 信号 | 阈值 | 来源 |
|---|---|---|
| `complexity` | 15 | 本文件定义（规则无默认值，须显式传） |
| `max-depth` | 4 | 本文件定义 |
| `max-lines-per-function` | 100 | 本文件定义 |
| `max-params` | 5 | 本文件定义 |
| `max-lines` | 800 | 本文件定义 |
| import 扇出 | 20 | 本文件定义 |

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `max-lines-per-function` | **Measured** |
| Deep Nesting | `max-depth` | **Measured** |
| Excessive Parameters | `max-params` | **Measured** |
| God Structure | `max-lines` + 结构扫描文件行数 | **Measured**（规则部分） |
| Shotgun Surgery | 结构扫描 import 扇出 | **Observed** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| Dead Code | 阅读代码（无默认规则可用） | **Inferred** |
| Primitive Obsession | 阅读代码 | **Inferred** |
| Feature Envy | 阅读代码 | **Inferred** |
| Cyclic Dependency | 阅读 import 图 | **Inferred** |

`complexity` 规则不单独映射类目：与 Rust 层的认知复杂度同理，它是判定置信度的
辅助量——行数越界但圈复杂度未越界，通常是扁平结构，置信度应降档。

结构扫描的产出只能标 Observed，不得标 Measured。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `eslint` 不可用（无网络 / 未安装） | 跳过全部 Measured 类目，报告标注「Measured 层未执行」；结构扫描照常 |
| 项目使用 flat config 且 `--no-eslintrc` 报错 | 改用 `ESLINT_USE_FLAT_CONFIG=false`；仍失败则按上一行降级 |
| TypeScript 解析失败 | 记录受影响文件，其余继续 |

不得自行安装依赖，不得修改项目的 eslint 配置。
