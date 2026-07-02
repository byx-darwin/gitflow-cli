---
name: gitflow-issue-review
description: Issue 需求分析工作流 — 获取 Issue 详情，从标题清晰度、描述充分度、验收标准明确度三个维度分析需求完整性，输出改进建议并回写到 Issue 评论
---

# gitflow issue review 工作流

引导用户对指定 Issue 进行结构化的需求完整性分析，从标题清晰度、描述充分度、验收标准明确度三个维度评估，输出分析报告，并将分析结果作为评论回写到 Issue 中，帮助作者完善需求描述。

## 工作流

### 步骤 1：获取 Issue 详情

调用 `gitflow issue view` 获取目标 Issue 的完整信息：

```bash
gitflow issue view <issue-number>
```

记录以下信息用于后续分析：

- 标题
- 描述内容（Markdown）
- 当前标签
- 关联的 Issue 或 PR
- 评论历史

### 步骤 2：三维度需求分析

按照以下三个维度逐项评估 Issue 的质量：

#### 2.1 标题清晰度

评估标题是否：

- 遵循 conventional commits 前缀约定（如 `feat:`, `fix:`, `docs:`）
- 包含足够的上下文信息（如模块名、作用域）
- 能让人快速理解 Issue 的主题
- 长度合理（不过长、不过短）

**质量等级：**
- 🟢 优秀：标题清晰、具体、符合规范
- 🟡 一般：标题能理解但不够具体，缺少作用域
- 🔴 不足：标题模糊、过于笼统或不符合规范

#### 2.2 描述充分度

评估描述是否包含：

- **背景说明**：问题或需求的上下文
- **目标陈述**：完成后应达到的效果
- **实现约束**：技术限制、兼容性要求等（如适用）
- **参考资料**：相关文档、链接、截图（如适用）

**质量等级：**
- 🟢 优秀：包含背景、目标、约束，信息充分
- 🟡 一般：有基本描述但缺少部分关键信息
- 🔴 不足：描述为空、过于简略或只有标题重复

#### 2.3 验收标准明确度

评估验收标准是否：

- 使用 Markdown checkbox（`- [ ]`）格式
- 每条标准具体、可验证
- 覆盖正常路径和异常路径
- 没有歧义或主观性描述

**质量等级：**
- 🟢 优秀：标准具体、可验证、覆盖完整
- 🟡 一般：有标准但不够具体或部分遗漏
- 🔴 不足：无验收标准或标准不可验证

### 步骤 3：生成分析报告

汇总三维度评估结果，生成分析报告。格式：

```markdown
## Issue 需求分析报告

**Issue:** #<number> — <标题>
**分析时间:** <timestamp>

### 评分总览

| 维度 | 等级 | 说明 |
|------|------|------|
| 标题清晰度 | 🟢/🟡/🔴 | 简要说明 |
| 描述充分度 | 🟢/🟡/🔴 | 简要说明 |
| 验收标准明确度 | 🟢/🟡/🔴 | 简要说明 |

### 详细分析

#### 标题清晰度

<!-- 具体分析内容 -->

#### 描述充分度

<!-- 具体分析内容 -->

#### 验收标准明确度

<!-- 具体分析内容 -->

### 改进建议

1. <!-- 具体可操作的改进建议 -->
2. <!-- ... -->

### 建议的完善标题（如需修改）

`<!-- 建议的新标题 -->`

### 建议的补充内容（如需补充）

```markdown
<!-- 建议补充的描述或验收标准 -->
```
```

### 步骤 4：输出分析报告

将分析报告保存为临时文件，供评论使用：

```bash
cat > /tmp/issue-analysis.md << 'EOF'
<!-- 分析报告内容 -->
EOF
```

### 步骤 5：回写评论到 Issue

调用 `gitflow issue comment` 将分析报告作为评论发布到 Issue：

```bash
gitflow issue comment <issue-number> --body-file /tmp/issue-analysis.md
```

### 步骤 6：清理临时文件

删除临时分析报告文件：

```bash
rm -f /tmp/issue-analysis.md
```

## 使用示例

### 分析一个功能需求 Issue

```bash
# 获取 Issue 详情
gitflow issue view 42

# 分析后生成报告并回写评论
cat > /tmp/issue-analysis.md << 'EOF'
## Issue 需求分析报告

**Issue:** #42 — feat(auth): add two-factor authentication
**分析时间:** 2026-07-02

### 评分总览

| 维度 | 等级 | 说明 |
|------|------|------|
| 标题清晰度 | 🟢 优秀 | 标题格式规范，作用域明确 |
| 描述充分度 | 🟡 一般 | 有背景和目标，但缺少技术约束说明 |
| 验收标准明确度 | 🔴 不足 | 缺少验收标准章节 |

### 改进建议

1. 在描述中补充支持的身份验证方式（TOTP、SMS、硬件密钥）
2. 说明与现有 session 管理的兼容方案
3. 添加验收标准章节，至少包含：
   - [ ] 用户可以开启/关闭 2FA
   - [ ] 未开启 2FA 的用户登录后提示设置
   - [ ] 2FA 验证失败时显示错误信息
   - [ ] 恢复码机制正常工作

### 建议的补充内容

```markdown
## 验收标准

- [ ] 用户可以通过 TOTP 方式开启 2FA
- [ ] 登录后未设置 2FA 的用户被引导至设置页面
- [ ] 2FA 验证码错误时显示友好提示
- [ ] 提供一次性恢复码用于紧急登录
- [ ] 管理员可以强制用户启用 2FA
```
EOF

gitflow issue comment 42 --body-file /tmp/issue-analysis.md
rm -f /tmp/issue-analysis.md
```

### 分析一个描述不足的 Bug Issue

```bash
gitflow issue view 15

# Bug Issue 常见问题：缺少复现步骤
cat > /tmp/issue-analysis.md << 'EOF'
## Issue 需求分析报告

**Issue:** #15 — fix(cli): login redirect loops
**分析时间:** 2026-07-02

### 评分总览

| 维度 | 等级 | 说明 |
|------|------|------|
| 标题清晰度 | 🟡 一般 | 缺少作用域（如 `auth` 或 `session`） |
| 描述充分度 | 🔴 不足 | 缺少复现步骤和期望行为 |
| 验收标准明确度 | 🔴 不足 | 无验收标准 |

### 改进建议

1. 标题建议修改为 `fix(auth): login redirect loops on expired token`
2. 补充复现步骤：什么操作触发了循环？
3. 补充环境信息：浏览器、操作系统、gitflow-cli 版本
4. 添加验收标准

### 建议的完善标题

`fix(auth): login redirect loops on expired token`
EOF

gitflow issue comment 15 --body-file /tmp/issue-analysis.md
rm -f /tmp/issue-analysis.md
```

## 注意事项

- 分析报告应客观中立，避免主观评价，专注于可改进的具体点
- 对于已有完善描述的 Issue，可以跳过分析或给出 🟢 全通过的评价
- 回写评论前应先确认分析结论，避免误导 Issue 作者
- 分析应聚焦于需求层面的完整性，不涉及代码实现细节（代码审查属于 PR review 工作流）
- 对于 `question` 或 `discussion` 类 Issue，需求分析的标准可以适当放宽
