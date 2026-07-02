//! `gitflow auth` 子命令实现。
//!
//! 提供登录、登出、状态查询、Token 获取等认证相关功能，
//! 支持通过 clap 解析参数后调用对应平台的 [`AuthProvider`] 实现。
//! Phase 1 仅支持 JSON 输出。

use clap::Subcommand;
use gitflow_cli_core::{CliOutput, auth::AuthProvider};
use gitflow_cli_gitcode::GitCodeAuthProvider;
use gitflow_cli_github::GitHubAuthProvider;
use gitflow_cli_gitlab::GitLabAuthProvider;

use crate::OutputFormat;

/// Auth 子命令集合。
///
/// 支持 `login`、`logout`、`status`、`token` 操作。
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// 执行登录流程。
    Login,

    /// 执行登出流程，清除本地凭据。
    Logout,

    /// 查询当前认证状态。
    Status,

    /// 获取当前有效的访问 Token。
    Token,
}

/// 处理 `gitflow auth` 子命令。
///
/// 根据 `platform` 选择对应的 Auth 提供者，然后执行具体命令并输出结果。
/// 支持 `github`、`gitlab`、`gitcode` 三个平台，Phase 1 仅支持 JSON 输出格式。
///
/// # Errors
///
/// 返回错误当：
/// - 平台暂不支持。
/// - 底层 provider 调用失败。
/// - JSON 序列化失败。
#[allow(
    clippy::too_many_lines,
    reason = "Command dispatch: each match arm maps to one operation"
)]
pub async fn handle(
    command: AuthCommand,
    platform: &str,
    repo: &str,
    output_format: OutputFormat,
) -> miette::Result<()> {
    let provider: Box<dyn AuthProvider> = match platform {
        "github" => Box::new(GitHubAuthProvider::new()),
        "gitlab" => Box::new(GitLabAuthProvider::new()),
        "gitcode" => Box::new(GitCodeAuthProvider::new()),
        other => {
            return Err(miette::miette!(
                "Platform '{other}' not yet supported for auth commands"
            ));
        }
    };

    // AuthProvider 方法不需要 repo，但 handle 签名一致，保持接口统一
    let _ = repo;

    match command {
        AuthCommand::Login => {
            provider
                .login()
                .await
                .map_err(|e| miette::miette!("Failed to login: {e}"))?;
            let result = serde_json::json!({
                "action": "login",
                "success": true,
            });
            let output = CliOutput::success(result, platform, "auth login");
            print_output(&output, &output_format)?;
        }
        AuthCommand::Logout => {
            provider
                .logout()
                .await
                .map_err(|e| miette::miette!("Failed to logout: {e}"))?;
            let result = serde_json::json!({
                "action": "logout",
                "success": true,
            });
            let output = CliOutput::success(result, platform, "auth logout");
            print_output(&output, &output_format)?;
        }
        AuthCommand::Status => {
            let status = provider
                .status()
                .await
                .map_err(|e| miette::miette!("Failed to get auth status: {e}"))?;
            let output = CliOutput::success(status, platform, "auth status");
            print_output(&output, &output_format)?;
        }
        AuthCommand::Token => {
            let token = provider
                .token()
                .await
                .map_err(|e| miette::miette!("Failed to get token: {e}"))?;
            let output = CliOutput::success(
                serde_json::json!({ "token": token }),
                platform,
                "auth token",
            );
            print_output(&output, &output_format)?;
        }
    }

    Ok(())
}

/// 根据输出格式打印结果。
///
/// Phase 1 仅支持 JSON（pretty-printed）。Text 格式暂未实现，返回错误。
///
/// # Errors
///
/// 返回错误当：
/// - JSON 序列化失败。
/// - 输出格式为 `Text`（Phase 1 不支持）。
fn print_output<T: serde::Serialize>(value: &T, format: &OutputFormat) -> miette::Result<()> {
    crate::commands::output::print_output(value, format)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    reason = "Test code: panic is acceptable for assertion failures"
)]
mod tests {
    use super::*;

    #[test]
    fn test_should_print_json_output() {
        let value = serde_json::json!({"action": "login", "success": true});
        let result = print_output(&value, &OutputFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_accept_text_output() {
        let value = serde_json::json!({"number": 1});
        let result = print_output(&value, &OutputFormat::Text);
        assert!(result.is_ok());
    }

    // --- AuthCommand 解析测试 ---

    #[test]
    fn test_should_parse_auth_login() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "auth", "login"]).expect("parse");
        match cli.command {
            crate::Commands::Auth(AuthCommand::Login) => {}
            _ => panic!("Expected AuthCommand::Login"),
        }
    }

    #[test]
    fn test_should_parse_auth_logout() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "auth", "logout"]).expect("parse");
        match cli.command {
            crate::Commands::Auth(AuthCommand::Logout) => {}
            _ => panic!("Expected AuthCommand::Logout"),
        }
    }

    #[test]
    fn test_should_parse_auth_status() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "auth", "status"]).expect("parse");
        match cli.command {
            crate::Commands::Auth(AuthCommand::Status) => {}
            _ => panic!("Expected AuthCommand::Status"),
        }
    }

    #[test]
    fn test_should_parse_auth_token() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["gitflow", "auth", "token"]).expect("parse");
        match cli.command {
            crate::Commands::Auth(AuthCommand::Token) => {}
            _ => panic!("Expected AuthCommand::Token"),
        }
    }
}
