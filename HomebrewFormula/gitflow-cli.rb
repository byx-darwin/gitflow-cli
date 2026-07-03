class GitflowCli < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.1.0/gitflow-cli-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.1.0/gitflow-cli-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.1.0/gitflow-cli-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.1.0/gitflow-cli-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  # gh CLI 是运行时依赖（GitHub 平台需要）
  # glab 和 gc 是可选的（GitLab/GitCode 平台需要）
  depends_on "gh"

  def install
    bin.install "gitflow-cli"

    # 安装 Shell 补全
    generate_completions_from_executable(bin/"gitflow-cli", "completions")
  end

  test do
    system "#{bin}/gitflow-cli", "--version"
    system "#{bin}/gitflow-cli", "--help"
  end
end
