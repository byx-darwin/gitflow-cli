//! TTY 控制模块
//!
//! 提供交互模式和非交互模式的命令执行能力。

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::ExitStatus,
};

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

/// Resolve the `gf` binary that e2e tests exercise.
///
/// Returns the binary built from this workspace — never whatever `gf` happens
/// to be on `PATH`. Resolving through `PATH` made e2e results depend on when
/// the developer last ran `cargo install`, so a branch could be tested against
/// a binary that predates its own changes (Issue #358).
///
/// # Errors
///
/// Returns [`TtyError::Io`] when the current executable cannot be located, or
/// when no `gf` sits beside it. The latter means the harness was not launched
/// through `cargo test` / `cargo nextest run`, which build the binary first;
/// falling back to a `PATH` lookup is deliberately not done.
pub fn gf_binary() -> Result<PathBuf, TtyError> {
    let test_exe = std::env::current_exe()?;

    gf_binary_from_test_exe(&test_exe).ok_or_else(|| {
        TtyError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "e2e harness: no `gf{suffix}` next to {parent}. e2e tests run the binary built \
                 from this workspace, so they must be launched through `cargo test` / `cargo \
                 nextest run`, which build it first. Falling back to a PATH lookup is \
                 deliberately not done — see Issue #358.",
                suffix = std::env::consts::EXE_SUFFIX,
                parent = test_exe
                    .parent()
                    .and_then(Path::parent)
                    .unwrap_or(&test_exe)
                    .display(),
            ),
        ))
    })
}

/// Derive the built `gf` path from the location of a test executable.
///
/// Cargo places integration-test executables in `<target>/<profile>/deps/` and
/// binaries in `<target>/<profile>/`, so the binary sits one level above the
/// test executable's own directory. Returns `None` when nothing usable is
/// there — callers must not silently fall back to `PATH`.
fn gf_binary_from_test_exe(test_exe: &Path) -> Option<PathBuf> {
    let profile_dir = test_exe.parent()?.parent()?;
    let candidate = profile_dir.join(format!("gf{}", std::env::consts::EXE_SUFFIX));
    candidate.is_file().then_some(candidate)
}

/// TTY 测试运行器
#[derive(Debug)]
pub struct TtyRunner {
    #[allow(dead_code, reason = "Mode reserved for future TTY-specific logic")]
    mode: TtyMode,
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
    env_removals: Vec<String>,
}

impl TtyRunner {
    /// 创建新的 TTY 运行器
    #[must_use]
    pub fn new(mode: TtyMode) -> Self {
        Self {
            mode,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env_vars: HashMap::new(),
            env_removals: Vec::new(),
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

    /// 从子进程环境中移除变量(如清除继承的 `GH_TOKEN` 以测试未认证路径)
    pub fn env_remove<K>(&mut self, key: K) -> &mut Self
    where
        K: Into<String>,
    {
        self.env_removals.push(key.into());
        self
    }

    /// 覆盖执行时的工作目录(默认取进程自身 cwd)。
    ///
    /// 用于让测试在指定目录(例如 [`crate::scratch_repo_dir`] 创建的临时仓库)中
    /// 执行 `gf`,绕过 `gf` 仅从 `git remote get-url origin` 解析仓库路径、
    /// 部分子命令无 `--repo` 覆盖的限制。
    pub fn dir(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.working_dir = path.into();
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

        let mut cmd = Command::new(gf_binary()?);
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

        for key in &self.env_removals {
            cmd.env_remove(key);
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
mod binary_resolution_tests {
    use std::fs;

    use super::{gf_binary, gf_binary_from_test_exe};

    fn bin_name() -> String {
        format!("gf{}", std::env::consts::EXE_SUFFIX)
    }

    #[test]
    fn test_should_resolve_gf_beside_the_profile_directory() {
        let root = tempfile::tempdir().expect("tempdir");
        let deps = root.path().join("debug").join("deps");
        fs::create_dir_all(&deps).expect("create deps");
        let expected = root.path().join("debug").join(bin_name());
        fs::write(&expected, b"").expect("write gf");

        let test_exe = deps.join("e2e_gitcode-0123456789abcdef");
        fs::write(&test_exe, b"").expect("write test exe");

        assert_eq!(
            gf_binary_from_test_exe(&test_exe),
            Some(expected),
            "the binary under test lives one level above the deps directory"
        );
    }

    #[test]
    fn test_should_not_resolve_when_built_binary_is_absent() {
        let root = tempfile::tempdir().expect("tempdir");
        let deps = root.path().join("debug").join("deps");
        fs::create_dir_all(&deps).expect("create deps");
        let test_exe = deps.join("e2e_gitcode-0123456789abcdef");
        fs::write(&test_exe, b"").expect("write test exe");

        assert_eq!(
            gf_binary_from_test_exe(&test_exe),
            None,
            "falling back to a PATH lookup is what Issue #358 is about — never do it silently"
        );
    }

    #[test]
    fn test_should_not_resolve_when_a_directory_shadows_the_binary_name() {
        let root = tempfile::tempdir().expect("tempdir");
        let deps = root.path().join("debug").join("deps");
        fs::create_dir_all(&deps).expect("create deps");
        fs::create_dir_all(root.path().join("debug").join(bin_name())).expect("create dir");
        let test_exe = deps.join("e2e_gitcode-0123456789abcdef");
        fs::write(&test_exe, b"").expect("write test exe");

        assert_eq!(
            gf_binary_from_test_exe(&test_exe),
            None,
            "a directory named `gf` is not the binary under test"
        );
    }

    #[test]
    fn test_should_not_resolve_when_path_has_no_grandparent() {
        assert_eq!(gf_binary_from_test_exe(std::path::Path::new("gf")), None);
    }

    #[test]
    fn test_should_resolve_to_an_existing_file_in_this_test_run() {
        let resolved = gf_binary().expect("workspace build must be resolvable during cargo test");
        assert!(
            resolved.is_file(),
            "gf_binary() must point at the workspace build, got {}",
            resolved.display()
        );
        assert!(
            resolved.is_absolute(),
            "a bare name would be resolved through PATH by the OS — got {}",
            resolved.display()
        );
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

    #[test]
    fn test_should_record_env_removals_in_order() {
        let mut runner = TtyRunner::new(TtyMode::NonInteractive);
        runner
            .env_remove("GH_TOKEN")
            .env_remove("GITHUB_TOKEN")
            .env("E2E_PROBE", "1");
        assert_eq!(
            runner.env_removals,
            vec!["GH_TOKEN".to_string(), "GITHUB_TOKEN".to_string()]
        );
        assert_eq!(runner.env_vars.get("E2E_PROBE"), Some(&"1".to_string()));
    }

    #[test]
    fn test_should_override_working_dir_when_dir_is_set() {
        let mut runner = TtyRunner::new(TtyMode::NonInteractive);
        let custom = PathBuf::from("/tmp/e2e-core-dir-test");
        runner.dir(custom.clone());
        assert_eq!(runner.working_dir, custom);
    }
}
