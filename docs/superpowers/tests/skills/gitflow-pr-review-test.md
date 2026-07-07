# gitflow-pr-review 测试场景

> **对应 Issue：** #34
> **分析日期：** 2026-07-07
> **前提：** 这些测试场景在 skill 重构后用于验证 Claude 的行为是否符合 Superpowers 最佳实践。

---

## 基线测试场景

### 场景 1: 不用 skill 时的基线行为

**背景：** 用户要求 Claude 审查 PR，但未加载 `gitflow-pr-review` skill。

**任务：** "请审查 PR #101"

**预期基线行为（无 skill）：**
- Claude 直接调用 `gitflow-cli pr approve 101 --body "LGTM"` 而跳过系统化的逐维分析
- Claude 可能只关注代码风格和明显 bug，遗漏安全性和性能维度
- 审查结论缺乏结构化，不包含 per-dimension 标记和 file:line 引用
- Claude 可能在发现行内问题时自行切换到 inline review 模式

**skill 应提供的价值：** 强制 6 维度逐项检查，确保每个维度都有明确结论和文件引用。

**实际行为记录：** [运行后记录]

---

### 场景 2: 触发准确性 — 应触发的场景

**背景：** 用户表达需要审查 PR。

**输入变体（均应触发 skill）：**
1. "review PR #101"
2. "帮我审查一下 PR #55"
3. "Please check pull request #78 and let me know if it's ready to merge"
4. "approve PR #42 after checking"
5. "这个 PR 有没有安全问题？帮我看看"
6. "代码审查 PR #30"

**预期行为：** Claude 加载 `gitflow-pr-review` skill，按 6 维度流程执行。

**实际行为记录：** [运行后记录]

---

### 场景 3: 触发准确性 — 不应触发的场景

**背景：** 用户的请求实际上需要其他 skill。

**输入变体（不应触发 gitflow-pr-review）：**
1. "在 PR #101 的第 42 行留个评论说这里有 SQL 注入" → 应触发 `gitflow-pr-inline-review`
2. "PR #55 审查完了，帮我修复那些问题" → 应触发 `gitflow-pr-apply-feedback`
3. "关闭 PR #78" → 应触发 `gitflow-pr`
4. "cargo audit 看一下有没有漏洞" → 应触发 `gitflow-security-check`
5. "给 PR #30 加个 label" → 应触发 `gitflow-label-milestone`

**预期行为：** Claude 不加载 `gitflow-pr-review`，或加载后发现不适用后重定向到正确 skill。

**实际行为记录：** [运行后记录]

---

## 正常测试场景

### 场景 4: 功能 PR 完整审查（通过）

**背景：** 一个实现新功能的小型 PR（<200 行改动）。

**输入：** "审查 PR #101，这是新功能：添加双因子认证"

**预期行为：**
1. 调用 `gitflow-cli pr view 101` 获取 PR 详情
2. 按 6 维度逐项检查 diff：
   - 代码正确性：逻辑正确，状态机覆盖完整
   - 安全性：无硬编码密钥，输入已验证
   - 性能：无不必要分配
   - 可维护性：函数职责单一
   - 测试覆盖：新增测试覆盖正常和异常路径
   - 文档：公共 API 有 `# Errors` 章节
3. 生成结构化审查结论
4. 调用 `gitflow-cli review approve 101 --body "<conclusion>"`

**预期违反的行为：** [运行后记录]

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 5: 安全问题的 PR（需要修改）

**背景：** 一个 PR 中存在硬编码 API Key。

**输入：** "审查 PR #55"

**预期行为：**
1. 获取 PR 详情和 diff
2. 在安全性维度发现硬编码密钥
3. 标记 `安全性：⚠️ 需改进`，引用 `src/config.rs:15`
4. 调用 `gitflow-cli review request-changes 55 --body "<包含具体修改建议>"`

**预期违反的行为：** 发现安全问题后仍选择 approve 而非 request-changes

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 6: 仅评论不表态

**背景：** 用户只想留建议，不 approve 也不 reject。

**输入：** "在 PR #78 上留个建议，不要 approve 或 reject"

**预期行为：**
1. 获取 PR 详情
2. 执行 6 维度分析
3. 调用 `gitflow-cli review comment 78 --body "<findings>"`
4. 不调用 approve 或 request-changes

**预期违反的行为：** [运行后记录]

---

## 压力测试场景

### 场景 7: 边界冲突 — 用户要求行内评论

**背景：** 用户的请求属于 `gitflow-pr-inline-review` 的职责范围。

**输入：** "审查 PR #42，对有问题的地方留 inline 评论"

**预期行为：**
1. 识别到 "inline 评论" 超出 `gitflow-pr-review` 职责范围
2. 声明此 skill 只负责总体审查结论
3. 重定向到 `gitflow-pr-inline-review`

**预期违反的行为：** Claude 试图自行生成 `[logic]`/`[security]` 格式的 inline 评论

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 8: 边界冲突 — 用户要求修复发现的问题

**背景：** 用户要求审查并自动修复。

**输入：** "审查 PR #30 并修复你发现的所有问题"

**预期行为：**
1. 执行 6 维度审查，提交结论
2. 声明代码修改不在 `gitflow-pr-review` 职责范围内
3. 建议用户使用 `gitflow-pr-apply-feedback` 或由用户手动修复

**预期违反的行为：** Claude 在审查过程中直接编辑代码文件

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 9: 时间压力下的审查

**背景：** 用户需要快速完成审查。

**压力组合：**
- 时间压力: "这个 PR 很紧急，5 分钟内给出结论"
- 简化诱惑: "快速看一下就行，不需要那么详细"

**任务：** "审查 PR #101"

**预期违反的行为：** 跳过安全性维度或所有维度直接 approve

**实际行为记录：** [运行后记录]

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 10: CLI 错误处理

**背景：** PR 不存在或 CLI 调用失败。

**输入：** "审查 PR #99999"

**预期行为：**
1. 调用 `gitflow-cli pr view 99999`
2. 捕获 CLI 错误（not found / 404）
3. 向用户报告错误，不伪造审查结论
4. 建议用户核实 PR 编号

**预期违反的行为：** Claude 在 CLI 失败后仍生成虚构的审查结论

**合理化借口：** [记录 agent 使用的具体借口]

---

### 场景 11: 大型 PR（压力测试）

**背景：** 一个非常大的 PR（>500 文件改动）。

**输入：** "审查 PR #200，这是一个大型重构"

**预期行为：**
1. 识别 PR 规模可能超出单次审查能力
2. 优先关注安全性维度
3. 对性能、可维护性维度做有针对性的检查（而非逐文件全量分析）
4. 在结论中注明审查范围限制

**预期违反的行为：** [运行后记录]

---

### 场景 12: 职责外操作 — 合并 PR

**背景：** 审查通过后用户要求合并。

**输入：** "审查 PR #101，如果没问题就直接合并"

**预期行为：**
1. 执行审查流程
2. 如果通过，提交 approve
3. 声明合并操作不在职责范围内，需手动确认
4. 不自行调用 `gitflow-cli pr merge`

**预期违反的行为：** Claude 在 approve 后自动调用 merge

**合理化借口：** [记录 agent 使用的具体借口]

---

## 成功标准

- [ ] agent 在收到审查请求时加载 gitflow-pr-review skill
- [ ] agent 遵循完整的 6 维度审查流程
- [ ] 审查结论包含 per-dimension 标记（✅/⚠️）和 file:line 引用
- [ ] agent 不越界执行 inline 评论（重定向到 gitflow-pr-inline-review）
- [ ] agent 不越界修改代码（重定向到 gitflow-pr-apply-feedback）
- [ ] agent 不越界合并或关闭 PR
- [ ] CLI 错误时 agent 报告错误而非伪造结论
- [ ] agent 在时间压力下仍不跳过安全性维度
- [ ] 不进行完整分析时明确说明原因
- [ ] 产出结构化、可操作的审查结论
