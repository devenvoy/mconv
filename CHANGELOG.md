# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-09-11

### Added
- Complete rewrite from legacy bash/dialog script into high-performance native Rust application.
- Modern Vite / `@clack/prompts`-style step-by-step interactive CLI interface.
- Native OS directory dialog integration for macOS (Finder `osascript`), Linux (`zenity`/`kdialog`), and Windows (PowerShell).
- Real-time parallel multi-progress bar display via `indicatif::MultiProgress` tracking worker speed, %, elapsed, and ETA.
- Multi-tier video remux engine with direct bitstream copying (`-c copy`) achieving 1000x–3000x real-time speedup.
- Hardware acceleration engine auto-detecting Apple Silicon VideoToolbox (`h264_videotoolbox`), NVIDIA NVENC, and Intel QuickSync.
- Defensive concurrency throttling and free disk space pre-flight checks (`libc::statvfs`).
- Thread-safe isolated scratch file architecture (`.mconv_tmp_<stem>.<ext>`) with zero-risk atomic renames.
- Comprehensive developer architecture guide (`DOCUMENTATION.md`) and user guide (`USER_GUIDE.md`).
- Homebrew formula and multi-platform GitHub Actions CI/CD release pipeline.

### Changed
- Release binary footprint reduced to 608 KB using whole-program Link-Time Optimization (`lto`), symbol stripping, and POSIX `libc` integration.
- Removed legacy installer scripts in favor of Homebrew tap and standalone binary releases.
- Cleaned CLI output of emoji clutter in favor of clean text glyphs and dedicated vector SVG assets.

### Security
- Target collision protection prevents accidental overwriting of existing media.
- Input files are guaranteed untouched on transcode errors or user cancellations.
