class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-aarch64-apple-darwin.tar.gz"
      sha256 "05018fae4b969207bb87b169242a07d029daca7b94bf76d6b435d31610c1738a"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-x86_64-apple-darwin.tar.gz"
      sha256 "8a3b03deed1cf90fa3fd1d469f898ea7fae88516efae364d9e243e1291e9e028"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "5de6ec506d6258f4f213994733f4ec6b58c107c767330bd616fac79a0b053b65"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.1.0/gf-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "6c59f88bbc32b131b6490f2c41cf5a545a3f758492973ad1cc3ed3339ebdf19c"
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
