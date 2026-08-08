# Design: gf update Release Binary Signature Verification

**Issue:** #152
**Date:** 2026-08-09
**Status:** Approved

## Summary

为 `gf update` 下载 release binary 时增加 ed25519 签名校验，确保下载的 binary 来自官方发布者且未被篡改。校验失败时中止更新，不替换当前 binary。

## Background

`gf update`（PR #150）使用 `self_update` 0.42 的 GitHub 后端下载并替换 binary，当前仅依赖 HTTPS TLS 传输完整性，未校验发布资产的签名或校验和。Release 工作流已发布 `checksums.txt`，但未被消费。SHA-256 校验和只能证明传输完整性，无法防止恶意篡改（攻击者若能修改 release 资产，也能修改 `checksums.txt`）。

## Design Decisions

| 决策 | 选择 | 理由 |
|------|------|------|
| 信任模型 | ed25519 签名 | 强信任链，只有私钥持有者能生成有效签名 |
| 签名目标 | 每个 archive（.tar.gz / .zip） | self_update 原生支持，无需自定义校验逻辑 |
| 跳过校验 | 编译期 feature flag `skip-verify` | 官方构建始终校验，开发构建可跳过 |
| 密钥管理 | 一次性生成，私钥存 GitHub Secret，公钥嵌入源码 | 简单，适合单维护者项目 |
| 签名工具 | 自写 Rust binary（ed25519-dalek） | 与 self_update 签名格式兼容 |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    CI (GitHub Actions)               │
│                                                      │
│  build archives → sign-release 工具签名 → 上传       │
│       │              │                    │          │
│  gf-*.tar.gz   gf-*.tar.gz.sig    checksums.txt     │
└─────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│              gf update (客户端)                      │
│                                                      │
│  1. 查询最新版本                                      │
│  2. 下载 archive + .sig                               │
│  3. 用编译期嵌入的公钥验证 ed25519 签名                 │
│  4. 验证通过 → 替换 binary                            │
│     验证失败 → 中止，不替换，输出错误                    │
└─────────────────────────────────────────────────────┘
```

## Trust Chain

1. 密钥对一次性生成（ed25519-dalek）
2. **私钥** → GitHub Actions Secret (`RELEASE_SIGNING_PRIVATE_KEY`)，hex 编码存储
3. **公钥** → 硬编码在 Rust 源码中（`const VERIFYING_KEY: [u8; 32]`）
4. CI 用私钥签名每个 archive → 生成 `.sig` 文件
5. `gf update` 用编译期嵌入的公钥验证 `.sig`
6. 验证通过 → self_update 替换 binary；验证失败 → 中止

## Key Management

### 密钥对生成

一次性脚本 `tools/generate-signing-key.rs`：
- 使用 `ed25519-dalek` 生成密钥对
- 输出私钥（hex 编码）→ 手动添加到 GitHub Actions Secrets
- 输出公钥（Rust 常量格式）→ 粘贴到源码

### 私钥存储

- GitHub Actions Secret 名称：`RELEASE_SIGNING_PRIVATE_KEY`
- hex 编码存储，CI 中解码后使用

### 公钥嵌入

```rust
// apps/cli/src/commands/update.rs
#[cfg(not(feature = "skip-verify"))]
const VERIFYING_KEY: [u8; 32] = [/* 公钥字节 */];
```

### 密钥轮换

1. 重新生成密钥对
2. 更新 GitHub Secret（新私钥）
3. 更新源码中的公钥常量
4. 发布新版 gf（包含新公钥）
5. 用新私钥签名后续 release

注意：旧版 gf 使用旧公钥，无法验证新签名。必须先更新 gf 到包含新公钥的版本，再发布用新私钥签名的 release。

## CI Changes

### Release 工作流修改

在 `.github/workflows/release.yml` 的 `release` job 中，"Checksums" 步骤后、"Create GitHub Release" 步骤前，新增签名步骤：

```yaml
- name: Sign release assets
  env:
    RELEASE_SIGNING_PRIVATE_KEY: ${{ secrets.RELEASE_SIGNING_PRIVATE_KEY }}
  run: |
    cargo run --bin sign-release -- sign \
      --key "$RELEASE_SIGNING_PRIVATE_KEY" \
      --input release/ \
      --output release/
```

### 签名工具

新增 `tools/sign-release.rs`（workspace `[[bin]]` target）：
- 读取私钥（hex 编码，从环境变量）
- 遍历目录中所有 `.tar.gz` 和 `.zip` 文件
- 对每个文件生成 `filename.sig`（ed25519 签名）
- 签名算法与 `self_update` 的 `signatures` feature 兼容

### Release 资产变化

```
# 之前
gf-x86_64-apple-darwin.tar.gz
gf-aarch64-apple-darwin.tar.gz
...
checksums.txt

# 之后
gf-x86_64-apple-darwin.tar.gz
gf-x86_64-apple-darwin.tar.gz.sig    ← 新增
gf-aarch64-apple-darwin.tar.gz
gf-aarch64-apple-darwin.tar.gz.sig    ← 新增
...
checksums.txt
```

## Client Changes

### Cargo 配置

`self_update` 的 `signatures` feature 始终启用。`skip-verify` feature 控制是否在构建时调用 `.verifying_keys()`：
- 未启用 `skip-verify`：调用 `.verifying_keys(&[VERIFYING_KEY])` → self_update 下载并验证 `.sig`
- 启用 `skip-verify`：不调用 `.verifying_keys()` → self_update 不查找 `.sig`，行为与当前一致

```toml
# apps/cli/Cargo.toml
[features]
default = []
skip-verify = []

# workspace Cargo.toml
# self_update = { version = "0.42", default-features = false, features = ["rustls", "compression-flate2", "signatures"] }
```

### update.rs 变更

```rust
#[cfg(not(feature = "skip-verify"))]
const VERIFYING_KEY: [u8; 32] = [/* 公钥字节 */];

let mut builder = self_update::backends::github::Update::configure()
    .repo_owner(REPO_OWNER)
    .repo_name(REPO_NAME)
    .bin_name(BIN_NAME)
    .current_version(&current)
    .target_version_tag(&format!("v{latest}"))
    .target(&target)
    .show_download_progress(true)
    .show_output(true)
    .no_confirm(true);

#[cfg(not(feature = "skip-verify"))]
{
    builder = builder.verifying_keys(&[VERIFYING_KEY]);
}

let status = builder.build()?.update()?;
```

### 行为矩阵

| 场景 | 行为 |
|------|------|
| 官方 release + 默认编译 | 下载 archive + .sig → 验证签名 → 通过则替换 |
| 官方 release + 签名校验失败 | 中止，不替换 binary，输出错误 |
| 自建 release（无 .sig） + 默认编译 | 校验失败（找不到 .sig），中止 |
| 任意 release + `--features skip-verify` 编译 | 跳过校验，直接替换（与当前行为一致） |

## Error Handling

| 错误场景 | 处理方式 |
|----------|----------|
| `.sig` 文件不存在 | 输出 `错误: 未找到签名文件，无法验证完整性`，中止 |
| 签名不匹配（篡改或损坏） | 输出 `错误: 签名校验失败，release 资产可能已被篡改`，中止 |
| 公钥未嵌入（不应发生） | 编译期错误，`skip-verify` 未启用时公钥必须存在 |
| self_update 下载失败 | 现有错误处理不变 |

关键原则：校验失败时绝对不替换 binary。self_update 的签名校验在替换前执行，由 self_update 保证此原则。

## Testing Strategy

| 测试类型 | 覆盖内容 |
|----------|----------|
| 单元测试 | 公钥常量存在性、feature flag 条件编译正确性 |
| 集成测试 | 签名工具的 sign → verify 往返测试（临时密钥对） |
| 手工验证（dogfooding） | 发布测试 release，用 `gf update` 实际下载并验证 |
| 不测试 | self_update 内部的签名验证逻辑（上游保证） |

签名工具测试用例：
- 生成临时密钥对 → 签名临时文件 → 用公钥验证 → 应通过
- 篡改文件内容 → 验证 → 应失败
- 错误私钥签名 → 验证 → 应失败

## Acceptance Criteria

- [x] 校验 release 资产签名（ed25519 `.sig` 文件）
- [x] 校验失败时不替换当前 binary，输出错误
- [x] 明确信任链（ed25519 密钥对，私钥存 GitHub Secret，公钥嵌入源码）
- [x] 有绕过机制（编译期 `skip-verify` feature flag）

## Out of Scope

- checksums.txt 校验（ed25519 签名已涵盖完整性 + 真实性）
- Sigstore/cosign 无密钥签名（过于复杂）
- 自动密钥轮换
- 多签名者支持
