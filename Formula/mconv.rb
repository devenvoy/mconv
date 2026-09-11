class Mconv < Formula
  desc "Interactive high-performance TUI media converter powered by FFmpeg"
  homepage "https://github.com/devenvoy/mconv"
  version "2.0.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/devenvoy/mconv/releases/download/v2.0.0/mconv-v2.0.0-macos-arm64.tar.gz"
      sha256 "0280127e41ea22183116440ece96bd60b64d4da5f8bb7b6ea10731bdfb0df3c3"
    else
      url "https://github.com/devenvoy/mconv/releases/download/v2.0.0/mconv-v2.0.0-macos-x86_64.tar.gz"
      sha256 "87191463de508ad5d2891a585a1b9eb3b8a4705cf796e55bef10c700a015a569"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/devenvoy/mconv/releases/download/v2.0.0/mconv-v2.0.0-linux-x86_64.tar.gz"
      sha256 "eeb6b476742d7dd1580acdedbce067df629422bb9bc6a19392f95ee0038b4329"
    end
  end

  depends_on "ffmpeg"

  def install
    bin.install "mconv"
  end

  test do
    assert_match "mconv v2.0.0", shell_output("#{bin}/mconv --version")
  end
end
