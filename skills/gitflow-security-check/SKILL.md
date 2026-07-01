---
name: gitflow-security-check
description: 安全审计工作流 — 检查密钥硬编码、依赖漏洞和输入验证，调用 cargo audit 执行依赖漏洞扫描
---

# gitflow security check 工作流

提供一套系统的安全审计 checklist，帮助识别项目中的安全漏洞，包括密钥硬编码、依赖漏洞、输入验证等问题。

## 审计清单

### 1. 密钥硬编码检查

扫描代码仓库中的敏感信息：

- [ ] API Keys / Access Tokens（GitHub Token、AWS Key、数据库密码等）
- [ ] 私钥文件（`.pem`、`.key` 等）
- [ ] 数据库连接字符串包含明文密码
- [ ] 加密密钥硬编码在源码中
- [ ] `.env` 文件是否被 `.gitignore` 排除
- [ ] 是否存在 `config/` 或 `secrets/` 目录被误提交

扫描方法：

```bash
# 搜索常见密钥模式
grep -rn "password\|secret\|api_key\|token\s*=\s*['\"]" --include="*.rs" --include="*.toml" --include="*.yaml" --include="*.yml" --include="*.json" src/
# 检查是否有 .env 文件被跟踪
git ls-files | grep -E "\.env|\.env\."
```

### 2. 依赖漏洞扫描

检查项目依赖是否存在已知安全漏洞：

```bash
cargo audit
```

重点关注：
- CRITICAL 和 HIGH 级别的漏洞
- 是否有可用的修复版本
- 是否可通过 `cargo update -p <crate>` 升级修复

### 3. 输入验证检查

确认所有外部输入在进入系统边界时得到正确验证：

- [ ] HTTP 请求参数/Body 是否有长度限制和类型校验
- [ ] CLI 参数是否有长度和字符白名单限制
- [ ] 文件路径参数是否使用了 `SafePath` 类型验证
- [ ] 数据库查询是否使用参数化 API（禁止字符串拼接）
- [ ] URL 参数是否检查了 scheme 白名单和防止 SSRF
- [ ] 正则表达式是否使用 `regex` 库而非不安全的动态编译
- [ ] 数值计算是否使用 `checked_*`、`saturating_*` 等防溢出方法

### 4. 错误处理与日志

- [ ] 错误消息中是否泄露内部实现细节
- [ ] 日志中是否包含敏感信息（密码、Token、用户数据）
- [ ] 是否使用 `secrecy` 类型包装敏感字段
- [ ] `Debug` 实现是否对敏感字段做了脱敏

### 5. 认证与授权

- [ ] 所有 API 端点是否进行了认证检查
- [ ] 权限控制是否基于最小权限原则
- [ ] Token 是否有过期时间和刷新机制
- [ ] 是否存在越权访问路径（IDOR 漏洞）

### 6. 依赖策略与供应链

- [ ] 是否使用了 `cargo deny check` 验证许可证合规性
- [ ] 新增依赖是否经过安全审查（维护状态、已知 CVE）
- [ ] 是否避免使用 `unsafe` 代码（项目要求 `#![forbid(unsafe_code)]`）
- [ ] 是否使用了过时的或有 CVE 记录的依赖版本

## 工作流

### 步骤 1：执行依赖漏洞扫描

```bash
cargo audit
```

记录输出中的漏洞数量、严重级别和受影响的 crate。

### 步骤 2：检查密钥硬编码

执行上述 grep 命令，检查是否存在硬编码的敏感信息。

### 步骤 3：检查输入验证

抽查关键模块（认证、数据库访问、文件操作、CLI 参数处理），确认输入边界是否正确验证。

### 步骤 4：检查错误处理和日志

确认错误消息不含敏感信息，日志输出使用结构化字段而非字符串拼接。

### 步骤 5：汇总审计结果

生成审计报告，按严重级别分类：

```markdown
## 安全审计报告

### 依赖漏洞

| 严重级别 | Crate | CVE | 修复版本 | 状态 |
|---------|-------|-----|---------|------|
| HIGH | openssl | CVE-2023-XXXX | 3.1.0 | ⚠️ 待修复 |

### 密钥硬编码

| 文件 | 行号 | 问题 | 状态 |
|------|------|------|------|
| src/config.rs | 15 | 测试用 API Key 硬编码 | ✅ 已在测试环境白名单 |

### 输入验证

| 模块 | 问题 | 状态 |
|------|------|------|
| auth | ✅ 所有输入使用 SafePath | ✅ 通过 |

### 总结
<!-- 总体安全评估和修复建议 -->
```

### 步骤 6：修复建议

对于发现的问题，提供具体的修复建议：

- 依赖漏洞：`cargo update -p <crate>` 升级至修复版本
- 密钥硬编码：移至环境变量或密钥管理服务
- 输入验证缺失：使用 `SafePath`、参数化查询、长度限制等

## 使用示例

### 快速安全检查

```bash
# 1. 依赖漏洞扫描
cargo audit

# 2. 许可证合规检查
cargo deny check

# 3. 搜索硬编码密钥
grep -rn "api_key\s*=\s*['\"]" --include="*.rs" src/
```

### 完整安全审计

```bash
# 运行完整审计流程
cargo audit
cargo deny check
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic

# 检查 unsafe 代码使用情况
grep -rn "unsafe" --include="*.rs" src/
```

### 处理依赖漏洞

```bash
# 发现漏洞后，查看详细信息
cargo audit

# 升级到修复版本
cargo update -p vulnerable-crate

# 重新验证
cargo audit
```

## 注意事项

- `cargo audit` 依赖 `rustsec-advisory-db`，首次运行会自动下载，后续需定期更新
- 定期运行安全审计（建议合并到 CI/CD pipeline）
- 测试中的硬编码值应在 `#[cfg(test)]` 模块内，并使用 `#[allow(dead_code)]` 标注
- 对于无法立即修复的漏洞，应在 `audit.toml` 中记录忽略原因和修复计划
- 审计结果应记录在项目文档中，方便后续追踪
