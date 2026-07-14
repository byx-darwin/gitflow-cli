class GitflowCli < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.9.0/gitflow-cli-aarch64-apple-darwin.tar.gz"
      sha256 "9efe4aed61efb3f353e9838c9b05fc903066526b1c12c1a87da99c6ba1ee062d"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.9.0/gitflow-cli-x86_64-apple-darwin.tar.gz"
      sha256 "79b9d9104a42e7c107603b3e03069c1710ed8e32d35239d9e059ee8918b0db26"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.9.0/gitflow-cli-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "fd9d6c8a340b3c61eb8bc417192ab518db429d5a0bb3e43c24ba435cd5bbc7f9"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v0.9.0/gitflow-cli-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fcbc9a7a2b387f244d6dbc041f395fb0bc9dc895dcb672ac7aee5dd3ae400eea"
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
