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
| `NcssMethodCount` | 60 | 工具默认 |
| `ExcessiveParameterList` | 10 | 工具默认 |
| `ExcessiveClassLength` | 1000 | 工具默认 |
| `CouplingBetweenObjects` | 20 | 工具默认 |
| 文件行数 | 800 行 | 本文件定义 |
| import 扇出 | 30 | 本文件定义 |

## 类目映射

| 类目 | 检测来源 | 证据强度上限 |
|---|---|---|
| Long Function | `NcssMethodCount` | **Measured** |
| Deep Nesting | `CyclomaticComplexity` | **Measured** |
| Excessive Parameters | `ExcessiveParameterList` | **Measured** |
| God Structure | `ExcessiveClassLength` + `CouplingBetweenObjects` | **Measured** |
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
