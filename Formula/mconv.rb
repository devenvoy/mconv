class Mconv < Formula
  desc "Interactive high-performance TUI media converter powered by FFmpeg"
  homepage "https://github.com/mconv/mconv"
  url "https://github.com/mconv/mconv/archive/refs/tags/v2.0.0.tar.gz"
  # Compute sha256 via: curl -sL https://github.com/mconv/mconv/archive/refs/tags/v2.0.0.tar.gz | sha256sum
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "Apache-2.0"
  head "https://github.com/mconv/mconv.git", branch: "main"

  depends_on "rust" => :build
  depends_on "ffmpeg"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mconv v2.0.0", shell_output("#{bin}/mconv --version")
  end
end
