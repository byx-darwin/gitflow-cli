//! 列表类命令的共享参数校验。

use crate::errors::UserInputError;

/// `--limit` 允许的最大值。
///
/// 信任边界处的显式上界：没有它，`--limit 4000000000` 这类输入会在向下游
/// 拼接 `--limit <cap + 1>` 时逼近甚至越过 `u32`/`i32` 边界，产生令人困惑的
/// 底层报错而非清晰的用户输入校验错误。
pub const MAX_LIMIT: u32 = 100_000;

/// 校验 `--limit`：必须大于 0 且不超过 [`MAX_LIMIT`]。
///
/// `0` 会让分页器返回空集并报告截断，对用户毫无意义，因此在信任边界处直接拒绝，
/// 而不是让它穿透到适配器。`None` 原样透传，由适配器套用
/// [`gitflow_core::DEFAULT_LIST_LIMIT`]。
///
/// # Errors
///
/// `limit` 为 `Some(0)` 或大于 [`MAX_LIMIT`] 时返回 [`UserInputError`]。
pub fn validate_limit(limit: Option<u32>) -> Result<Option<u32>, UserInputError> {
    if limit == Some(0) {
        return Err(UserInputError::new(
            "Invalid --limit '0'. Expected a positive integer.".to_string(),
        ));
    }
    if let Some(n) = limit
        && n > MAX_LIMIT
    {
        return Err(UserInputError::new(format!(
            "Invalid --limit '{n}'. Expected a value no greater than {MAX_LIMIT}."
        )));
    }
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_accept_absent_limit() {
        assert!(matches!(validate_limit(None), Ok(None)));
    }

    #[test]
    fn test_should_accept_positive_limit() {
        assert!(matches!(validate_limit(Some(1)), Ok(Some(1))));
        assert!(matches!(validate_limit(Some(5000)), Ok(Some(5000))));
    }

    #[test]
    fn test_should_reject_zero_limit() {
        let err = validate_limit(Some(0)).expect_err("0 必须被拒绝");
        assert!(err.to_string().contains("--limit"));
    }

    #[test]
    fn test_should_reject_limit_above_maximum() {
        assert!(matches!(
            validate_limit(Some(MAX_LIMIT)),
            Ok(Some(n)) if n == MAX_LIMIT
        ));
        let err = validate_limit(Some(MAX_LIMIT + 1)).expect_err("超过上限必须被拒绝");
        assert!(err.to_string().contains("--limit"));
        let err = validate_limit(Some(4_000_000_000)).expect_err("远超上限必须被拒绝");
        assert!(err.to_string().contains("--limit"));
    }
}
