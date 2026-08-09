//! GEO 实体一致性守护测试
//!
//! 检查规范一句话定位在 Cargo.toml、llms.txt、llms-full.txt 中的逐字一致性。

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "守护测试读取已知存在的仓库内工件文件"
)]

use std::fs;
use std::path::PathBuf;

/// 全渠道逐字一致的规范一句话定位。
const CANONICAL_POSITIONING: &str =
    "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

#[test]
fn test_should_keep_canonical_positioning_in_llms_txt() {
    let content = read("website/public/llms.txt");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "llms.txt 必须包含规范一句话定位"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_llms_full_txt() {
    let content = read("website/public/llms-full.txt");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "llms-full.txt 必须包含规范一句话定位"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_cli_cargo_toml() {
    let content = read("apps/cli/Cargo.toml");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "apps/cli/Cargo.toml description 必须以规范一句话定位开头"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_jsonld_generator() {
    let content = read("website/src/lib/jsonld.ts");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "jsonld.ts 常量必须等于规范一句话定位"
    );
}

#[test]
fn test_should_not_contain_template_placeholders() {
    let files = [
        "Cargo.toml",
        "apps/cli/Cargo.toml",
        "crates/core/Cargo.toml",
        "website/public/llms.txt",
        "website/public/llms-full.txt",
    ];
    let placeholders = ["Your Name", "yourdomain", "{{version}}"];

    for file in files {
        let content = read(file);
        for placeholder in &placeholders {
            assert!(
                !content.contains(placeholder),
                "{file} 包含模板占位符: {placeholder}"
            );
        }
    }
}
