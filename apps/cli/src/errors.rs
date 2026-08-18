//! CLI 内部错误分类。
//!
//! 区分「用户输入/参数错误」与「真实运行缺陷」，供主动上报 bug 功能过滤误报。
//! 用户传错参数（如非法 `--state` 值）不属于 CLI 缺陷，不应被自动上报为 Issue。

use miette::Diagnostic;
use thiserror::Error;

/// 用户输入/参数校验错误。
///
/// 携带 miette code `gf::user_input`，供 `main.rs` 顶层分类识别。
/// 此类错误不会被主动上报（避免把用户传参错误当 bug 上报）。
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code = "gf::user_input")]
pub(crate) struct UserInputError {
    message: String,
}

impl UserInputError {
    /// 构造用户输入错误。
    #[must_use]
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
