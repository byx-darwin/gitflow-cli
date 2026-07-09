//! E2E 测试核心库
//!
//! 提供共享的测试工具，包括 TTY 控制、测试配置和资源管理。

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

pub mod config;
pub mod fixture;
pub mod tty;

pub use config::TestConfig;
pub use fixture::{TestFixture, TestResource};
pub use tty::{CommandOutput, TtyMode, TtyRunner};
