# fix(release): release.toml 模板占位符修复设计

## 问题

v1.0.0 发布时发现 `release.toml` 的模板占位符与 cargo-release 1.1.3 不匹配：

- `{version}`（单花括号）→ 未替换，commit subject 出现字面量 `chore: release v{version}`，tag 创建出现字面量 `v{version}`
- `{{version}}`（双花括号）→ 历史提交 `9331bfa` 也报告未替换

## 根因分析

通过实测验证（cargo-release 1.1.3）：

| 模板语法 | 提交信息 | 标签 | 结果 |
|---------|---------|------|------|
| `{{version}}`（双花括号）| `chore: release v0.1.1` ✅ | `v0.1.1` ✅ | **有效** |
| `{version}`（单花括号）| `chore: release v{version}` ❌ | `v{version}` ❌ | **无效** |

**结论：** cargo-release 1.1.3 的正确语法是 `{{version}}`（双花括号，Mustache 风格）。历史提交 `9331bfa` 的失败可能由其他因素导致（如旧版 cargo-release 或环境问题）。

## 修复方案

### 1. 核心修复：release.toml 模板语法

**修改文件：** `release.toml`

**变更内容：**

```diff
- tag-name = "v{version}"
- tag-message = "Release v{version}"
- pre-release-commit-message = "chore: release v{version}"
+ tag-name = "v{{version}}"
+ tag-message = "Release v{{version}}"
+ pre-release-commit-message = "chore: release v{{version}}"
```

**验证标准：**
- `cargo release` dry-run 模式下 commit message 和 tag name 包含实际版本号
- 不出现字面量 `{version}` 或 `{{version}}`

### 2. 增强验证：scripts/release.sh

**目标：** 在发布流程中自动检测模板是否被正确替换，防止未来再次出现模板残留问题。

**修改文件：** `scripts/release.sh`

**变更内容：**

1. **扩展模板残留检测**：同时检测单花括号和双花括号
   ```bash
   # 现有：只检测 {{var}}
   TEMPLATE_RESIDUE_PATTERN='\{\{[a-zA-Z_]+\}\}'
   # 新增：同时检测 {var} 和 {{var}}
   TEMPLATE_RESIDUE_PATTERN='\{\{?[a-zA-Z_]+\}\}?'
   ```

2. **在 release 流程的关键节点插入验证**：
   - `cargo release` 执行后、`git push` 前
   - 检查最新 commit message 是否包含模板残留
   - 检查新创建的 tag name 是否包含模板残留
   - 如果检测到残留，**立即中止并回滚**

3. **错误信息增强**：
   ```
   ✗ Template residue detected in commit message: "chore: release v{{version}}"
   ✗ Expected: "chore: release v1.2.3"
   ✗ This usually means release.toml uses incorrect placeholder syntax.
   ✗ For cargo-release 1.1.3+, use {{version}} (double curly braces).
   ```

**验证标准：**
- 使用错误模板时，脚本自动检测并中止
- 使用正确模板时，脚本正常通过
- `bash scripts/release.sh --self-test` 覆盖新增检测逻辑

### 3. 测试策略

**测试内容：**

1. **单元测试（scripts/release.sh --self-test）**
   - 测试 `validate_commit_subject` 函数：
     - ✅ `"chore: release v1.0.0"` → PASS
     - ❌ `"chore: release v{{version}}"` → FAIL（双花括号残留）
     - ❌ `"chore: release v{version}"` → FAIL（单花括号残留）
   - 测试 `validate_tag_name` 函数：
     - ✅ `"v1.0.0"` → PASS
     - ❌ `"v{{version}}"` → FAIL
     - ❌ `"v{version}"` → FAIL

2. **集成测试（make release-rehearse）**
   - 运行完整的 dry-run 发布流程
   - 验证 commit message 和 tag name 被正确替换
   - 确认无模板残留

3. **回归测试（现有 e2e 测试）**
   - 确保现有 e2e 测试不受影响
   - 运行 `make test` 确认所有测试通过

**验证标准：**
- 所有 self-test 用例通过
- `make release-rehearse` 成功完成
- `make test` 全部通过

## 影响范围

- `release.toml`：模板语法修正
- `scripts/release.sh`：增强模板残留检测
- 无 API 变更，无破坏性改动

## 相关

Refs #132 · `release.toml` · `scripts/release.sh`
