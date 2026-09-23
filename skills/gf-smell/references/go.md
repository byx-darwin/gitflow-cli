# Go Detection Layer

**Shared language profile:** `gf-quality/references/profiles/go.md`. Read it for tools, version sources, and scan exclusions.

**Detection:** `go.mod` or `go.work` (per `gf-quality/references/detector.md`).

## 检测命令

**单次运行，捕获输出后复用。** 结构类信号与 Dead Code 分开捕获，每条命令各跑一次。

```bash
# (1) 结构类信号：gocyclo 必须显式排除 _test.go
OUT=$(mktemp "${TMPDIR:-/tmp}/gf-smell-go.XXXXXX")
{
  gocyclo -over 15 -ignore '_test|vendor/|/\.worktree/|/\.claude/worktrees/' . 2>&1
  staticcheck ./... 2>&1
  go vet ./... 2>&1
} > "$OUT" 2>&1

# (2) Dead Code：必须再跑一次，关闭测试文件
OUT_DEAD=$(mktemp "${TMPDIR:-/tmp}/gf-smell-go-dead.XXXXXX")
staticcheck -tests=false ./... 2>&1 | grep 'U1000' > "$OUT_DEAD" 2>&1
```

Dead Code 的候选**只**从 `$OUT_DEAD` 取；`$OUT` 里的 U1000 一律忽略。

### 为什么 Dead Code 必须单独跑 `-tests=false`

与 Rust 层的 `dead_code` 是**同一类缺陷、相反方向**：那里是把测试单元算进来导致
**多报**，这里是把测试单元算进来导致**少报**。

`staticcheck` 的 `-tests` 默认为 `true`，即默认连同测试一起分析。官方 CLI 文档明确
写道：`-tests=false` "is primarily useful for the U1000 check, as it allows finding
code that is only used by tests and would otherwise be unused."
（来源：<https://staticcheck.dev/docs/running-staticcheck/cli/>，staticcheck 2025.1 系列文档）

也就是说，**只被测试引用的生产代码在默认设置下不会被报出来**。读者看到「U1000 零命中」
会以为生产代码没有死代码，实际结论只是「连测试一起算也没有死代码」——这两句话不等价。

两次捕获因此对应两个不同的类别，报告必须分开写：

| 来源 | 含义 | 如何归类 |
|---|---|---|
| 同时出现在 `$OUT` 与 `$OUT_DEAD` | 生产与测试都不引用 | 真死代码，进 Dead Code 候选 |
| 只出现在 `$OUT_DEAD`（即 `-tests=false` 新增的） | 生产代码不引用，只有测试引用 | **单独一类**：仅测试使用的生产符号。可能是合理的测试专用导出，也可能是实现已废弃而测试还在跑。须 Stage 2 逐条读代码判定，不得与真死代码混列 |

用 `comm -13` 对两份位置清单取差集即可得到第二类。

**若只有一次默认捕获可用**，不得据此宣称 Dead Code 干净：必须补跑 (2)；无法补跑时，
Dead Code 类目在报告中记为「未执行」。

### 为什么 `gocyclo` 必须显式传 `-ignore`

`gocyclo` 不内置任何测试文件排除，官方 README 的示例本身就是
`gocyclo -top 20 -ignore "_test|Godeps|vendor/" ."`——需要把 `_test` 写进正则才排得掉。
（来源：<https://github.com/fzipp/gocyclo>）Go 的表驱动测试天然又长又多分支，不排除会让
Long Function 候选表被测试函数淹没。结构扫描一侧已用 `-not -name '*_test.go'` 排除，
两侧口径必须一致。

`go vet ./...` 同样分析测试文件，其包级报错可能来自只存在于测试的导入环。该信号喂给
Cyclic Dependency 类目，Stage 2 须确认报错位置是否在 `_test.go` 中。

结构扫描（排除目录与 Stage 0 一致）：

```bash
FILES=$(find . -name '*.go' -not -path './vendor/*' -not -path './.worktree/*' \
  -not -path './.claude/worktrees/*' -not -name '*_test.go')

echo "$FILES" | xargs wc -l | sort -rn | head -20
echo "$FILES" | xargs grep -c '^func ' | sort -t: -k2 -rn | head -20
echo "$FILES" | xargs grep -c '^\s*import\|^\s*"' | sort -t: -k2 -rn | head -20
```

## 阈值

| 信号 | 阈值 | 来源 |
|---|---|---|
| 圈复杂度（`gocyclo`） | 15 | 本文件定义（工具无默认，须显式传 `-over`） |
| 函数行数 | 60 行 | 本文件定义 |
| 函数参数数 | 5 个 | 本文件定义 |
| 文件行数 | 800 行 | 本文件定义 |
| 单文件函数数 | 40 个 | 本文件定义 |

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `gocyclo`（**须带 `-ignore` 排除 `_test`**）+ 结构扫描函数行数 | **Measured**（gocyclo 部分；未排除测试文件时命中可能是测试函数，不得标 Measured） |
| Deep Nesting | `gocyclo` 间接反映 | **Inferred** |
| Excessive Parameters | 结构扫描函数签名 | **Observed** |
| Dead Code | `staticcheck` U1000，**且必须取自 `-tests=false` 的捕获** | **Measured**（仅当同时持有两次捕获并已按上表分出「真死代码」与「仅测试使用」两类）；单跑默认设置得到的 U1000 结果不足以支撑 Dead Code 结论 |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| God Structure | 结构扫描：文件行数 · 单文件函数数 | **Observed** |
| Shotgun Surgery | 结构扫描：import 扇出 | **Observed** |
| Primitive Obsession | 阅读代码 | **Inferred** |
| Feature Envy | 阅读代码 | **Inferred** |
| Cyclic Dependency | `go vet` 包级报错 | **Measured**（仅当报错不在 `_test.go` 中；测试专属的导入环须另行标注） |

结构扫描的产出只能标 Observed，不得标 Measured：按行首模式统计函数会被字符串
字面量与注释干扰，逐条位置须人工复核。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `gocyclo` 未安装 | 参见共享 Go profile，建议用户自行安装；跳过圈复杂度类目，报告标注 |
| `staticcheck` 未安装 | 参见共享 Go profile，建议用户自行安装；跳过 Dead Code 类目，报告标注 |
| `staticcheck` 版本不支持 `-tests` | 不发明替代做法；跳过 Dead Code 类目，报告标注「Dead Code 层未执行」，记录 `staticcheck -version` 原文 |
| `go vet` 失败 | 记录错误，不改代码；其余类目继续 |

不得自行安装工具。
