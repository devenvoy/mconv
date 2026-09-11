class Mconv < Formula
  desc "Interactive high-performance TUI media converter powered by FFmpeg"
  homepage "https://github.com/devenvoy/mconv"
  url "https://github.com/devenvoy/mconv/archive/refs/tags/v2.0.0.tar.gz"
  sha256 "2d7e501843e2bbd7b39595d39ffc9dedd3c498101d3337dfde707a435c2f9e77"
  license "Apache-2.0"
  head "https://github.com/devenvoy/mconv.git", branch: "main"

  depends_on "rust" => :build
  depends_on "ffmpeg"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mconv v2.0.0", shell_output("#{bin}/mconv --version")
  end
end
