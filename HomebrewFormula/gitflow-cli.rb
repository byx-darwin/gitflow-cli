# Homebrew Formula for gitflow-cli
#
# 安装方式：
#   brew tap byx-darwin/gitflow-cli
#   brew install gitflow-cli
#
# 或直接通过 URL：
#   brew install --build-from-source ./HomebrewFormula/gitflow-cli.rb

class GitflowCli < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  url "https://github.com/byx-darwin/gitflow-cli/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "" # 由 GitHub Release workflow 自动更新，或通过 `brew fetch --build-from-source` 获取
  license "MIT"
  version "0.1.0"

  depends_on "rust" => :build

  # gh CLI 是运行时依赖（GitHub 平台需要）
  # glab 和 gc 是可选的（GitLab/GitCode 平台需要）
  depends_on "gh"

  def install
    system "cargo", "install", *std_cargo_args

    # 安装 Shell 补全
    generate_completions_from_executable(bin/"gitflow", "completions")
  end

  test do
    system "#{bin}/gitflow", "--version"
    system "#{bin}/gitflow", "--help"
    # 验证补全生成能力
    system "#{bin}/gitflow", "completions", "bash"
  end
end
