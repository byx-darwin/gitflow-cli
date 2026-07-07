# PR Review Checklist (6 Dimensions)

> **Source:** Externalized from `gitflow-pr-review` skill.
> **Purpose:** Per-dimension review items to cite during PR assessments.

---

## 1. 代码正确性 (Correctness)

- [ ] 逻辑实现是否符合预期功能和 spec
- [ ] 边界条件和异常路径是否正确处理
- [ ] 是否存在 off-by-one 错误、空指针、未初始化变量等常见问题
- [ ] 状态机/条件分支是否覆盖所有情况
- [ ] 是否存在竞态条件或死锁风险（并发代码）

## 2. 安全性 (Security)

- [ ] 是否存在密钥/Token/API Key 硬编码
- [ ] 用户输入是否经过验证/清洗后再使用
- [ ] 是否存在 SQL 注入、命令注入、路径遍历风险
- [ ] 敏感信息是否已脱敏（日志、错误消息、调试输出）
- [ ] 认证/授权检查是否完整
- [ ] 是否使用了不安全的加密算法（如 MD5、SHA1 用于安全场景）

## 3. 性能 (Performance)

- [ ] 是否存在不必要的内存分配或克隆
- [ ] 循环内是否有 I/O 操作或数据库查询
- [ ] 是否有可以预分配容量的集合（`Vec::with_capacity` 等）
- [ ] 异步代码是否正确使用 `spawn_blocking` 处理阻塞操作
- [ ] 是否有未关闭的资源或连接泄漏

## 4. 可维护性 (Maintainability)

- [ ] 代码是否遵循 SOLID 原则和 DRY 原则
- [ ] 函数/方法是否职责单一、长度合理
- [ ] 变量命名是否清晰、符合命名约定
- [ ] 是否有过度嵌套或过长的条件链
- [ ] 依赖引入是否合理，无循环依赖
- [ ] 错误处理是否使用 `Result` 而非 `Option` 隐藏错误

## 5. 测试覆盖 (Test Coverage)

- [ ] 新增功能/修复是否有对应的测试用例
- [ ] 测试是否覆盖了正常路径和异常路径
- [ ] 是否有边界条件/极端情况的测试
- [ ] 测试命名是否清晰（`test_should_<expected_behavior>`）
- [ ] 是否存在 flaky 测试或不合理的 `#[ignore]`

## 6. 文档 (Documentation)

- [ ] 公共 API 是否有文档注释
- [ ] 文档是否包含 `# Errors`、`# Panics`、`# Safety` 等必要章节
- [ ] 模块级文档注释是否完整
- [ ] 示例代码是否正确、可运行
- [ ] `CLAUDE.md` / `specs/` 等项目文档是否同步更新

---

## Distinction from `gitflow-pr-inline-review`

| Aspect | `gitflow-pr-review` (this document) | `gitflow-pr-inline-review` |
|--------|-------------------------------------|----------------------------|
| Scope | Overall PR verdict | Per-line inline comments on diff |
| Output | `review approve` / `request-changes` / `comment` | `comment <sha> --path <f> --line <l>` |
| Dimensions | 6 (correctness, security, performance, maintainability, tests, docs) | 4 tags: `[logic]` `[security]` `[naming]` `[boundary]` |
| Decision | Submits a review decision | No decision — publishes line-level feedback |
| Workflow | Analyze → conclude → submit verdict | Analyze → draft → user confirm → publish |
