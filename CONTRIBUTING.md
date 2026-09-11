# Contributing to mconv

Thank you for your interest in contributing to `mconv`! We welcome contributions from the community.

## Development Setup

### Prerequisites
- **Rust toolchain** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **FFmpeg**: Required for media encoding operations.
- **Git**

### Clone & Build
```bash
git clone https://github.com/devenvoy/mconv.git
cd mconv

# Check compilation
cargo check

# Run tests
cargo test

# Build release binary
cargo build --release
```

## Code Guidelines

1. **Defensive Safety**: Never modify input files directly. Use atomic scratch naming (`.mconv_tmp_<stem>.<ext>`) and verify output integrity before renaming.
2. **Minimal Binary Footprint**: Keep dependencies minimal. Avoid pulling heavy crates (e.g. external datetime or regex frameworks) when standard library or POSIX `libc` suffices.
3. **No Emoji Clutter in CLI**: Keep terminal output clean, professional, and accessible across ASCII and UTF-8 terminal emulators.
4. **Self-Documenting Code**: Write idiomatic Rust with clear function names, explicit error handling, and documentation in `DOCUMENTATION.md`.

## Pull Request Process

1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feature/my-new-feature
   ```
2. Ensure all tests pass:
   ```bash
   cargo test
   ```
3. Format and lint your code:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```
4. Commit using clear, conventional commit messages:
   - `feat: add support for AV1 webm transcode`
   - `fix: resolve subtitle stream indexing error`
   - `docs: update hardware acceleration matrix`
5. Open a Pull Request with a clear description of your changes.

## License
By contributing to `mconv`, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE).
