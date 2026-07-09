//! TTY 控制模块
//!
//! 提供交互模式和非交互模式的命令执行能力。

use std::{collections::HashMap, path::PathBuf, process::ExitStatus};

use thiserror::Error;

/// TTY 模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyMode {
    /// 有 TTY（交互模式）
    Interactive,
    /// 无 TTY（非交互模式，stdin 重定向）
    NonInteractive,
}

/// TTY 相关错误
#[derive(Debug, Error)]
pub enum TtyError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 命令输出
#[derive(Debug)]
pub struct CommandOutput {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出状态
    pub status: ExitStatus,
}

/// TTY 测试运行器
#[derive(Debug)]
pub struct TtyRunner {
    #[allow(dead_code, reason = "Mode reserved for future TTY-specific logic")]
    mode: TtyMode,
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl TtyRunner {
    /// 创建新的 TTY 运行器
    #[must_use]
    pub fn new(mode: TtyMode) -> Self {
        Self {
            mode,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env_vars: HashMap::new(),
        }
    }

    /// 设置环境变量
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// 执行命令并返回输出
    ///
    /// # Errors
    ///
    /// Returns `TtyError::Io` if the command cannot be executed or if reading
    /// the output fails.
    pub async fn run(&self, args: &[&str]) -> Result<CommandOutput, TtyError> {
        use tokio::process::Command;

        let mut cmd = Command::new("gitflow-cli");
        cmd.args(args);
        cmd.current_dir(&self.working_dir);

        // 两种模式都使用 stdin 重定向，区别在于是否分配 TTY
        // 简化实现：统一使用 stdin: null
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        for (k, v) in &self.env_vars {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_mode_equality() {
        assert_eq!(TtyMode::Interactive, TtyMode::Interactive);
        assert_eq!(TtyMode::NonInteractive, TtyMode::NonInteractive);
        assert_ne!(TtyMode::Interactive, TtyMode::NonInteractive);
    }
}
