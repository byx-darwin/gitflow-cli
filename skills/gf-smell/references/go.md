# Go Detection Layer

**Detection:** `go.mod` or `go.work` (per `gf-quality/references/detector.md`).

## 检测命令

**单次运行，捕获输出后复用。**

```bash
OUT=$(mktemp -t gf-smell-go)
{
  gocyclo -over 15 . 2>&1
  staticcheck ./... 2>&1
  go vet ./... 2>&1
} > "$OUT" 2>&1
```

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
| Long Function | `gocyclo` + 结构扫描函数行数 | **Measured**（gocyclo 部分） |
| Deep Nesting | `gocyclo` 间接反映 | **Inferred** |
| Excessive Parameters | 结构扫描函数签名 | **Observed** |
| Dead Code | `staticcheck` U1000 系列 | **Measured** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| God Structure | 结构扫描：文件行数 · 单文件函数数 | **Observed** |
| Shotgun Surgery | 结构扫描：import 扇出 | **Observed** |
| Primitive Obsession | 阅读代码 | **Inferred** |
| Feature Envy | 阅读代码 | **Inferred** |
| Cyclic Dependency | `go vet` 包级报错 | **Measured** |

结构扫描的产出只能标 Observed，不得标 Measured：按行首模式统计函数会被字符串
字面量与注释干扰，逐条位置须人工复核。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `gocyclo` 未安装 | 提示 `go install github.com/fzipp/gocyclo/cmd/gocyclo@latest`；跳过圈复杂度类目，报告标注 |
| `staticcheck` 未安装 | 提示 `go install honnef.co/go/tools/cmd/staticcheck@latest`；跳过 Dead Code 类目，报告标注 |
| `go vet` 失败 | 记录错误，不改代码；其余类目继续 |

不得自行安装工具。
