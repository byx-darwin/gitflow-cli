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

use std::{fs, path::PathBuf};

/// 全渠道逐字一致的规范一句话定位。
const CANONICAL_POSITIONING: &str = "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / \
                                     GitCode 三大平台，配合 AI Agent \
                                     Skills，覆盖从需求到发布的完整工程循环。";

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
        assert!(
            !raw.contains("Your Name"),
            "{rel} still contains 'Your Name' placeholder"
        );
        assert!(
            !raw.contains("yourdomain"),
            "{rel} still contains 'yourdomain' placeholder"
        );
        assert!(
            !raw.contains("{{version}}"),
            "{rel} contains unrendered template variable"
        );

        let doc: toml::Value = raw
            .parse()
            .unwrap_or_else(|e| panic!("{rel} is not valid TOML: {e}"));

        if rel == "Cargo.toml" {
            let pkg = &doc["workspace"]["package"];
            let authors = pkg["authors"]
                .as_array()
                .expect("workspace authors must be an array");
            assert!(!authors.is_empty(), "workspace authors empty");
            let homepage = pkg["homepage"]
                .as_str()
                .expect("workspace homepage missing");
            assert!(
                homepage.starts_with("https://"),
                "workspace homepage must be https URL"
            );
            assert!(
                !homepage.contains("yourdomain"),
                "workspace homepage has placeholder"
            );
        } else {
            let pkg = &doc["package"];
            let desc = pkg["description"]
                .as_str()
                .unwrap_or_else(|| panic!("{rel}: description missing"));
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

#[test]
fn test_should_have_valid_geo_files() {
    let llms = read("website/public/llms.txt");
    assert!(llms.starts_with("# gf"), "llms.txt must start with '# gf'");
    assert!(
        llms.contains(CANONICAL_POSITIONING),
        "llms.txt must quote canonical positioning"
    );

    let full = read("website/public/llms-full.txt");
    assert!(
        full.len() > 500,
        "llms-full.txt should be a substantial full-text document"
    );
    for section in ["命令", "架构", "兼容性", "FAQ"] {
        assert!(
            full.contains(section),
            "llms-full.txt missing section {section}"
        );
    }

    let robots = read("website/public/robots.txt");
    assert!(
        robots.contains("Sitemap:"),
        "robots.txt must declare Sitemap"
    );

    let matrix: serde_json::Value =
        serde_json::from_str(&read("crates/core/resources/compatibility-matrix.json"))
            .expect("compatibility-matrix.json invalid");
    assert_eq!(matrix["schema_version"].as_i64(), Some(1));
    let platforms = matrix["platforms"]
        .as_array()
        .expect("platforms must be an array");
    assert_eq!(
        platforms.len(),
        3,
        "expected 3 platforms in compatibility matrix"
    );

    let md = read("docs/compatibility-matrix.md");
    for name in ["GitHub", "GitLab", "GitCode"] {
        assert!(md.contains(name), "compatibility-matrix.md missing {name}");
    }
}

#[test]
fn test_should_have_binstall_metadata_matching_release_layout() {
    let cli = read("apps/cli/Cargo.toml");
    let doc: toml::Value = cli.parse().expect("apps/cli/Cargo.toml must be valid TOML");
    let binstall = &doc["package"]["metadata"]["binstall"];

    // Asset filename is `gf-{target}.tgz` — the binary name, not the crate
    // name (`gitflow-cli`). Matches `.github/workflows/release.yml` `BINARY_NAME: gf`.
    let pkg_url = binstall["pkg-url"]
        .as_str()
        .expect("binstall pkg-url missing");
    assert_eq!(
        pkg_url, "{ repo }/releases/download/v{ version }/{ bin }-{ target }.{ archive-format }",
        "binstall pkg-url must reference the `{{ bin }}` asset name"
    );

    // Binary lives at the archive root (`gf` / `gf.exe`), no subdirectory.
    let bin_dir = binstall["bin-dir"]
        .as_str()
        .expect("binstall bin-dir missing");
    assert_eq!(
        bin_dir, "{ bin }{ binary-ext }",
        "binstall bin-dir must point at the root-level binary ({{ bin }}{{ binary-ext }})"
    );

    let pkg_fmt = binstall["pkg-fmt"]
        .as_str()
        .expect("binstall pkg-fmt missing");
    assert_eq!(pkg_fmt, "tgz", "binstall pkg-fmt must be tgz");
}

#[test]
fn test_should_have_demo_asset() {
    let svg = read("docs/assets/demo.svg");
    assert!(
        svg.contains("<svg"),
        "demo.svg must be a valid SVG document"
    );
    assert!(svg.contains("</svg>"), "demo.svg must be closed");
}

#[test]
fn test_should_keep_entity_consistency() {
    let readme = read("README.md");
    assert!(
        readme.contains(CANONICAL_POSITIONING),
        "README.md missing canonical positioning"
    );

    let llms = read("website/public/llms.txt");
    assert!(
        llms.contains(CANONICAL_POSITIONING),
        "llms.txt missing canonical positioning"
    );

    let jsonld = read("website/src/lib/jsonld.ts");
    assert!(
        jsonld.contains(CANONICAL_POSITIONING),
        "jsonld.ts missing canonical positioning"
    );
}

#[test]
fn test_should_have_valid_jsonld() {
    let jsonld = read("website/src/lib/jsonld.ts");

    // Check that jsonld.ts has the canonical positioning constant
    assert!(
        jsonld.contains(CANONICAL_POSITIONING),
        "jsonld.ts missing canonical positioning"
    );

    // Check that it exports the generator functions
    assert!(
        jsonld.contains("export function generateSoftwareAppJsonLd"),
        "jsonld.ts missing generateSoftwareAppJsonLd"
    );
    assert!(
        jsonld.contains("export function generateFAQPageJsonLd"),
        "jsonld.ts missing generateFAQPageJsonLd"
    );
    assert!(
        jsonld.contains("export function generateHowToJsonLd"),
        "jsonld.ts missing generateHowToJsonLd"
    );

    // Check that it has the correct JSON-LD types
    assert!(
        jsonld.contains("SoftwareApplication"),
        "jsonld.ts missing SoftwareApplication type"
    );
    assert!(jsonld.contains("FAQPage"), "jsonld.ts missing FAQPage type");
    assert!(jsonld.contains("HowTo"), "jsonld.ts missing HowTo type");

    // Check that Base.astro imports and uses the generator
    let base = read("website/src/layouts/Base.astro");
    assert!(
        base.contains("generateSoftwareAppJsonLd"),
        "Base.astro not importing generateSoftwareAppJsonLd"
    );
    assert!(
        base.contains("generateFAQPageJsonLd"),
        "Base.astro not importing generateFAQPageJsonLd"
    );
}
