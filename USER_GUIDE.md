# mconv User Guide & Manual

A comprehensive guide to using the `mconv` interactive terminal media converter.

---

## 1. Getting Started

### Prerequisites
`mconv` requires **FFmpeg** to be installed on your system.

- **macOS**: `brew install ffmpeg`
- **Ubuntu / Debian**: `sudo apt update && sudo apt install ffmpeg`
- **Arch Linux**: `sudo pacman -S ffmpeg`
- **Fedora**: `sudo dnf install ffmpeg`
- **Windows**: `winget install Gyan.FFmpeg` or `choco install ffmpeg`

Verify your installation:
```bash
ffmpeg -version
mconv --version
```

---

## 2. Interactive Navigation

`mconv` uses a Vite / `@clack/prompts`-inspired keyboard workflow:

- **Arrow Keys (Up / Down)** or **`k` / `j`**: Navigate between menu options.
- **Enter**: Confirm selection and proceed to the next step.
- **Esc** or **`q`**: Cancel current prompt or exit.
- **Ctrl + C**: Abort program immediately.

Completed prompts automatically collapse into compact summary lines:
```
◇ Select target folder › /Users/devanshpc/Movies
◇ Include subfolders? › Yes (scan recursively)
◇ Convert FROM which format? › .mkv (12 files)
◇ Convert TO which format? › .mp4 (video)
```

---

## 3. Folder Selection Methods

When selecting a directory, `mconv` presents four methods:

1. **[OS] Open Native OS Folder Picker**:
   - Spawns your operating system's native directory chooser (macOS Finder dialog via AppleScript, `zenity`/`kdialog` on Linux, PowerShell on Windows).
2. **[DIR] Current Directory**:
   - Instantly selects the directory where you launched `mconv`.
3. **[PATH] Enter / Paste Path Manually**:
   - Allows typing or pasting an absolute or relative path with auto-completion support.
4. **[TREE] Browse Folder Tree**:
   - Interactive terminal browser to drill down into subdirectories or navigate up to parent folders.

---

## 4. Format Conversion Matrix

| Media Type | Input Extensions | Recommended Target Formats | Notes |
| :--- | :--- | :--- | :--- |
| **Video** | `mp4`, `mkv`, `avi`, `mov`, `wmv`, `flv`, `webm`, `mpg`, `mpeg`, `m4v`, `ts`, `3gp` | `mp4`, `mkv`, `webm`, `avi`, `mov`, `flv`, `gif` | Stream copy (`-c copy`) enabled for compatible streams |
| **Audio** | `mp3`, `wav`, `flac`, `aac`, `ogg`, `m4a`, `wma`, `opus`, `aiff` | `mp3`, `aac`, `m4a`, `flac`, `wav`, `ogg`, `opus`, `wma` | High-quality presets (LAME VBR Q2, Opus 128k, etc.) |
| **Image** | `jpg`, `jpeg`, `png`, `gif`, `bmp`, `webp`, `tiff`, `tif` | `png`, `jpg`, `webp`, `bmp`, `gif`, `tiff` | Single frame conversion with format-optimized compression |
| **Subtitle** | `srt`, `vtt`, `ass`, `ssa`, `sub` | `srt`, `vtt`, `ass` | Stream extraction and conversion |

---

## 5. Parallel Concurrency Presets

`mconv` adapts its worker pool depending on the media category:

### Video Presets
- **Automatic (Recommended)**: Clamped between 2 and 3 concurrent workers. Balances multi-core CPU/GPU utilization without saturating disk I/O or overheating laptops.
- **Conservative (2 workers)**: Keeps system whisper-quiet, fans low, and desktop responsive.
- **Sequential (1 worker)**: Lowest memory footprint, zero thermal stress. Ideal for older hardware or battery power.
- **Aggressive (4 workers)**: High throughput for fast NVMe PCIe SSDs.
- **Max Power (All logical cores)**: Unthrottled. Best for dedicated render workstations.

### Audio & Image Presets
- Defaults up to 6 parallel workers for rapid I/O throughput on smaller files.

---

## 6. Safety & Rollback Guarantees

`mconv` follows strict non-destructive safety principles:
1. **Target Collision Protection**: If the target filename (e.g. `video.mp4`) already exists, `mconv` skips the file with `[SKIP]` to prevent accidental overwrite.
2. **Hidden Scratch Files**: Conversions write to hidden scratch files (`.mconv_tmp_<stem>.<ext>`) in the target directory.
3. **Atomic Renames**: Once verified non-empty, the scratch file is renamed atomically using `std::fs::rename`.
4. **Clean Rollback**: If FFmpeg encounters an error or is interrupted, the scratch file is deleted immediately. The original file is **never touched**.
5. **Original Deletion Safeguard**: Original input files are only removed if you explicitly chose "Delete originals", and only after the target output is fully verified.

---

## 7. Audit Logging

Every conversion session automatically logs timestamps, input/output paths, execution durations, methods (remux vs transcode), and errors to:
```
~/.mconv/logs/mconv_<YYYYMMDD_HHMMSS>_<PID>.log
```
You can inspect recent logs directly inside `mconv` by selecting **"View audit log"** from the main menu.
