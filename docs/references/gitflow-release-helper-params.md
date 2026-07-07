# gitflow-cli release-helper 完整参考

> 本文档为 `gitflow-release-helper` skill 的外部化引用。

## SemVer 版本推断规则

| Commit 前缀 | 版本升级 | 示例 |
|-------------|----------|------|
| `feat!:` 或 `BREAKING CHANGE` | Major（X.0.0） | 1.0.0 → 2.0.0 |
| `feat:` | Minor（x.Y.0） | 1.2.0 → 1.3.0 |
| `fix:`, `perf:`, `refactor:` | Patch（x.y.Z） | 1.2.3 → 1.2.4 |
| `docs:`, `chore:`, `ci:`, `test:` | 无 / Patch | 视项目策略 |

## Conventional Commits → Release Note 分组

| Commit 前缀 | Release Note 分组 |
|-------------|-------------------|
| `feat` | ✨ Features |
| `fix` | 🐛 Bug Fixes |
| `perf` | ⚡ Performance |
| `refactor` | ♻️ Code Refactoring |
| `docs` | 📝 Documentation |
| `test` | ✅ Tests |
| `ci`/`build` | 🔧 CI / Build |
| `chore` | 🧹 Chores |

Breaking Changes 单独置顶。

## Release Note 模板（最小）

```markdown
# Release <version>

## ⚠️ Breaking Changes
- <变更描述> (<短 hash>)

## ✨ Features
- <feature 描述> (<短 hash>)

## 🐛 Bug Fixes
- <fix 描述> (<短 hash>)

## 📊 Statistics
- **Total commits:** <n>
- **Contributors:** <list>
- **Files changed:** <n>
```

## 常用命令速查

```bash
git describe --tags --abbrev=0                       # 当前最新 tag
git tag --sort=-v:refname | head -20                  # 列出近期 tags
git log <last-tag>..HEAD --pretty=format:"%h %s" --no-merges
gitflow-cli release changelog --since <last-tag>     # 若支持
gitflow-cli release create --tag <vX.Y.Z> --notes "..."
```
