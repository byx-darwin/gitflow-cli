# Python Detection Layer

**Shared language profile:** `gf-quality/references/profiles/python.md`. Read it for tools, version sources, and scan exclusions.

**Detection:** `pyproject.toml` / `setup.py` / `setup.cfg`
(per `gf-quality/references/detector.md`).

## 检测命令

**单次运行，捕获输出后复用。** 生产代码与测试代码分开捕获，每条命令各跑一次。

```bash
TEST_EXCLUDE='tests,test,conftest.py,test_*.py,*_test.py,**/tests/**,**/test/**'
RADON_EXCLUDE='venv/*,.venv/*,build/*,dist/*,tests/*,test/*,*/test_*.py,*/*_test.py,conftest.py'

# (1) 生产代码：候选表只从这一份取
OUT=$(mktemp "${TMPDIR:-/tmp}/gf-smell-python.XXXXXX")
{
  ruff check . \
    --isolated \
    --exclude "$TEST_EXCLUDE" \
    --select C901,PLR0911,PLR0912,PLR0913,PLR0915 \
    --output-format concise 2>&1
  radon cc . -s -n C -e "$RADON_EXCLUDE" 2>&1
} > "$OUT" 2>&1

# (2) 测试代码：同样规则、只看测试文件，另存备查，不进候选表
OUT_TEST=$(mktemp "${TMPDIR:-/tmp}/gf-smell-python-test.XXXXXX")
ruff check . \
  --isolated \
  --select C901,PLR0911,PLR0912,PLR0913,PLR0915 \
  --output-format concise 2>&1 \
  | grep -E '(^|/)(tests?/|conftest\.py|test_|.*_test\.py)' > "$OUT_TEST" 2>&1
```

`--isolated` 是刻意的：项目自身的配置可能已把这些规则 ignore 掉，那属于
「有人决定不看」。项目配置本身不被修改。

### 为什么必须把测试文件分开

与 Rust 层的 `dead_code` 同一类缺陷：**结果取决于哪些文件被纳入分析，而读者会默认
那是生产代码**。

本层不存在未使用符号检测（Dead Code 类目本就是 `Inferred`／阅读代码，所选规则集
`C901,PLR09xx` 不含 `F401` 之类），因此**没有死代码方向的假阳性**；风险全部在
Measured 类目一侧。

`ruff` 的默认 `exclude` 只覆盖版本控制目录、缓存、虚拟环境与构建产物
（`.git`、`.mypy_cache`、`.venv`、`dist`、`build`、`node_modules`……），
**不含任何测试目录**；官方文档反而把 `tests` 当作「需要你自己加进 `extend-exclude`」
的示例。（来源：<https://docs.astral.sh/ruff/settings/>）而 `--isolated` 只会让 ruff
忽略配置文件，不可能凭空**增加**排除项，因此它绝不会替你排掉 `tests/`。

后果具体到规则：`PLR0915`（语句数，阈值 50）与 `PLR0912`（分支数，阈值 12）在
pytest 的参数化测试、长夹具、`assert` 密集的用例上极易越界。读者看到
「Long Function · Measured」会以为是生产函数。

候选表**只**从 `$OUT` 取。`$OUT_TEST` 保留备查：超长测试用例属于测试可维护性，
与「生产代码的 Long Function」不是同一个结论，不得混列。

**若只有一次未分离的捕获可用**，不得直接采信其中的 Measured 类目：须按路径用上面的
测试模式二次过滤，并在报告中说明过滤是事后做的。

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
| Long Function | `PLR0915` 语句数，**取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获）；未分离时命中可能是测试用例，不得标 Measured |
| Deep Nesting | `PLR0912` 分支数 + `radon` 等级，**均取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获） |
| Excessive Parameters | `PLR0913`，**取自生产代码捕获** | **Measured**（仅当来自已排除测试文件的捕获；夹具参数多的测试函数极易误入） |
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
| `ruff` 未安装 | 参见共享 Python profile，建议用户自行安装；跳过全部 Measured 类目，报告标注 |
| `radon` 未安装 | 参见共享 Python profile，建议用户自行安装；Deep Nesting 只保留 `PLR0912` 一个来源，报告标注 |
| 语法错误导致解析失败 | 记录受影响文件，其余继续 |

不得自行安装依赖，不得修改项目配置。
