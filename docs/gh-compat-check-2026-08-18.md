# gf GitHub (gh) 兼容性检查报告

## 环境

| 项目 | 值 |
|---|---|
| 仓库 | `byx-darwin/gitflow-cli` |
| gf 版本 | v1.3.0 |
| gh 版本 | v2.97.0 |
| 平台 | GitHub（github.com） |
| 用户 | byx-darwin |
| 测试日期 | 2026-08-18 |
| 测试方式 | 只读命令实测 + 源码审查（`crates/github/src/*.rs`）+ 写操作实测（label create/edit/delete） |

## 结论摘要

**gf 1.3.0 在 GitHub 平台下整体兼容性良好**，与 GitLab 侧（glab 1.113）的「大量不兼容」形成鲜明对比。GitHub 侧实现（`gitflow_github`）已采用正确的底层调用模式：写操作走 `gh api` 或 gh 原生子命令，重新拉取走 `--json`，代码注释明确标注了「gh release create doesn't support --json」等注意事项。

**但发现 1 个真实 bug（P1）+ 2 个观察项（P2/P3）**：

1. **【P1】`gf label edit` 假失败**：编辑**实际成功**，但 gf 报错「Failed to edit label: GitHub CLI 执行失败」。根因：编辑成功后 `fetch_label` 调用 **`gh label view --json`**，而 **gh 2.97 没有 `label view` 子命令**（Available: clone/create/delete/edit/list）→ 重新拉取失败 → 整个命令误报失败。
2. **【P2】错误信息错位**（与 GitLab 侧同源）：所有失败统一提示「运行 `gh auth login`」，即使已登录。掩盖真实原因。
3. **【P3】`gf label list` 分页缺失**：`gh label list` 默认返回前 30 条，gf 未传 `--limit`/`--per-page`。仓库 44 个 label 只显示 30 个，创建成功的 label 在列表中不可见。

另注意：`gf release list` 返回 `id: 0` / `url: ""` 属字段未请求（非不兼容，见下）。

---

## 详细测试结果

### ✅ 通过的模块（实测）

| 命令 | 结果 | 说明 |
|---|---|---|
| `gf auth status` / `gf auth token` | ✅ | `gh auth token` 在 gh 2.97 存在（GitLab 侧 `glab auth token` 不存在，这是核心差异） |
| `gf issue list --state open` | ✅ | 9 个 open issue 正常返回 |
| `gf issue view <n>` / `comments <n>` | ✅ | 走 `gh api` 正常，URL 为 `/issues/N`（GitHub 无 GitLab `/work_items/N` 问题） |
| `gf label list` | ⚠️ | 返回正常，但**仅前 30 条**（见 P3） |
| `gf label create` | ✅ | 创建成功，JSON 返回完整字段 |
| `gf label delete` | ✅ | `gh label delete --yes` 兼容 |
| `gf milestone list` | ✅ | 正常返回空 |
| `gf release list` / `view` | ⚠️ | 正常，但 list 的 `id:0`/`url:""`（字段未请求，非失败） |
| `gf pr list --state open` | ✅ | 正常 |
| `gf pipeline status` / `report` | ✅ | `gh run list --json` 正常，GitHub Actions runs 正确映射 |
| `gf workflow list` | ✅ | 正常 |
| `gf doctor` | ✅ | 正常 |

### ❌ P1：`gf label edit` 假失败（唯一实质 bug）

**复现**：

```
$ gf label edit compat-gh-test2 --color ddccbb --description "gf-edited"
  × Failed to edit label 'compat-gh-test2': GitHub CLI 执行失败
  │ 🔧 修复建议：运行 `gh auth login` 完成登录
```

**但实际效果**（验证）：

```json
{"color":"ddccbb","description":"gf-edited"}   // 编辑已生效！
```

**底层调用探针**（PATH 拦截 gh 捕获实际参数）：

```
ARGS: label edit compat-gh-test2 --repo byx-darwin/gitflow-cli --color 3344ff --description probe2
EXIT: 0                                            ← edit 本身成功
ARGS: label view compat-gh-test2 --repo byx-darwin/gitflow-cli --json name,color,description
EXIT: 1                                            ← 重新拉取失败
STDERR: unknown flag: --json
Usage: gh label <command> [flags]
Available commands: clone, create, delete, edit, list    ← 没有 view
```

**根因（源码）**：`crates/github/src/label.rs`

- `edit()`（约 105-137 行）：`gh label edit` 成功后调用 `self.fetch_label(name)` 重新拉取数据。
- `fetch_label()`（约 160-178 行）：调用 **`gh label view <name> --json name,color,description`**。
- **gh 2.97 无 `label view` 子命令** → `fetch_label` 永远失败 → `edit` 永远报错。
- `gh label edit` 是成功且幂等性差的操作：用户看到失败重试会**反复覆盖写**。

**修复方向**：`fetch_label` 改用 `gh api repos/{owner}/{repo}/labels/{name}`（REST 返回 JSON，含 name/color/description），替换 `gh label view --json`。

### P2：错误信息错位（与 GitLab 侧同源）

所有写操作失败统一提示「运行 `gh auth login`」→ 误导。P1 场景已登录却提示登录。应区分：认证失败 / 参数不兼容 / 资源不存在 / 权限不足。

### P3：`gf label list` 分页缺失

- `gh label list` 默认 `--limit 30`，gf 未传分页参数 → 仓库 44 个 label 只显示 30 个。
- 实测：`gf label create compat-gh-test` 成功，但 `gf label list` 看不到（排在第 30 条之后）；`gh label list --limit 100` 可见。
- **修复方向**：`list()` 调用 `gh label list` 时传 `--limit 100`，或走 `gh api` 分页拉全量。

### 观察：`gf release list` 的 `id:0` / `url:""`

- `RELEASE_LIST_FIELDS = "tagName,name,isDraft,isPrerelease,createdAt,publishedAt"` 未请求 `databaseId`/`url` → `ReleaseData.id=0`、`url=""`。
- `gh release list` 本身不返回 `url`（`url` 是 `view` 字段），`id` 对应 `databaseId`。
- 非兼容性 bug，属数据不完整。修复可选：list 字段补 `url`，或文档说明 list 不含 url/id（用 view 获取）。

---

## 与 GitLab 侧（#199）对比

| 维度 | GitLab（glab 1.113） | GitHub（gh 2.97） |
|---|---|---|
| 写操作 `--output json` 硬编码 | ❌ 大量子命令不支持 → 全部写操作失败 | ✅ 无此问题（github crate 用 gh api / 原生子命令） |
| 不存在的子命令 | ❌ `auth token`/`mr ready`/`mr draft`/`label view` | ⚠️ 仅 **`gh label view`**（fetch_label 用） |
| URL 解析 | ❌ `/work_items/N` 无法解析 | ✅ `/issues/N` 正常 |
| note 参数 | ❌ `--body` vs glab `--message` | ✅ 走 gh api POST，无此问题 |
| 退出码一致性 | ❌ 不一致 | ⚠️ 未系统性验证（P1 场景 edit exit=0 但 gf 报错） |
| 错误信息错位 | ❌ 统一提示 glab auth login | ❌ 统一提示 gh auth login（P2，同源） |

**结论**：GitHub 侧无需 GitLab 侧那样的大规模修复。仅需修 P1（label fetch 改用 gh api）+ 可选补 P3 分页与错误信息区分。

## 修复建议

1. **P1（必修）**：`crates/github/src/label.rs` 的 `fetch_label` 改用 `gh api repos/{repo}/labels/{name}`，删除对 `gh label view --json` 的调用。
2. **P2（建议）**：错误分类展示，去掉统一的「运行 gh auth login」提示。
3. **P3（可选）**：label list 补分页参数。
4. **release list 字段（可选）**：按需补 `url`/`databaseId`。

## 清理情况

- 测试创建的 label `compat-gh-test`、`compat-gh-test2` 已删除，仓库无残留。
- 仓库中另有 `test-label`（"Test label"）与 `test-label-gh`（"Test"）两个 label 疑似历史测试残留（非本次创建，未删除，待用户确认）。
