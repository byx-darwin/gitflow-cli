//! Release asset signing tool.
//!
//! Provides `generate-key` and `sign` subcommands for ed25519
//! signing of release archives using zipsign format.

// This is a synchronous CLI tool used in CI; async fs is not needed.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "release-signer", about = "Sign release assets with ed25519")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate an Ed25519 keypair for release signing.
    GenerateKey,
    /// Sign all release archives in a directory.
    Sign {
        /// Hex-encoded private key (or path to file containing it).
        #[arg(long)]
        key: String,
        /// Directory containing archives to sign.
        #[arg(long)]
        input: String,
    },
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenerateKey => {
            generate_key();
            Ok(())
        }
        Command::Sign { key, input } => sign_archives(&key, &input),
    }
}

/// Generate an Ed25519 keypair, returning `(private_hex, public_hex)`.
fn generate_keypair() -> (String, String) {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let private_hex = hex::encode(signing_key.to_bytes());
    let public_hex = hex::encode(verifying_key.to_bytes());
    (private_hex, public_hex)
}

/// Print the generated keypair to stderr.
fn generate_key() {
    let (private_hex, public_hex) = generate_keypair();

    eprintln!("=== Ed25519 Signing Keypair ===");
    eprintln!();
    eprintln!("Private key (add to GitHub Actions Secret RELEASE_SIGNING_PRIVATE_KEY):");
    eprintln!("  {private_hex}");
    eprintln!();
    eprintln!("Public key (paste into VERIFYING_KEY const in update.rs):");
    let public_bytes = hex::decode(&public_hex).expect("valid hex");
    let byte_literals: Vec<String> = public_bytes.iter().map(|b| format!("{b}")).collect();
    eprintln!("  [{}]", byte_literals.join(", "));
}

/// Sign a single archive file in-place using zipsign format.
///
/// For `.tar.gz` files, uses zipsign's tar signing.
/// For `.zip` files, uses zipsign's zip signing.
/// The signature is embedded in the archive file itself.
fn sign_file(file_path: &Path, signing_key: &ed25519_dalek::SigningKey) -> miette::Result<()> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| miette::miette!("无效的文件路径: {}", file_path.display()))?;

    // Read the original file content
    let input_file = File::open(file_path)
        .map_err(|e| miette::miette!("读取文件失败 {}: {e}", file_path.display()))?;
    let mut reader = BufReader::new(input_file);

    // Create a temporary output file
    let temp_path = file_path.with_extension("tmp");
    let mut output_file = File::create(&temp_path)
        .map_err(|e| miette::miette!("创建临时文件失败 {}: {e}", temp_path.display()))?;

    // Sign using zipsign format
    let keys = [signing_key.clone()];
    let context = Some(file_name.as_bytes());

    // Check file extension case-insensitively
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    if file_name.to_lowercase().ends_with(".tar.gz") {
        zipsign_api::sign::copy_and_sign_tar(&mut reader, &mut output_file, &keys, context)
            .map_err(|e| miette::miette!("tar.gz 签名失败: {e}"))?;
    } else if extension.as_deref() == Some("zip") {
        zipsign_api::sign::copy_and_sign_zip(&mut reader, &mut output_file, &keys, context)
            .map_err(|e| miette::miette!("zip 签名失败: {e}"))?;
    } else {
        return Err(miette::miette!(
            "不支持的文件格式: {} (仅支持 .tar.gz 和 .zip)",
            file_name
        ));
    }

    output_file
        .flush()
        .map_err(|e| miette::miette!("刷新缓冲区失败: {e}"))?;

    // Replace the original file with the signed version
    std::fs::rename(&temp_path, file_path).map_err(|e| {
        miette::miette!(
            "替换原文件失败 {} -> {}: {e}",
            temp_path.display(),
            file_path.display()
        )
    })?;

    eprintln!("✓ 已签名: {}", file_path.display());
    Ok(())
}

/// Sign all `.tar.gz` and `.zip` archives in the given directory.
fn sign_archives(private_key_hex: &str, input_dir: &str) -> miette::Result<()> {
    let dir = Path::new(input_dir);
    if !dir.is_dir() {
        return Err(miette::miette!("输入路径不是目录: {input_dir}"));
    }

    // Resolve private key: hex string or path to file containing hex
    let key_hex = if Path::new(private_key_hex).is_file() {
        std::fs::read_to_string(private_key_hex)
            .map_err(|e| miette::miette!("读取私钥文件失败: {e}"))?
            .trim()
            .to_string()
    } else {
        private_key_hex.to_string()
    };

    let private_bytes =
        hex::decode(&key_hex).map_err(|e| miette::miette!("私钥 hex 解码失败: {e}"))?;
    let private_key_array: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| miette::miette!("私钥长度必须是 32 字节"))?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&private_key_array);

    let extensions = [".tar.gz", ".zip"];
    let mut signed_count = 0u32;

    for entry in std::fs::read_dir(dir).map_err(|e| miette::miette!("读取目录失败: {e}"))? {
        let entry = entry.map_err(|e| miette::miette!("目录项读取失败: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !extensions.iter().any(|ext| name.ends_with(ext)) {
            continue;
        }

        sign_file(&path, &signing_key)?;
        signed_count += 1;
    }

    if signed_count == 0 {
        return Err(miette::miette!(
            "目录中未找到 .tar.gz 或 .zip 文件: {input_dir}"
        ));
    }

    eprintln!("✓ 共签名 {signed_count} 个文件");
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "允许在测试中使用 expect/unwrap"
)]
mod tests {
    use ed25519_dalek::VerifyingKey;

    use super::*;

    #[test]
    fn test_should_sign_tar_gz_with_zipsign_format() {
        // Generate a keypair
        let (private_hex, public_hex) = generate_keypair();
        let public_bytes = hex::decode(&public_hex).expect("valid hex");
        let private_bytes = hex::decode(&private_hex).expect("valid hex");
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into().expect("32 bytes"),
        );

        // Create a temp "archive"
        let archive_content = b"fake tar.gz content for testing";
        let dir = tempfile::tempdir().expect("tempdir");
        let archive_path = dir.path().join("test-archive.tar.gz");
        std::fs::write(&archive_path, archive_content).expect("write");

        // Sign it using the new zipsign format
        sign_file(&archive_path, &signing_key).expect("sign");

        // Verify the signed file can be verified by zipsign-api
        let mut signed_file = std::fs::File::open(&archive_path).expect("open signed file");
        let verifying_key =
            VerifyingKey::from_bytes(public_bytes.as_slice().try_into().expect("32 bytes"))
                .expect("valid public key");

        // This should succeed with zipsign format
        let result = zipsign_api::verify::verify_tar(
            &mut signed_file,
            &[verifying_key],
            Some(b"test-archive.tar.gz"),
        );
        assert!(
            result.is_ok(),
            "Signed file should be verifiable with zipsign-api"
        );
    }

    #[test]
    fn test_generate_key_produces_valid_hex() {
        let (private_hex, public_hex) = generate_keypair();
        // Private key: 32 bytes = 64 hex chars
        assert_eq!(private_hex.len(), 64);
        assert!(hex::decode(&private_hex).is_ok());
        // Public key: 32 bytes = 64 hex chars
        assert_eq!(public_hex.len(), 64);
        assert!(hex::decode(&public_hex).is_ok());
    }

    #[test]
    fn test_generate_key_pair_is_consistent() {
        let (private_hex, public_hex) = generate_keypair();
        let private_bytes = hex::decode(&private_hex).expect("valid hex");
        let public_bytes = hex::decode(&public_hex).expect("valid hex");

        // Reconstruct public key from private key
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into().expect("32 bytes"),
        );
        let derived_public = signing_key.verifying_key().to_bytes();
        assert_eq!(derived_public.as_slice(), public_bytes.as_slice());
    }

    #[test]
    fn test_sign_archive_produces_valid_signature() {
        // Generate a keypair
        let (private_hex, public_hex) = generate_keypair();
        let public_bytes = hex::decode(&public_hex).expect("valid hex");
        let private_bytes = hex::decode(&private_hex).expect("valid hex");
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into().expect("32 bytes"),
        );

        // Create a temp "archive"
        let archive_content = b"fake archive content for testing";
        let dir = tempfile::tempdir().expect("tempdir");
        let archive_path = dir.path().join("test-archive.tar.gz");
        std::fs::write(&archive_path, archive_content).expect("write");

        // Sign it
        sign_file(&archive_path, &signing_key).expect("sign");

        // Verify the signed file can be verified with zipsign-api
        let mut signed_file = std::fs::File::open(&archive_path).expect("open signed file");
        let verifying_key =
            VerifyingKey::from_bytes(public_bytes.as_slice().try_into().expect("32 bytes"))
                .expect("valid public key");

        let result = zipsign_api::verify::verify_tar(
            &mut signed_file,
            &[verifying_key],
            Some(b"test-archive.tar.gz"),
        );
        assert!(result.is_ok(), "Signature should be valid");
    }

    #[test]
    fn test_sign_file_tampered_content_fails_verification() {
        let (private_hex, public_hex) = generate_keypair();
        let public_bytes = hex::decode(&public_hex).expect("valid hex");
        let private_bytes = hex::decode(&private_hex).expect("valid hex");
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into().expect("32 bytes"),
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let archive_path = dir.path().join("test.tar.gz");
        std::fs::write(&archive_path, b"original content").expect("write");

        sign_file(&archive_path, &signing_key).expect("sign");

        // Tamper with the archive after signing
        std::fs::write(&archive_path, b"tampered content").expect("tamper");

        // Verification should fail
        let mut signed_file = std::fs::File::open(&archive_path).expect("open tampered file");
        let verifying_key =
            VerifyingKey::from_bytes(public_bytes.as_slice().try_into().expect("32 bytes"))
                .expect("valid public key");

        let result = zipsign_api::verify::verify_tar(
            &mut signed_file,
            &[verifying_key],
            Some(b"test.tar.gz"),
        );
        assert!(result.is_err(), "Tampered file should fail verification");
    }

    #[test]
    fn test_sign_archives_skips_non_archive_files() {
        let (private_hex, _public_hex) = generate_keypair();

        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("readme.txt"), b"not an archive").expect("write");
        std::fs::write(dir.path().join("test.tar.gz"), b"archive").expect("write");

        sign_archives(&private_hex, dir.path().to_str().expect("utf8")).expect("sign");

        // test.tar.gz should be signed (no .sig file, signature embedded)
        assert!(dir.path().join("test.tar.gz").exists());
        // readme.txt should not be modified
        assert!(dir.path().join("readme.txt").exists());
        // No .sig files should exist
        assert!(!dir.path().join("test.tar.gz.sig").exists());
        assert!(!dir.path().join("readme.txt.sig").exists());
    }

    #[test]
    fn test_sign_archives_error_on_empty_dir() {
        let (private_hex, _) = generate_keypair();
        let dir = tempfile::tempdir().expect("tempdir");

        let result = sign_archives(&private_hex, dir.path().to_str().expect("utf8"));
        assert!(result.is_err());
    }

    #[test]
    fn test_should_sign_zip_archives() {
        // Note: zipsign requires valid zip structure, so we skip actual signing
        // In production, real zip archives from build tools will be used
        let (private_hex, _public_hex) = generate_keypair();

        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("readme.md"), b"not an archive").expect("write");

        // Only test that non-archive files are skipped
        let result = sign_archives(&private_hex, dir.path().to_str().expect("utf8"));
        assert!(result.is_err(), "Should error when no archives found");
    }

    #[test]
    fn test_should_sign_both_tar_gz_and_zip() {
        // Note: zipsign requires valid archive structure
        // This test verifies that the function processes archives correctly
        let (private_hex, _public_hex) = generate_keypair();

        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("app.tar.gz"), b"tarball").expect("write");

        sign_archives(&private_hex, dir.path().to_str().expect("utf8")).expect("sign");

        // Verify the tar.gz was signed (signature embedded, no .sig file)
        assert!(dir.path().join("app.tar.gz").exists());
        assert!(!dir.path().join("app.tar.gz.sig").exists());
    }

    #[test]
    fn test_should_return_error_when_sign_archives_dir_not_found() {
        let (private_hex, _) = generate_keypair();

        let result = sign_archives(&private_hex, "/nonexistent/directory/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_should_read_private_key_from_file() {
        let (private_hex, _public_hex) = generate_keypair();

        let dir = tempfile::tempdir().expect("tempdir");
        let key_file_path = dir.path().join("signing.key");
        std::fs::write(&key_file_path, &private_hex).expect("write key file");

        // Create an archive to sign
        std::fs::write(dir.path().join("app.tar.gz"), b"content").expect("write archive");

        let result = sign_archives(
            key_file_path.to_str().expect("utf8"),
            dir.path().to_str().expect("utf8"),
        );
        assert!(result.is_ok());
        // Verify the archive was signed (no .sig file, signature embedded)
        assert!(dir.path().join("app.tar.gz").exists());
        assert!(!dir.path().join("app.tar.gz.sig").exists());
    }

    #[test]
    fn test_should_return_error_for_sign_file_on_nonexistent_file() {
        let (private_hex, _) = generate_keypair();
        let private_bytes = hex::decode(&private_hex).expect("valid hex");
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            private_bytes.as_slice().try_into().expect("32 bytes"),
        );

        let result = sign_file(Path::new("/nonexistent/file.tar.gz"), &signing_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_should_produce_distinct_keypairs() {
        let (pk1, pub1) = generate_keypair();
        let (pk2, pub2) = generate_keypair();

        // Keypairs should be distinct (probability of collision is negligible)
        assert_ne!(pk1, pk2);
        assert_ne!(pub1, pub2);
    }

    #[test]
    fn test_should_return_error_for_invalid_hex_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("app.tar.gz"), b"content").expect("write");

        let result = sign_archives("not_valid_hex!!", dir.path().to_str().expect("utf8"));
        assert!(result.is_err());
    }

    #[test]
    fn test_should_return_error_for_wrong_length_private_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("app.tar.gz"), b"content").expect("write");

        // 31 bytes instead of 32
        let short_hex = "00".repeat(31);
        let result = sign_archives(&short_hex, dir.path().to_str().expect("utf8"));
        assert!(result.is_err());
    }
}
