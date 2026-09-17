//! 列表类命令的共享参数校验。

use crate::errors::UserInputError;

/// 校验 `--limit`：必须大于 0。
///
/// `0` 会让分页器返回空集并报告截断，对用户毫无意义，因此在信任边界处直接拒绝，
/// 而不是让它穿透到适配器。`None` 原样透传，由适配器套用
/// [`gitflow_core::DEFAULT_LIST_LIMIT`]。
///
/// # Errors
///
/// `limit` 为 `Some(0)` 时返回 [`UserInputError`]。
pub fn validate_limit(limit: Option<u32>) -> Result<Option<u32>, UserInputError> {
    if limit == Some(0) {
        return Err(UserInputError::new(
            "Invalid --limit '0'. Expected a positive integer.".to_string(),
        ));
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
}
