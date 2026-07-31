//! 兼容性矩阵数据。
//!
//! 从 `docs/compatibility-matrix.json` 编译时嵌入，
//! 提供各平台 CLI 版本要求和功能覆盖信息。

use serde::Deserialize;

/// 编译时嵌入的兼容性矩阵 JSON。
const MATRIX_JSON: &str = include_str!("../../../docs/compatibility-matrix.json");

/// 兼容性矩阵根结构。
#[derive(Debug, Deserialize)]
struct MatrixRoot {
    /// 矩阵 schema 版本。
    #[allow(dead_code, reason = "Deserialized for validation, not yet used at runtime")]
    schema_version: u32,
    /// 最后更新日期。
    #[allow(dead_code, reason = "Deserialized for validation, not yet used at runtime")]
    updated_at: String,
    /// gitflow-cli 版本。
    #[allow(dead_code, reason = "Deserialized for validation, not yet used at runtime")]
    gitflow_cli_version: String,
    /// 平台列表。
    platforms: Vec<PlatformCompat>,
}

/// 单个平台的兼容性信息。
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformCompat {
    /// 平台显示名称（如 `"GitHub"`）。
    pub name: String,
    /// 平台标识符（如 `"github"`）。
    pub identifier: String,
    /// CLI 可执行文件名（如 `"gh"`）。
    pub cli_binary: String,
    /// 最低版本号（semver）。
    pub min_version: String,
    /// 已测试的版本列表。
    pub tested_versions: Vec<String>,
    /// 官方安装指引链接。
    pub install_url: String,
    /// 文档链接。
    pub doc_link: String,
}

/// 获取所有平台的兼容性信息。
///
/// # Panics
///
/// 当嵌入的 JSON 格式无效时 panic（编译时数据损坏，属于不可恢复错误）。
#[must_use]
#[allow(
    clippy::expect_used,
    reason = "Embedded JSON is compile-time data; parse failure means corrupted build artifacts"
)]
pub fn platform_compatibility() -> Vec<PlatformCompat> {
    let root: MatrixRoot = serde_json::from_str(MATRIX_JSON)
        .expect("embedded compatibility-matrix.json is invalid");
    root.platforms
}

/// 获取指定平台的兼容性信息。
#[must_use]
pub fn platform_requirement(identifier: &str) -> Option<PlatformCompat> {
    platform_compatibility()
        .into_iter()
        .find(|p| p.identifier == identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_load_all_three_platforms() {
        let platforms = platform_compatibility();
        assert_eq!(platforms.len(), 3);
    }

    #[test]
    fn test_should_return_github_requirement() {
        let gh = platform_requirement("github").expect("github should exist");
        assert_eq!(gh.cli_binary, "gh");
        assert_eq!(gh.min_version, "2.0.0");
        assert!(!gh.install_url.is_empty());
        assert!(!gh.doc_link.is_empty());
    }

    #[test]
    fn test_should_return_gitcode_min_version_0_6() {
        let gc = platform_requirement("gitcode").expect("gitcode should exist");
        assert_eq!(gc.min_version, "0.6.0");
    }

    #[test]
    fn test_should_return_none_for_unknown_platform() {
        assert!(platform_requirement("bitbucket").is_none());
    }
}
