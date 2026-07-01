use gitflow-cli_core::{Config, CoreError, Result};

#[test]
fn integration_config_roundtrip() -> Result<()> {
    let config = Config::new("integration-test")?.with_description("test config");
    assert_eq!(config.name(), "integration-test");
    assert_eq!(config.description(), Some("test config"));
    Ok(())
}

#[test]
fn integration_empty_config_name_should_fail() {
    let err = Config::new("");
    assert!(matches!(err, Err(CoreError::App(_))));
}
