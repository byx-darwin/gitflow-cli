//! 跨工件守护测试：元数据卫生、GEO/实体一致性、演示资产。
//!
//! 这些测试在 CI 中长期看护 1.0 发布物的一致性，防止模板占位符回归。
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_methods,
    reason = "守护测试读取已知存在的仓库内工件文件"
)]

use std::fs;
use std::path::PathBuf;

/// 全渠道逐字一致的规范一句话定位。
#[allow(dead_code, reason = "reserved for subsequent guardian tests")]
const CANONICAL_POSITIONING: &str = "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

/// 解析仓库根目录（`apps/cli` 的上两级）。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// 读取仓库根下相对路径文件的 UTF-8 文本。
fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

/// 需要检查的全部 Cargo 清单相对路径。
fn manifest_paths() -> Vec<&'static str> {
    vec![
        "Cargo.toml",
        "apps/cli/Cargo.toml",
        "crates/core/Cargo.toml",
        "crates/github/Cargo.toml",
        "crates/gitlab/Cargo.toml",
        "crates/gitcode/Cargo.toml",
    ]
}

#[test]
fn test_should_not_contain_template_placeholders() {
    for rel in manifest_paths() {
        let raw = read(rel);
        assert!(!raw.contains("Your Name"), "{rel} still contains 'Your Name' placeholder");
        assert!(!raw.contains("yourdomain"), "{rel} still contains 'yourdomain' placeholder");
        assert!(!raw.contains("{{version}}"), "{rel} contains unrendered template variable");

        let doc: toml::Value = raw.parse().unwrap_or_else(|e| panic!("{rel} is not valid TOML: {e}"));

        if rel == "Cargo.toml" {
            let pkg = &doc["workspace"]["package"];
            let authors = pkg["authors"].as_array().expect("workspace authors must be an array");
            assert!(!authors.is_empty(), "workspace authors empty");
            let homepage = pkg["homepage"].as_str().expect("workspace homepage missing");
            assert!(homepage.starts_with("https://"), "workspace homepage must be https URL");
            assert!(!homepage.contains("yourdomain"), "workspace homepage has placeholder");
        } else {
            let pkg = &doc["package"];
            let desc = pkg["description"].as_str().unwrap_or_else(|| panic!("{rel}: description missing"));
            assert!(!desc.trim().is_empty(), "{rel}: description empty");
            if let Some(d) = pkg["documentation"].as_str() {
                assert!(
                    d.starts_with("https://docs.rs/"),
                    "{rel}: documentation must point to docs.rs, got {d}"
                );
            }
        }
    }
}
