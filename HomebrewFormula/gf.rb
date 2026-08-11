class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.3.0/gf-1.3.0-aarch64-apple-darwin.tar.gz"
      sha256 "63d0fe43a2a74f42e950ab466501e969cd8a03c07bc4eeb069380ec78d6a98aa"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.3.0/gf-1.3.0-x86_64-apple-darwin.tar.gz"
      sha256 "f476d7de92602f90f5e44017723d775133b3361a55fb16638430a68751b31325"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.3.0/gf-1.3.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "cf544d748ef5b5cd0bb84c47472694c447e2708c9819d861a587abd1c41b7226"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.3.0/gf-1.3.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d0188385cb18974160f6ff32adf4720983f40e8bdbb40da2dc6f160c99e95feb"
    end
  end

  # gh CLI 是运行时依赖（GitHub 平台需要）
  # glab 和 gc 是可选的（GitLab/GitCode 平台需要）
  depends_on "gh"

  def install
    bin.install "gf"

    # 安装 Shell 补全
    generate_completions_from_executable(bin/"gf", "completions")
  end

  test do
    system "#{bin}/gf", "--version"
    system "#{bin}/gf", "--help"
  end
end
