class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.2.0/gf-aarch64-apple-darwin.tar.gz"
      sha256 "cf8cc034c15866ca94da7175531c308e4b1e013da478911645bd678959138c3a"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.2.0/gf-x86_64-apple-darwin.tar.gz"
      sha256 "68de790202600dcd4c7f747a48bd730d5c241163ecc1383994c6e6ec04bd298c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.2.0/gf-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "b02c560db916618aa646e3f40bd5cf07d291a7abdf9528e985e9b268ad82d3bf"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.2.0/gf-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "7ddab83316a01bd6ce18229457c368f52bfff8d78ed808e6cedff20a9faa1073"
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
