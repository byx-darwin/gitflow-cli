# Java Detection Layer

**Detection:** `pom.xml` (Maven) or `build.gradle` / `build.gradle.kts` /
`settings.gradle` (Gradle), per `gf-quality/references/detector.md`.

## 检测命令

**单次运行，捕获输出后复用。**

```bash
# 生产源文件清单：显式排除测试源码根，两个工具共用同一份清单
find . -name '*.java' \
  -not -path './build/*' -not -path './target/*' \
  -not -path './.worktree/*' -not -path './.claude/worktrees/*' \
  -not -path '*/src/test/java/*' -not -path '*/src/test/*' \
  -not -path './test/*' -not -path './tests/*' \
  > /tmp/gf-smell-java-prod.txt

# 测试源文件清单：另存备查，不进候选表
find . -name '*.java' \
  \( -path '*/src/test/java/*' -o -path '*/src/test/*' \
     -o -path './test/*' -o -path './tests/*' \) \
  > /tmp/gf-smell-java-test.txt

# (1) 生产代码：候选表只从这一份取
OUT=$(mktemp -t gf-smell-java)
{
  pmd check --file-list /tmp/gf-smell-java-prod.txt \
    -R category/java/design.xml -f text --no-cache 2>&1
  checkstyle -c /google_checks.xml $(cat /tmp/gf-smell-java-prod.txt) 2>&1
} > "$OUT" 2>&1

# (2) 测试代码：另存备查
OUT_TEST=$(mktemp -t gf-smell-java-test)
{
  pmd check --file-list /tmp/gf-smell-java-test.txt \
    -R category/java/design.xml -f text --no-cache 2>&1
  checkstyle -c /google_checks.xml $(cat /tmp/gf-smell-java-test.txt) 2>&1
} > "$OUT_TEST" 2>&1
```

### 为什么用 `--file-list` 而不是 `-d .`

与 Rust 层的 `dead_code` 同一类缺陷：**结果取决于哪些文件被纳入分析**。

PMD 的目录扫描默认递归，官方 CLI 参考中**没有任何内置的测试源码排除**——
`src/test/java` 会被一并分析。（来源：<https://docs.pmd-code.org/latest/pmd_userdocs_cli_reference.html>，
PMD 7.x）`--exclude` / `--exclude-file-list` 虽然存在，但文档标明 PMD 侧的 `--exclude`
自 **7.14.0** 才加入，`--exclude-file-list` 同样是 7.14.0（旧名 `--ignore-list` 已弃用）。
`--file-list`（「a file containing a list of files to analyze, one path per line」）
没有这个版本门槛，且官方文档本身就用 `find ... > filelist.txt` 演示这种筛选方式，
因此本层统一用它——同一份清单也让 PMD 与 checkstyle 的口径完全一致。

后果具体到规则：`NcssCount` 的 `methodReportLevel`（60）与 `CyclomaticComplexity`（10）
在长测试方法上极易越界；`checkstyle` 的未使用 import 提示在测试文件里同样常见。
读者看到「Long Function · Measured」会以为是生产方法。

候选表**只**从 `$OUT` 取。`$OUT_TEST` 保留备查，属于测试可维护性，不得与生产代码结论混列。

**若只有一次 `-d .` 的捕获可用**，不得直接采信其中的 Measured 类目：须按路径剔除
测试源码根后再进候选表，并在报告中说明过滤是事后做的。

`--no-cache` 与 Rust 层的「单次运行」是同一个理由：缓存命中会让重复调用返回空
输出，把「缓存」误报成「干净」。

结构扫描：

```bash
# 与检测命令同口径：结构扫描同样只看生产源文件
FILES=$(cat /tmp/gf-smell-java-prod.txt)

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
| Long Function | `NcssCount`（`methodReportLevel`），**取自生产文件清单** | **Measured**（仅当来自已排除测试源码根的捕获）；用 `-d .` 时命中可能是测试方法，不得标 Measured |
| Deep Nesting | `CyclomaticComplexity`，**取自生产文件清单** | **Measured**（仅当来自已排除测试源码根的捕获） |
| Excessive Parameters | `ExcessiveParameterList`，**取自生产文件清单** | **Measured**（仅当来自已排除测试源码根的捕获） |
| God Structure | `NcssCount`（`classReportLevel`）+ `CouplingBetweenObjects`，**取自生产文件清单** | **Measured**（仅当来自已排除测试源码根的捕获） |
| Feature Envy | `CouplingBetweenObjects` + 阅读代码 | **Observed** |
| Shotgun Surgery | 结构扫描 import 扇出 | **Observed** |
| Duplicated Logic | 结构扫描 + 阅读比对 | **Observed** |
| Dead Code | `checkstyle` 未使用 import 提示，**取自生产文件清单** | **Observed**（本就不是 Measured；未使用 import 只证明该 import 无用，不证明被导入的类型无用。测试文件里的同类提示不得计入） |
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
