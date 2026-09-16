# Python Detection Layer

**Detection:** `pyproject.toml` / `setup.py` / `setup.cfg`
(per `gf-quality/references/detector.md`).

## 检测命令

**单次运行，捕获输出后复用。**

```bash
OUT=$(mktemp -t gf-smell-python)
{
  ruff check . \
    --isolated \
    --select C901,PLR0911,PLR0912,PLR0913,PLR0915 \
    --output-format concise 2>&1
  radon cc . -s -n C -e 'venv/*,.venv/*,build/*,dist/*' 2>&1
} > "$OUT" 2>&1
```

`--isolated` 是刻意的：项目自身的配置可能已把这些规则 ignore 掉，那属于
「有人决定不看」。项目配置本身不被修改。

结构扫描：

```bash
FILES=$(find . -name '*.py' \
  -not -path './venv/*' -not -path './.venv/*' \
  -not -path './build/*' -not -path './dist/*' \
  -not -path './.worktree/*' -not -path './.claude/worktrees/*')

echo "$FILES" | xargs wc -l | sort -rn | head -20
echo "$FILES" | xargs grep -c '^\s*def \|^\s*async def ' | sort -t: -k2 -rn | head -20
echo "$FILES" | xargs grep -c '^from \|^import ' | sort -t: -k2 -rn | head -20
```

## 阈值

| 信号 | 阈值 | 来源 |
|---|---|---|
| `C901` McCabe 复杂度 | 10 | 工具默认 |
| `PLR0911` return 语句数 | 6 | 工具默认 |
| `PLR0912` 分支数 | 12 | 工具默认 |
| `PLR0913` 参数数 | 5 | 工具默认 |
| `PLR0915` 语句数 | 50 | 工具默认 |
| `radon` 等级 | C 及以下 | 本文件定义（`-n C`） |
| 文件行数 | 800 行 | 本文件定义 |
| 单文件函数数 | 40 个 | 本文件定义 |

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `PLR0915` 语句数 | **Measured** |
| Deep Nesting | `PLR0912` 分支数 + `radon` 等级 | **Measured** |
| Excessive Parameters | `PLR0913` | **Measured** |
| God Structure | 结构扫描：文件行数 · 单文件函数数 | **Observed** |
| Shotgun Surgery | 结构扫描 import 扇出 | **Observed** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| Dead Code | 阅读代码（所选规则集不含） | **Inferred** |
| Primitive Obsession | 阅读代码 | **Inferred** |
| Feature Envy | 阅读代码 | **Inferred** |
| Cyclic Dependency | 阅读 import 图 | **Inferred** |

`C901` 与 `PLR0911` 不单独映射类目：二者是判定置信度的辅助量。

结构扫描的产出只能标 Observed，不得标 Measured：按行首 `def` 统计会把嵌套函数、
字符串内容与注释一并计入，逐条位置须人工复核。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `ruff` 未安装 | 提示 `pip install ruff`；跳过全部 Measured 类目，报告标注 |
| `radon` 未安装 | 提示 `pip install radon`；Deep Nesting 只保留 `PLR0912` 一个来源，报告标注 |
| 语法错误导致解析失败 | 记录受影响文件，其余继续 |

不得自行安装依赖，不得修改项目配置。
