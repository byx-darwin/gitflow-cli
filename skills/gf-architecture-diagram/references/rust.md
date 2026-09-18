# Rust Extraction Layer

**Detection:** `Cargo.toml` at project root (per `gf-quality/references/detector.md`).

## 提取命令

**单次运行，捕获输出后复用。**

```bash
cargo metadata --no-deps --format-version=1 > /tmp/gf-arch-metadata.json
```

`--no-deps` 是刻意的：只需要工作区成员（`packages[]`）与它们互相之间的依赖，
不需要展开每个成员的完整第三方依赖树——那会把外部 crate 也拉进 `packages[]`，
污染「内部模块」的判定。

## 模块判定

一个 `packages[]` 条目算作**内部模块**（成为图中的节点），当且仅当它出现在
`cargo metadata --no-deps` 的输出里——该命令本身已经把工作区外的 crate 排除，
不需要额外过滤逻辑。节点名取 `packages[].name`。

```python
import json
data = json.load(open("/tmp/gf-arch-metadata.json"))
modules = sorted(p["name"] for p in data["packages"])
```

## 边过滤规则

一条边 `A -> B` 成立，当且仅当 `A` 的 `dependencies[]` 中存在一项 `dep`
满足 `dep["name"] == B` **且** `dep.get("path")` 非空（`path` 字段非空是
「这是工作区内路径依赖，不是 crates.io 版本依赖」的判定依据）。

```python
edges = []
for pkg in data["packages"]:
    for dep in pkg["dependencies"]:
        if dep.get("path") and dep["name"] in modules:
            edges.append((pkg["name"], dep["name"]))
edges = sorted(set(edges))
```

`dep["name"] in modules` 是双重校验：`path` 非空已经意味着工作区内，但显式
再校验一次目标名称也在 `modules` 集合里，防止 `cargo metadata` 未来版本的
字段语义变化导致误判。

## 工具缺失降级

| 缺失 | 降级 |
|---|---|
| `cargo` 不可用 | 停止，不改用手工解析 `Cargo.toml`——那是本 Issue 明确要避免的语义推断 |
| `cargo metadata` 因工作区配置错误失败 | 报告 cargo 的原始错误，不修复 `Cargo.toml`，不重试 |
| 输出为空 `packages[]`（非工作区项目） | 报告「未检测到工作区成员，无内部模块可绘制」，不生成图 |

不得自行安装工具，不得用 `cargo clean` 规避元数据读取问题。
