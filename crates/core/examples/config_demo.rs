//! Example: using the `Config` type from the core library.

use gitflow-cli_core::{Config, Result};

fn main() -> Result<()> {
    let config = Config::new("my-app")?.with_description("A demo configuration");

    println!("config: {config:?}");
    println!("  name: {}", config.name());
    println!(
        "  description: {}",
        config.description().unwrap_or("(none)")
    );

    Ok(())
}
