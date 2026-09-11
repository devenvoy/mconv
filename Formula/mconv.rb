class Mconv < Formula
  desc "Interactive high-performance TUI media converter powered by FFmpeg"
  homepage "https://github.com/devenvoy/mconv"
  url "https://github.com/devenvoy/mconv/archive/refs/tags/v2.0.0.tar.gz"
  sha256 "4b8c4ba41ba6a2b27be62042e8b85f1d8be348985d0837d73550bc7a43f7ed50"
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
