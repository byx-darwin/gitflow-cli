# Code Review Report — Issue #345

> **交付方式：** `local_merge`（`git merge --no-ff`），无 PR 编号可挂载正式 GitHub review verdict。本报告是 `gf-workflow` Phase 4 的书面存档，`gf review approve/request-changes` 的 CLI 提交步骤因无 open PR 而不适用（`gf-review` skill 的 precondition 是 PR open）。
> **Merge commit:** `d352cc3`
> **Diff range:** `c5ec8ac..d352cc3`（`scripts/render-workflow-dashboard.py`、`scripts/tests/test_render_workflow_dashboard.py`、`Makefile`、`.gitignore`，另含设计文档/计划文档）
> **分析日期：** 2026-09-21
> **工作流：** `wf-2026-09-21-005`

## 背景

Issue #345 新增一个零依赖 Python3 脚本 `scripts/render-workflow-dashboard.py`，把 `.cache/workflows/active/*.json` 派生为单文件、自包含、离线可打开的 HTML 进度看板，随附独立测试脚本（不接入 `make test`）和新增 Makefile 目标 `render-workflow-dashboard`。实现阶段已有一次独立批量审查（未发现 Critical/Important，仅一条 Minor：空字符串 `status` 会退化为 `pending` 而非 `status-unknown`，非 spec 违反，未修复）。本报告不复用该结论，重新对最终合并态独立复核。

## 独立复核（不复用先前结论，重新验证）

### 1. HTML 转义主张——实测验证，非采信原审查断言

构造含 `<script>alert(1)</script>`、`<img src=x onerror=alert(2)>`、`"><script>alert(3)</script>` 的合同 JSON，直接调用 `render_dashboard()` 生成真实 HTML 并逐字节检查输出：

- 原始未转义标签（如字面 `<img src=x onerror=alert(2)>`）在输出中出现次数为 **0**。
- 对应的转义形式 `&lt;img src=x onerror=alert(2)&gt;`、`&lt;script&gt;alert(1)&lt;/script&gt;`、`&quot;&gt;&lt;script&gt;alert(3)&lt;/script&gt;` 均按预期出现。

结论：转义主张成立，`title`/`name`/`status`/`started_at`/`completed_at`/`executor` 等字段在写入 HTML 前均经 `html.escape()`，无 XSS 注入路径。

### 2. 回归基线复核——在合并后的 `dev` HEAD 上重新跑

```
make test              → 1611 tests run: 1611 passed, 0 skipped   （与实现阶段报告一致）
python3 scripts/tests/test_render_workflow_dashboard.py → Ran 8 tests … OK
make check-agent-sync  → Commands=78 Files=29 Refs=194 Mismatches=0   （基线不变）
make render-workflow-dashboard → 退出码 0，生成 .cache/workflows/dashboard.html
git check-ignore .cache/workflows/dashboard.html → 命中并原样打印路径（确认不会被误提交）
git status --short（跑完上述命令后）→ 无未预期改动
```

三条基线（1611/1611、78/29/194/0、目标产物被 `.gitignore` 覆盖）在合并到 `dev` 之后依然成立。

### 3. 独立发现的新问题（原分支内审查未覆盖）——**Important**

**问题：`render_dashboard()` 的逐文件容错只覆盖了 JSON 解析和顶层字段校验，未覆盖字段渲染阶段；任何合同的嵌套字段类型异常都会让整个脚本崩溃，而不是仅让该文件退化为错误卡片。**

代码位置 `scripts/render-workflow-dashboard.py:113-121`：

```python
for path in files:
    filename = os.path.basename(path)
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        _validate_contract(data)
    except (json.JSONDecodeError, ValueError, OSError) as exc:
        cards.append(_error_card_html(filename, str(exc)))
        continue
    cards.append(_card_html(data))   # ← 在 try/except 之外
```

`_validate_contract()` 只检查 `workflow_id` 存在、`phases` 是 dict，不检查 `phases` 内每个 phase 条目的类型，也不检查 `status`/`name` 等字段的类型。`_card_html()` → `_phase_html()` 在遇到非字符串 `status`（例如 `"status": ["complete"]`）时，会在 `STATUS_CLASSES.get(status, "status-unknown")` 处抛出 `TypeError: unhashable type: 'list'`，这次调用发生在 try/except 保护范围之外，异常直接向上传播，导致：

- **同目录下其余合法合同也不再渲染**（不是"该文件退化为错误卡片，其余文件正常"，而是整个 `render_dashboard()` 调用抛异常中止）。
- 不产出任何 HTML（连"当前没有活跃工作流"式的兜底页面都没有），`sys.exit(1)` 分支根本没机会命中——脚本以未捕获异常和非零退出码终止，`make render-workflow-dashboard` 直接报错退出。

已用独立脚本实测复现（临时目录 `/tmp/dash-crash-test`，同目录放一个 `status` 字段为列表的合同和一个完全正常的合同）：

```
TypeError: cannot use 'list' as a dict key (unhashable type: 'list')
  ...
  File "render-workflow-dashboard.py", line 121, in render_dashboard
    cards.append(_card_html(data))
```

复现后 `dashboard.html` 未生成——包括那个本应正常渲染的合法合同也一并陪葬。

**为何原审查/新增测试没抓到**：8 个测试用例覆盖了"缺少必需字段"（`_validate_contract` 能拦住）和"JSON 语法错误"，但没有覆盖"JSON 语法合法、必需字段存在，但字段类型不对"（如 `status` 是 list/dict/number 而非字符串，或某个 phase 条目本身不是 dict 而是字符串/数字）这一类。这类输入不会被 `json.JSONDecodeError`/`ValueError`/`OSError` 三种异常捕获，因为它触发的是 `_card_html`/`_phase_html` 内部的 `TypeError`/`AttributeError`。

**与 spec 的冲突**：设计文档 §6「逐文件容错」明确要求"某文件…缺少必需字段时，该文件渲染成一张'⚠️ 解析失败'的错误卡片…不中断其余文件的渲染，也不产出空白页"，Issue AC 也写明"合同损坏/缺字段给出可读报错，不静默空白"。当前实现只在"合同损坏"覆盖了 JSON 语法错误和顶层缺键两种情况，未覆盖"合同结构合法但嵌套字段类型异常"这一同样属于"损坏"范畴的情况，且崩溃后果比 spec 允许的"整体不可恢复"场景（仅限于输出路径无写权限等）更严重——它会因为单个文件的字段类型问题而波及所有其他健康合同。

**实际风险评估**：当前仓库内真实的 `.cache/workflows/active/wf-2026-09-21-005.json` 字段类型均规范（`status` 均为字符串），不会触发该崩溃；`gf-workflow` 编排器本身写入的合同预期类型稳定。但该脚本设计初衷即是要应对"合同损坏"（例如手工误改、并发写入截断、未来 schema 演进），当前实现在这类输入下不满足 spec 显式写明的容错承诺，判定为 Important（非 Critical：无安全影响，当前生产数据不会触发；但违反已写入 spec/AC 的行为承诺，且影响范围会波及无关的健康合同）。

**建议修复方向**（不在本次审查范围内实施，留待后续修复提交）：把 `cards.append(_card_html(data))` 也纳入 try/except，并把捕获的异常类型扩大到 `Exception`（渲染阶段的防御性异常都应归类为"该文件渲染失败"而非让整个批次崩溃），或者在 `_validate_contract` / `_phase_html` 中对非字符串类型做显式的类型收窄。

### 4. 延续的 Minor（不新增，仅确认仍存在）

`_phase_html()` 中 `status = phase.get("status") or "pending"`：空字符串 `""`、`0`、`False` 等假值 `status` 会被视为"缺失"而回退到 `pending`，理论上应该是"未知/非法状态"（`status-unknown`）。此前审查已记录，非 spec 违反，未修复；本次复核未发现新的严重性证据，维持"Minor，不阻塞"的判定。

### 5. 其它复核点（均无问题）

- `Makefile` 新增目标插入位置、`.PHONY` 列表追加，与设计文档 §7 描述完全一致，`diff` 逐行核对无偏差。
- `.gitignore` 新增 `__pycache__/` 一行，作用范围合理（标准 Python 产物），不影响既有规则。
- 测试脚本明确注明"不接入 `make test`"且未在任何 Makefile 目标中被隐式调用，符合"Python 测试脚本独立于 Rust/nextest 套件"的既定约束。
- 未发现依赖 Issue #344（Phase 3 change-surface gate）之后的合并状态漂移；`c5ec8ac..d352cc3` 范围内只包含本 Issue 声明的文件，无范围蔓延。

## 结论

**Request Changes.** 1 条 Important（`render_dashboard` 在遇到嵌套字段类型异常时会整体崩溃，波及同批次所有健康合同，违反 spec §6 逐文件容错承诺）、1 条 Minor（延续自实现阶段，空字符串 status 误判为 pending，不阻塞）。HTML 转义主张已独立实测验证成立；Rust 测试套件（1611/1611）与 `check-agent-sync` 基线（78/29/194/0）在合并后的 `dev` HEAD 上复测均通过，未发现回归。

因交付方式为 `local_merge`，本报告即为正式书面存档，不额外调用 `gf review` CLI 提交动作。建议开一个后续 fix 提交，把 `_card_html(data)` 纳入 try/except 保护范围（或对渲染阶段异常做等价兜底），修复后重跑本报告第 3 节的复现用例确认不再崩溃。
