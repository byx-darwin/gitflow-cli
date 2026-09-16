# Java Detection Layer

**Detection:** `pom.xml` (Maven) or `build.gradle` / `build.gradle.kts` /
`settings.gradle` (Gradle), per `gf-quality/references/detector.md`.

## 检测命令

**单次运行，捕获输出后复用。**

```bash
OUT=$(mktemp -t gf-smell-java)
{
  pmd check -d . -R category/java/design.xml -f text --no-cache 2>&1
  checkstyle -c /google_checks.xml $(find . -name '*.java' \
    -not -path './build/*' -not -path './target/*' \
    -not -path './.worktree/*' -not -path './.claude/worktrees/*') 2>&1
} > "$OUT" 2>&1
```

`--no-cache` 与 Rust 层的「单次运行」是同一个理由：缓存命中会让重复调用返回空
输出，把「缓存」误报成「干净」。

结构扫描：

```bash
FILES=$(find . -name '*.java' \
  -not -path './build/*' -not -path './target/*' \
  -not -path './.worktree/*' -not -path './.claude/worktrees/*')

echo "$FILES" | xargs wc -l | sort -rn | head -20
echo "$FILES" | xargs grep -c '^\s*\(public\|private\|protected\).*(' | sort -t: -k2 -rn | head -20
echo "$FILES" | xargs grep -c '^import ' | sort -t: -k2 -rn | head -20
```

## 阈值

| 信号 | 阈值 | 来源 |
|---|---|---|
| `CyclomaticComplexity` | 10 | 工具默认（PMD design 规则集） |
| `NcssCount`（`methodReportLevel`） | 60 | 工具默认 |
| `NcssCount`（`classReportLevel`） | 1500 | 工具默认 |
| `ExcessiveParameterList` | 10 | 工具默认 |
| `CouplingBetweenObjects` | 20 | 工具默认 |
| 文件行数 | 800 行 | 本文件定义 |
| import 扇出 | 30 | 本文件定义 |

`NcssCount` 衡量的是 NCSS（non-commenting source statements，非注释源语句数），
不是原始行数；与本文件及其余语言层中"文件行数"一类的基于行的阈值不是同一单位，
两者不可直接比较。

`NcssMethodCount`、`ExcessiveClassLength` 均已在 `category/java/design.xml` 中
不再可用——前者在 PMD 6.0.0 被 `NcssCount` 取代（旧规则属于 PMD 5 时代的
`rulesets/java/codesize.xml`），后者在 PMD 6.55 起被标记为 Deprecated、并在
PMD 7 中移除，其类级别度量已并入 `NcssCount` 的 `classReportLevel`。本文件的
检测命令只加载 `category/java/design.xml`，因此不使用这两个已消失的规则名。

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `NcssCount`（`methodReportLevel`） | **Measured** |
| Deep Nesting | `CyclomaticComplexity` | **Measured** |
| Excessive Parameters | `ExcessiveParameterList` | **Measured** |
| God Structure | `NcssCount`（`classReportLevel`） + `CouplingBetweenObjects` | **Measured** |
| Feature Envy | `CouplingBetweenObjects` + 阅读代码 | **Observed** |
| Shotgun Surgery | 结构扫描 import 扇出 | **Observed** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| Dead Code | `checkstyle` 未使用 import 提示 | **Observed** |
| Primitive Obsession | 阅读代码 | **Inferred** |
| Cyclic Dependency | 阅读 import 图 | **Inferred** |

结构扫描的产出只能标 Observed，不得标 Measured：按可见性修饰符加括号统计方法会
把字段声明与注释中的样例一并计入，逐条位置须人工复核。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `pmd` 未安装 | 提示安装 PMD 发行版；跳过全部 Measured 类目，报告标注 |
| `checkstyle` 未安装 | 跳过 Dead Code 类目，报告标注 |
| 项目未编译 / 依赖缺失 | PMD 的 design 规则集不需要编译产物，继续；记录任何解析失败的文件 |

不得自行安装工具，不得修改项目的 PMD / checkstyle 配置。
