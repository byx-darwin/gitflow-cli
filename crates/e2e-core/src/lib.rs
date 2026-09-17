//! E2E 测试核心库
//!
//! 提供共享的测试工具，包括 TTY 控制、测试配置和资源管理。

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::disallowed_methods,
        reason = "Tests unwrap fixture data built moments earlier, and build those fixtures with \
                  synchronous std::fs — the async replacements would need a runtime these plain \
                  #[test] functions do not have"
    )
)]

pub mod config;
pub mod fixture;
pub mod scratch;
pub mod tty;

pub use config::{TestConfig, TestMode};
pub use fixture::{TestFixture, TestResource};
pub use scratch::scratch_repo_dir;
pub use tty::{CommandOutput, TtyError, TtyMode, TtyRunner};
