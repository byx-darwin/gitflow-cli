# E2E 测试环境配置指南

## 概述

本文档说明如何配置 E2E 测试所需的环境，包括测试仓库、访问令牌和 CI Secrets。

---

## 1. 创建 GitHub 测试仓库

### 步骤

1. **登录 GitHub**
   - 访问 https://github.com
   - 使用 `byx-darwin` 账户登录

2. **创建新仓库**
   - 点击右上角 "+" → "New repository"
   - 仓库名称：`e2e-test-repo`
   - 描述：`E2E test repository for gitflow-cli`
   - 可见性：**Public**（或 Private，根据需求）
   - 初始化：**仅添加 README**
   - 点击 "Create repository"

3. **启用功能**
   - 进入仓库 → Settings → Features
   - 确保启用：
     - ✅ Issues
     - ✅ Pull requests
     - ✅ Releases

---

## 2. 创建 GitHub 访问令牌

### 步骤

1. **进入开发者设置**
   - GitHub → Settings → Developer settings
   - 或访问 https://github.com/settings/tokens

2. **创建个人访问令牌（Classic）**
   - 点击 "Tokens (classic)" → "Generate new token" → "Generate new token (classic)"
   - 名称：`e2e-test-token`
   - 过期时间：**90 天**（建议定期轮换）
   - 权限范围（勾选）：
     - ✅ `repo` (Full control of private repositories)
       - ✅ `repo:status`
       - ✅ `repo_deployment`
       - ✅ `public_repo`
       - ✅ `repo:invite`
       - ✅ `security_events`
     - ✅ `write:packages` (Upload packages to registry)
     - ✅ `delete:packages` (Delete packages)

3. **生成令牌**
   - 滚动到页面底部，点击 "Generate token"
   - **立即复制令牌**（格式：`ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`）
   - 安全保存（不会再次显示）

---

## 3. 配置 GitHub Secrets（CI 使用）

### 步骤

1. **进入仓库设置**
   - 访问 https://github.com/byx-darwin/gitflow-cli
   - 点击 "Settings" → "Secrets and variables" → "Actions"

2. **添加 Repository Secrets**
   - 点击 "New repository secret"
   - 添加以下 Secrets：

   | Name | Value |
   |------|-------|
   | `E2E_TEST_REPO` | `byx-darwin/e2e-test-repo` |
   | `E2E_GITHUB_TOKEN` | `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`（上一步生成的令牌） |

3. **验证配置**
   - 确认 Secrets 已添加（值会被隐藏）

---

## 4. 本地测试验证

### 设置环境变量

```bash
# 临时设置（当前会话）
export E2E_TEST_REPO="byx-darwin/e2e-test-repo"
export E2E_GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# 或创建 .env 文件（不要提交到 git）
cat > .env << EOF
E2E_TEST_REPO=byx-darwin/e2e-test-repo
E2E_GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
EOF
```

### 运行测试

```bash
# 运行所有 E2E 测试
cargo test -p e2e-github

# 或运行特定测试
cargo test -p e2e-github --test auth
cargo test -p e2e-github --test issue
cargo test -p e2e-github --test pr
```

---

## 5. CI 验证

### 触发 CI

1. **推送代码到 main 分支**
   ```bash
   git push origin main
   ```

2. **或创建 PR**
   ```bash
   gh pr create --title "test: add E2E test framework" --body "Add E2E test framework for non-interactive mode"
   ```

3. **查看 CI 结果**
   - 访问 https://github.com/byx-darwin/gitflow-cli/actions
   - 查看 "E2E Tests" workflow 的运行结果

---

## 6. 测试仓库维护

### 定期清理

测试仓库会积累测试创建的 Issue、PR、Label 等资源。建议定期清理：

```bash
# 清理已关闭的 Issue（使用 GitHub CLI）
gh issue list --repo byx-darwin/e2e-test-repo --state closed --limit 100 | \
  awk '{print $1}' | xargs -I {} gh issue delete {} --repo byx-darwin/e2e-test-repo --yes

# 清理已合并的 PR
gh pr list --repo byx-darwin/e2e-test-repo --state merged --limit 100 | \
  awk '{print $1}' | xargs -I {} gh pr delete {} --repo byx-darwin/e2e-test-repo --yes
```

### 自动化清理（可选）

可以创建 GitHub Actions workflow 定期清理：

```yaml
# .github/workflows/cleanup-test-repo.yml
name: Cleanup Test Repository

on:
  schedule:
    - cron: '0 0 * * 0'  # 每周日凌晨
  workflow_dispatch:

jobs:
  cleanup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Cleanup closed issues
        run: |
          gh issue list --repo ${{ secrets.E2E_TEST_REPO }} --state closed --limit 100 | \
            awk '{print $1}' | xargs -I {} gh issue delete {} --repo ${{ secrets.E2E_TEST_REPO }} --yes
        env:
          GITHUB_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
```

---

## 7. 故障排查

### 问题：测试失败 "Missing required environment variable: E2E_TEST_REPO"

**解决**：
```bash
export E2E_TEST_REPO="byx-darwin/e2e-test-repo"
export E2E_GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

### 问题：测试失败 "401 Unauthorized"

**原因**：令牌无效或权限不足

**解决**：
1. 验证令牌是否过期
2. 检查令牌权限是否包含 `repo`
3. 重新生成令牌

### 问题：CI 失败 "Secret not found"

**原因**：GitHub Secrets 未配置

**解决**：
1. 检查仓库 Settings → Secrets → Actions
2. 确认 `E2E_TEST_REPO` 和 `E2E_GITHUB_TOKEN` 已添加

---

## 8. 安全注意事项

1. **不要提交令牌**
   - `.env` 文件已在 `.gitignore` 中
   - 不要在代码中硬编码令牌
   - 不要在日志或错误消息中暴露令牌

2. **定期轮换令牌**
   - 建议每 90 天轮换一次
   - 轮换后更新 GitHub Secrets

3. **限制令牌权限**
   - 仅授予必要的权限
   - 不要使用 `admin` 权限

---

## 9. 相关资源

- [GitHub Actions Secrets 文档](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [GitHub 个人访问令牌](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- [gitflow-cli E2E 测试设计文档](./docs/superpowers/specs/2026-07-09-e2e-noninteractive-test-design.md)

---

**最后更新**: 2026-07-09
**维护者**: gitflow-cli 团队
