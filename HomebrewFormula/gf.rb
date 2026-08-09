class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-aarch64-apple-darwin.tar.gz"
      sha256 "df8c6cad0e9990ffb5ab62ea3ee527575c87c412db0b82d2e4a55323b2a6010e"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-x86_64-apple-darwin.tar.gz"
      sha256 "68e757cabec105647457898594c5e84dcc07a9acd2205f385ed66e9aeddc8fa6"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "940d2857a951c28097ec106648e76f9f157a5bb1d2661bb52c7416cadb14caf8"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "5f947f8ea702cc9be6449c6bb3ad53cdea89eb58d12861667f81e38f058b833d"
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
