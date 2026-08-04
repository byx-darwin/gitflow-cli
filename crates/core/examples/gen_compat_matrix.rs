//! 从 `docs/compatibility-matrix.json` 生成 Markdown 兼容性矩阵。
//!
//! 用法：`cargo run -p gf-core --example gen_compat_matrix`

#![allow(
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::format_push_string,
    reason = "Example binary: panics are acceptable for build-time tooling"
)]

use std::{collections::BTreeMap, fs};

use serde::Deserialize;

/// 兼容性矩阵根结构。
#[derive(Debug, Deserialize)]
struct MatrixRoot {
    /// 最后更新日期。
    updated_at: String,
    /// gf 版本。
    #[serde(rename = "gitflow_cli_version")]
    gf_version: String,
    /// 平台列表。
    platforms: Vec<PlatformEntry>,
}

/// 平台条目。
#[derive(Debug, Deserialize)]
struct PlatformEntry {
    /// 平台名称。
    name: String,
    /// CLI 二进制名。
    cli_binary: String,
    /// 最低版本。
    min_version: String,
    /// 已测试版本。
    tested_versions: Vec<String>,
    /// 功能映射。
    features: BTreeMap<String, bool>,
}

/// 读取 JSON 并生成 Markdown 文件。
fn main() {
    let json = fs::read_to_string("docs/compatibility-matrix.json")
        .expect("failed to read docs/compatibility-matrix.json");
    let root: MatrixRoot = serde_json::from_str(&json).expect("invalid JSON");

    let mut md = String::new();
    md.push_str("# 兼容性矩阵\n\n");
    md.push_str(&format!(
        "> 自动生成，请勿手动编辑。数据源：`docs/compatibility-matrix.json`\n> 更新时间：{} · gf \
         v{}\n\n",
        root.updated_at, root.gf_version
    ));
    md.push_str("| 平台 | CLI 工具 | 最低版本 | 已测试版本 | 功能覆盖 |\n");
    md.push_str("|------|---------|---------|-----------|--------|\n");

    for p in &root.platforms {
        let tested = p.tested_versions.join(", ");
        let features: Vec<String> = p
            .features
            .iter()
            .map(|(k, &v)| {
                if v {
                    format!("{k} ✅")
                } else {
                    format!("{k} ❌")
                }
            })
            .collect();
        md.push_str(&format!(
            "| {} | `{}` | ≥ {} | {} | {} |\n",
            p.name,
            p.cli_binary,
            p.min_version,
            tested,
            features.join(" ")
        ));
    }

    fs::write("docs/compatibility-matrix.md", &md).expect("failed to write markdown");
    println!("Generated docs/compatibility-matrix.md");
}
