class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.0.0/gf-aarch64-apple-darwin.tar.gz"
      sha256 "7bd7a8baae9bf3b46bc0a9ba0d8705949e3282f6fa3895a8072aaf78bcdc63bd"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.0.0/gf-x86_64-apple-darwin.tar.gz"
      sha256 "15a231dc955a192bc76221d5159b87a5fa01fb92f272a9ae506e4a254b52711f"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.0.0/gf-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "92fb01bcbc66f07bd97ac6cba5091d65d7c700e00da1025ac11af2ffaf0454dd"
    else
      url "https://github.com/byx-darwin/gitflow-cli/releases/download/v1.0.0/gf-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "09ee7d1eb4a909b8f349214b13a7203c6dbbcab6fa163167edbdb1048f189411"
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
