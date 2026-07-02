---
name: gitflow-release-helper
description: 发布助手工作流 — 分析自上次 release 以来的 git log，按 conventional commits 分组生成 Release Note，调用 gitflow-cli release create 创建发布并输出 release URL
---

# gitflow-cli release helper 工作流

引导用户完成版本发布流程。通过分析自上次 release 以来的 git 提交历史，按照 conventional commits 规范自动分组生成 Release Note，调用 `gitflow-cli release create` 命令创建发布，并输出 release URL 供团队确认。

## 工作流

### 步骤 1：确定版本号

首先需要确定本次发布的版本号。参考以下方式：

#### 1.1 获取当前最新版本

查看当前最新的 release tag：

```bash
git describe --tags --abbrev=0
```

或查看所有 tags：

```bash
git tag --sort=-v:refname | head -20
```

#### 1.2 根据变更类型推断版本升级

分析自上次 release 以来的 commit 类型，按照 SemVer 规则推断版本升级：

| Commit 类型 | 版本升级 | 示例 |
|-------------|----------|------|
| `feat!:` 或 `BREAKING CHANGE` | Major（X.0.0） | 1.0.0 → 2.0.0 |
| `feat:` | Minor（x.Y.0） | 1.2.0 → 1.3.0 |
| `fix:`, `perf:`, `refactor:` | Patch（x.y.Z） | 1.2.3 → 1.2.4 |
| `docs:`, `chore:`, `ci:`, `test:` | 不触发发布或 Patch | 视项目策略 |

#### 1.3 确认版本号

向用户确认本次发布的版本号：

```
当前版本: v1.2.3
自上次发布以来包含 breaking change（feat!: ...）
建议新版本: v2.0.0
请确认版本号，或输入自定义版本号：
```

### 步骤 2：分析 Git Log 获取变更

获取自上次 release 以来的所有 commits：

```bash
git log <last-tag>..HEAD --pretty=format:"%h %s" --no-merges
```

或使用 gitflow 提供的命令（如支持）：

```bash
gitflow-cli release changelog --since <last-tag>
```

### 步骤 3：按 Conventional Commits 分组生成 Release Note

将 commits 按 conventional commits 类型分组，生成结构化的 Release Note。

**分组规则：**

| Commit 前缀 | Release Note 分组 | 说明 |
|-------------|-------------------|------|
| `feat` | ✨ Features | 新功能 |
| `fix` | 🐛 Bug Fixes | 缺陷修复 |
| `perf` | ⚡ Performance | 性能优化 |
| `refactor` | ♻️ Code Refactoring | 代码重构 |
| `docs` | 📝 Documentation | 文档变更 |
| `test` | ✅ Tests | 测试相关 |
| `ci` / `build` | 🔧 CI / Build | 构建和 CI 变更 |
| `chore` | 🧹 Chores | 杂项维护 |

**Breaking Changes** 单独置顶，无论属于哪种类型。

**Release Note 模板：**

```markdown
# Release <version>

## ⚠️ Breaking Changes

- <breaking change 描述> (<commit-hash>)

## ✨ Features

- <feature 描述> (<commit-hash>)
  - <相关 issue 链接，如有>

## 🐛 Bug Fixes

- <fix 描述> (<commit-hash>)

## ⚡ Performance

- <perf 描述> (<commit-hash>)

## ♻️ Code Refactoring

- <refactor 描述> (<commit-hash>)

## 📝 Documentation

- <docs 描述> (<commit-hash>)

## 🔧 CI / Build

- <ci/build 描述> (<commit-hash>)

## 📊 Statistics

- **Total commits:** <n>
- **Contributors:** <list>
- **Files changed:** <n>
```

### 步骤 4：确认 Release Note

向用户展示生成的 Release Note，确认内容是否准确。用户可以：

- 修改或补充内容
- 调整分组
- 添加额外的发布说明（如升级指南、已知问题）

### 步骤 5：创建 Release

调用 `gitflow-cli release create` 创建发布：

```bash
gitflow-cli release create --tag <vX.Y.Z> --notes "<release-note>"
```

如果 Release Note 内容较长，可以使用 `--notes-file` 参数从文件读取：

```bash
cat > /tmp/release-note.md << 'EOF'
<!-- Release Note 内容 -->
EOF

gitflow-cli release create --tag <vX.Y.Z> --notes-file /tmp/release-note.md
```

### 步骤 6：输出 Release URL

解析命令输出，提取并展示 Release URL：

```markdown
## ✅ Release 创建成功！

**版本:** <vX.Y.Z>
**Tag:** <tag-name>
**Release URL:** <url>
**Release Note:** 已附加到 release 页面
```

### 步骤 7：清理临时文件

删除临时的 Release Note 文件：

```bash
rm -f /tmp/release-note.md
```

## 使用示例

### 发布一个 minor 版本（包含新功能和修复）

```bash
# 查看最新版本
git describe --tags --abbrev=0
# 输出: v1.2.3

# 获取自上次发布以来的 commits
git log v1.2.3..HEAD --pretty=format:"%h %s" --no-merges
# 输出:
# a1b2c3d feat(cli): add --dry-run flag to pr create
# e4f5g6h fix(auth): handle expired token gracefully
# i7j8k9l docs: update CLAUDE.md with new lint rules
# m0n1o2p feat(api): support batch issue creation
# q3r4s5t chore(deps): bump serde from 1.0.190 to 1.0.195

# 分析：包含 2 个 feat，建议 minor 升级 → v1.3.0

# 生成 Release Note
cat > /tmp/release-note.md << 'EOF'
# Release v1.3.0

## ✨ Features

- add --dry-run flag to pr create (a1b2c3d)
- support batch issue creation (m0n1o2p)
  - resolves #42

## 🐛 Bug Fixes

- handle expired token gracefully (e4f5g6h)
  - resolves #38

## 📝 Documentation

- update CLAUDE.md with new lint rules (i7j8k9l)

## 🧹 Chores

- bump serde from 1.0.190 to 1.0.195 (q3r4s5t)

## 📊 Statistics

- **Total commits:** 5
- **Contributors:** @alice, @bob
- **Files changed:** 12
EOF

# 创建 release
gitflow-cli release create --tag v1.3.0 --notes-file /tmp/release-note.md
# 输出: Release created: https://github.com/org/repo/releases/tag/v1.3.0

rm -f /tmp/release-note.md
```

### 发布一个包含 Breaking Change 的 major 版本

```bash
git log v1.3.0..HEAD --pretty=format:"%h %s" --no-merges
# 输出:
# x1y2z3a feat!: redesign authentication API
# b4c5d6e fix(auth): correct token refresh logic

# 有 breaking change → major 升级 → v2.0.0

cat > /tmp/release-note.md << 'EOF'
# Release v2.0.0

## ⚠️ Breaking Changes

- **redesign authentication API**: The `Auth::login` method now returns `Result<Session>` instead of `Option<Session>`. All callers need to handle the new error type. (x1y2z3a)

  **Migration guide:**
  ```rust
  // Before:
  if let Some(session) = auth.login(&credentials) { ... }

  // After:
  match auth.login(&credentials) {
      Ok(session) => { ... }
      Err(e) => eprintln!("Login failed: {e}"),
  }
  ```

## 🐛 Bug Fixes

- correct token refresh logic (b4c5d6e)

## 📊 Statistics

- **Total commits:** 2
- **Contributors:** @alice
- **Files changed:** 8
EOF

gitflow-cli release create --tag v2.0.0 --notes-file /tmp/release-note.md
rm -f /tmp/release-note.md
```

### 快速发布一个 patch 版本（仅修复）

```bash
git log v2.0.0..HEAD --pretty=format:"%h %s" --no-merges
# 输出:
# f1g2h3i fix(cli): prevent crash on empty input

# 仅 fix → patch 升级 → v2.0.1

cat > /tmp/release-note.md << 'EOF'
# Release v2.0.1

## 🐛 Bug Fixes

- prevent crash on empty input (f1g2h3i)
  - resolves #55

## 📊 Statistics

- **Total commits:** 1
- **Contributors:** @bob
- **Files changed:** 2
EOF

gitflow-cli release create --tag v2.0.1 --notes-file /tmp/release-note.md
rm -f /tmp/release-note.md
```

## 注意事项

- 版本号必须遵循 SemVer 规范，breaking change 必须 major 升级
- Release Note 中的 breaking changes 应放在最前面，并提供迁移指南
- 每个 commit 条目应附上 commit hash（短格式即可），便于追溯
- 如果 commit 关联了 Issue，应在条目中附上 issue 链接
- 对于非 conventional commit 格式的提交，应根据实际内容手动归类
- Release Note 内容较长时，优先使用 `--notes-file` 参数从文件读取，避免 shell 转义问题
- 创建 release 前应确保所有 CI 检查通过、main 分支处于最新状态
- 发布后应通知团队成员和相关利益方
