# Node.js / TypeScript Detection Layer

**Detection:** `package.json` (per `gf-quality/references/detector.md`), runtime
resolved by lock file per that same reference.

## 检测命令

**单次运行，捕获输出后复用。**

```bash
OUT=$(mktemp -t gf-smell-node)
npx eslint . \
  --no-config-lookup \
  --ext .js,.jsx,.ts,.tsx \
  --ignore-pattern 'node_modules/**' \
  --ignore-pattern 'dist/**' \
  --ignore-pattern 'build/**' \
  --rule '{"complexity":["warn",15]}' \
  --rule '{"max-depth":["warn",4]}' \
  --rule '{"max-lines-per-function":["warn",100]}' \
  --rule '{"max-params":["warn",5]}' \
  --rule '{"max-lines":["warn",800]}' \
  --format json \
  > "$OUT" 2>&1
```

`--no-config-lookup` 是刻意的：项目自身的 eslint 配置（`eslint.config.*`）可能已
把这些规则关掉，那属于「有人决定不看」，与 Rust 层用 `--force-warn` 穿透
`#[allow]` 是同一件事。项目配置本身不被修改。`--no-config-lookup` 是当前
（flat config，ESLint ≥ v9）下唯一能达到这个效果的开关：旧版 `--no-eslintrc`
在 v9.0.0 起已被移除，传入会直接报错并提示改用 `--no-config-lookup`。`--ext`
在 flat config 下仍然有效，官方文档明确说明它主要就是配合 `--no-config-lookup`
使用的（没有配置文件声明扩展名时，靠 `--ext` 显式指定）。`--format` 从内置
`unix` 改为内置 `json`：`unix` 在 v9.0.0 起从核心移除，只以 `eslint-formatter-unix`
独立包形式存在，需要额外安装；`json` 是仍打包在核心里的内置格式，且更利于脚本
解析捕获结果。

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
| `eslint` 版本早于 v8.21（不支持 `--no-config-lookup`，只认 eslintrc 体系） | 不发明替代开关：ESLint v9 移除了 `--no-eslintrc`、v10 又移除了整套 eslintrc 体系及 `ESLINT_USE_FLAT_CONFIG` 环境变量，v9 之前的旧版本也没有 `--no-config-lookup`；两端互不兼容意味着不存在同时对新旧版本都有效的单一穿透开关。记录探测到的 `eslint --version` 与 `--no-config-lookup` 的报错原文，跳过全部 Measured 类目，报告标注「该项目的 ESLint 版本无法被本探测穿透」 |
| TypeScript 解析失败 | 记录受影响文件，其余继续 |

不得自行安装依赖，不得修改项目的 eslint 配置。
