#!/usr/bin/env bash
#
# build.sh - Compiles mconv into a standalone, highly-optimized native binary using Rust.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${SCRIPT_DIR}/dist"
OUT_BIN="${OUT_DIR}/mconv"

have() { command -v "$1" >/dev/null 2>&1; }

if ! have cargo; then
  echo "ERROR: Cargo / Rust is required to build mconv v2.0." >&2
  echo "Install Rust via https://rustup.rs/ :" >&2
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" >&2
  exit 1
fi

echo "Compiling mconv (Release mode)..."
cd "$SCRIPT_DIR"
cargo build --release

mkdir -p "$OUT_DIR"
cp target/release/mconv "$OUT_BIN"
chmod +x "$OUT_BIN"

echo ""
echo "Successfully built standalone native executable: $OUT_BIN"
file "$OUT_BIN" 2>/dev/null || true
echo ""
echo "Verification check:"
"$OUT_BIN" --version
echo ""
echo "To install globally:"
echo "  sudo cp '$OUT_BIN' /usr/local/bin/mconv"
echo "Or run directly:"
echo "  $OUT_BIN"
