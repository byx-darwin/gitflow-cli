# Node.js / TypeScript Detection Layer

**Detection:** `package.json` (per `gf-quality/references/detector.md`), runtime
resolved by lock file per that same reference.

## 检测命令

**单次运行，捕获输出后复用。** 生产代码与测试代码分开捕获，每条命令各跑一次。

```bash
TEST_GLOBS=(
  --ignore-pattern '**/*.test.*'   --ignore-pattern '**/*.spec.*'
  --ignore-pattern '**/__tests__/**' --ignore-pattern '**/__mocks__/**'
  --ignore-pattern 'test/**'       --ignore-pattern 'tests/**'
  --ignore-pattern 'e2e/**'        --ignore-pattern 'cypress/**'
)

# (1) 生产代码：候选表只从这一份取
OUT=$(mktemp -t gf-smell-node)
npx eslint . "${TEST_GLOBS[@]}" \
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

# (2) 测试代码：同样规则、只看测试文件，另存备查，不进候选表
OUT_TEST=$(mktemp -t gf-smell-node-test)
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
  > "$OUT_TEST" 2>&1
```

### 为什么必须把测试文件分开

与 Rust 层的 `dead_code` 同一类缺陷：**结果取决于哪些文件被纳入分析，而读者会默认
那是生产代码**。

本层不存在 Dead Code 工具（该类目本就是 `Inferred`／阅读代码），因此没有「未使用符号」
方向的假阳性；风险在另一侧的 **Measured 类目**：`--no-config-lookup` 刻意绕开了项目
配置，连同项目自己的 `ignores`（flat config 里测试目录通常就排在那里）一起绕开了。
于是 `eslint .` 会把 `*.test.ts`、`__tests__/`、`e2e/` 一并纳入。其中：

- `max-lines` 是**文件级**规则，测试文件超过 800 行会直接成为 God Structure 候选——
  这一点无需推断，规则口径就是按文件计
- `max-lines-per-function` 会把 `describe()` / `it()` 之类的回调函数体计入。官方文档
  未逐字说明「作为实参传入的箭头回调」是否计入，但它明确把 IIFE 的函数体计入、并把
  「回调密集的代码被 `max-statements` 低估」列为该规则存在的理由
  （来源：<https://eslint.org/docs/latest/rules/max-lines-per-function>，ESLint v9 文档）。
  **此处按文档无法 100% 确证 `describe()` 回调一定被计入**，故不据此下断言；
  `max-lines` 一条已足以确立风险，分离捕获对两者都有效

候选表**只**从 `$OUT` 取。`$OUT_TEST` 保留备查：超长的测试文件不是无意义的信号，
但它属于测试可维护性，与「生产代码的 God Structure」不是同一个结论，不得混列。

**若只有一次未分离的捕获可用**，不得直接采信其中的 Measured 类目：须按 `filePath`
用上面的测试 glob 二次过滤，并在报告中说明过滤是事后做的。

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
| Long Function | `max-lines-per-function`，**取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获） |
| Deep Nesting | `max-depth`，**取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获） |
| Excessive Parameters | `max-params`，**取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获） |
| God Structure | `max-lines`（**取自生产代码捕获**）+ 结构扫描文件行数 | **Measured**（规则部分，且仅当来自已排除测试文件的捕获）；未分离时命中可能是测试文件，不得标 Measured |
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
