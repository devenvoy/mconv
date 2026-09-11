#![allow(clippy::needless_borrows_for_generic_args)]

use colored::*;
use mconv::converter::{check_ffmpeg, run_conversion, ConversionConfig};
use mconv::formats::{get_category, get_targets_for_category, MediaCategory};
use mconv::logger::Logger;
use mconv::picker::{self, pick_folder_native};
use mconv::scanner::{find_matching_files, scan_extensions};
use mconv::ui::banner::{print_banner, print_section_header};
use mconv::ui::prompt::{input_text, select_option, select_yes_no, PromptOption};
use mconv::ui::summary::{print_summary, ConversionSummary};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const VERSION: &str = "2.0.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!(
                    "mconv v{} - Interactive High-Performance Media Converter",
                    VERSION
                );
                println!();
                println!("USAGE:");
                println!("    mconv                Launch interactive terminal interface");
                println!("    mconv --version      Show version information");
                println!("    mconv --help         Show this help message");
                return;
            }
            "-v" | "--version" => {
                println!("mconv v{}", VERSION);
                return;
            }
            _ => {}
        }
    }

    if !check_ffmpeg() {
        eprintln!();
        eprintln!(
            "  {} {}",
            "✖".bright_red().bold(),
            "ffmpeg is required but was not found in PATH!"
                .bright_red()
                .bold()
        );
        eprintln!("  {} Please install ffmpeg:", "│".dimmed());
        eprintln!("  {}   macOS:  brew install ffmpeg", "│".dimmed());
        eprintln!("  {}   Ubuntu: sudo apt-get install ffmpeg", "│".dimmed());
        eprintln!("  {}   Arch:   sudo pacman -S ffmpeg", "│".dimmed());
        eprintln!();
        std::process::exit(1);
    }

    let logger = Arc::new(Logger::new());

    loop {
        print_banner();

        let menu_options = vec![
            PromptOption::with_hint(
                "Convert files in a folder",
                "Batch convert audio, video, images, or subtitles",
            ),
            PromptOption::with_hint(
                "Find files by extension",
                "Search and index files in a folder",
            ),
            PromptOption::with_hint(
                "View audit log",
                "Inspect current or previous conversion logs",
            ),
            PromptOption::with_hint("Exit", "Quit mconv"),
        ];

        let selection = select_option(
            "What would you like to do?",
            Some("Use Up/Down arrow keys to navigate, Enter to select"),
            &menu_options,
            0,
        );

        match selection {
            Some(0) => flow_convert(&logger),
            Some(1) => flow_find(),
            Some(2) => flow_view_log(&logger),
            _ => {
                println!();
                println!(
                    "  {} {}",
                    "✔".bright_green().bold(),
                    "Goodbye!".bright_white().bold()
                );
                println!();
                break;
            }
        }
    }
}

fn select_folder_flow() -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cur_str = current_dir.display().to_string();

    loop {
        let picker_options = vec![
            PromptOption::with_hint(
                "[OS]   Open Native OS Folder Picker",
                "Choose directory with macOS Finder / OS dialog",
            ),
            PromptOption::with_hint(
                format!("[DIR]  Current Directory ({})", cur_str),
                "Use the current working directory",
            ),
            PromptOption::with_hint(
                "[PATH] Enter / Paste Path Manually",
                "Type a custom folder path",
            ),
            PromptOption::with_hint(
                "[TREE] Browse Folder Tree",
                "Explore subdirectories directly in terminal",
            ),
        ];

        let choice = select_option(
            "Select target folder",
            Some("Pick how you want to choose the folder"),
            &picker_options,
            0,
        )?;

        match choice {
            0 => {
                println!(
                    "  {}  {}",
                    "│".dimmed(),
                    "Opening native OS file dialog...".cyan()
                );
                if let Some(folder) = pick_folder_native() {
                    println!(
                        "  {}  {} {}",
                        "◇".bright_green().bold(),
                        "OS Folder selected:".dimmed(),
                        folder.display().to_string().bright_white().bold()
                    );
                    println!("  {}", "│".dimmed());
                    return Some(folder);
                } else {
                    println!(
                        "  {}  {}",
                        "│".dimmed(),
                        "Native folder picker was cancelled or closed.".yellow()
                    );
                    println!("  {}", "│".dimmed());
                    continue;
                }
            }
            1 => return Some(current_dir),
            2 => {
                if let Some(typed) = input_text("Enter directory path", None, Some(&cur_str)) {
                    let path = PathBuf::from(typed);
                    if path.is_dir() {
                        return Some(path);
                    } else {
                        println!(
                            "  {}  {} {}",
                            "✖".bright_red().bold(),
                            "Not a valid directory:".red(),
                            path.display()
                        );
                        println!("  {}", "│".dimmed());
                    }
                }
            }
            3 => {
                if let Some(path) = browse_folders_tui(&current_dir) {
                    return Some(path);
                }
            }
            _ => return None,
        }
    }
}

fn browse_folders_tui(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let subdirs = picker::list_subdirectories(&current);
        let mut opts = vec![
            PromptOption::with_hint("✔ [ SELECT THIS FOLDER ]", current.display().to_string()),
            PromptOption::new("⬆ [ .. Parent Folder ]"),
        ];

        for d in &subdirs {
            let name = d
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            opts.push(PromptOption::new(format!("[DIR] {}", name)));
        }

        let sel = select_option(
            &format!("Browse: {}", current.display()),
            Some("Select a subdirectory to navigate or choose 'SELECT THIS FOLDER'"),
            &opts,
            0,
        )?;

        if sel == 0 {
            return Some(current);
        } else if sel == 1 {
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            }
        } else {
            let idx = sel - 2;
            if idx < subdirs.len() {
                current = subdirs[idx].clone();
            }
        }
    }
}

fn flow_convert(logger: &Arc<Logger>) {
    let target_dir = match select_folder_flow() {
        Some(d) => d,
        None => return,
    };

    let recurse_opts = vec![
        PromptOption::with_hint(
            "No  (top-level folder only)",
            "Fastest, does not inspect subfolders",
        ),
        PromptOption::with_hint(
            "Yes (scan recursively)",
            "Includes files inside all nested subdirectories",
        ),
    ];
    let rec_choice = match select_option("Include subfolders?", None, &recurse_opts, 0) {
        Some(c) => c == 1,
        None => return,
    };

    println!(
        "  {}  {}",
        "│".dimmed(),
        "Scanning folder for media files...".dimmed()
    );
    let ext_stats = scan_extensions(&target_dir, rec_choice);

    if ext_stats.is_empty() {
        println!();
        println!(
            "  {}  {}",
            "✖".bright_red().bold(),
            "No files found in selected directory.".bright_red().bold()
        );
        println!("  {}", "│".dimmed());
        return;
    }

    let from_options: Vec<PromptOption> = ext_stats
        .iter()
        .map(|s| {
            let cat = get_category(&s.ext);
            let cat_label = if cat != MediaCategory::Unknown {
                format!(" ({})", cat)
            } else {
                String::new()
            };
            PromptOption::with_hint(
                format!(".{}", s.ext),
                format!("{} file(s){}", s.count, cat_label),
            )
        })
        .collect();

    let from_idx = match select_option(
        "Convert FROM which format?",
        Some("Select the source file extension to batch convert"),
        &from_options,
        0,
    ) {
        Some(idx) => idx,
        None => return,
    };

    let from_ext = &ext_stats[from_idx].ext;
    let category = get_category(from_ext);

    if category == MediaCategory::Unknown {
        println!();
        println!(
            "  {}  Extension '.{}' is not a recognized media format (video/audio/image/subtitle).",
            "✖".bright_red().bold(),
            from_ext
        );
        println!("  {}", "│".dimmed());
        return;
    }

    let target_formats = get_targets_for_category(category, from_ext);
    if target_formats.is_empty() {
        println!();
        println!(
            "  {}  No available target formats for .{}",
            "✖".bright_red().bold(),
            from_ext
        );
        println!("  {}", "│".dimmed());
        return;
    }

    let to_options: Vec<PromptOption> = target_formats
        .iter()
        .map(|&t| match t {
            "mp4" => PromptOption::with_hint(".mp4", "Universal H.264 + AAC, high compatibility"),
            "mkv" => {
                PromptOption::with_hint(".mkv", "Matroska container with full metadata support")
            }
            "webm" => {
                PromptOption::with_hint(".webm", "VP9 + Opus for high efficiency web streaming")
            }
            "gif" => {
                PromptOption::with_hint(".gif", "High quality animated GIF with Lanczos scaling")
            }
            "mp3" => PromptOption::with_hint(".mp3", "Universal MP3 audio (LAME VBR Q2)"),
            "flac" => PromptOption::with_hint(".flac", "Lossless audio compression"),
            "wav" => PromptOption::with_hint(".wav", "Uncompressed PCM audio"),
            "aac" | "m4a" => {
                PromptOption::with_hint(format!(".{}", t), "Advanced Audio Coding (192kbps)")
            }
            "webp" => PromptOption::with_hint(".webp", "High compression web image format"),
            "png" => PromptOption::with_hint(".png", "Lossless image format"),
            "jpg" => PromptOption::with_hint(".jpg", "High quality JPEG image"),
            _ => PromptOption::new(format!(".{}", t)),
        })
        .collect();

    let to_idx = match select_option(
        "Convert TO which format?",
        Some(&format!("Source is .{} ({})", from_ext, category)),
        &to_options,
        0,
    ) {
        Some(idx) => idx,
        None => return,
    };

    let to_ext = target_formats[to_idx].to_string();

    let del_options = vec![
        PromptOption::with_hint(
            "No  - Keep original files",
            "Safest: originals remain untouched alongside new files",
        ),
        PromptOption::with_hint(
            "Yes - Delete originals after successful conversion",
            "Only deletes after output is verified non-empty",
        ),
    ];

    let delete_originals = match select_option(
        "Delete originals after converting?",
        Some(
            "Safety guarantee: original file is NEVER deleted if conversion fails or target exists",
        ),
        &del_options,
        0,
    ) {
        Some(c) => c == 1,
        None => return,
    };

    let files_to_convert = find_matching_files(&target_dir, from_ext, rec_choice);
    if files_to_convert.is_empty() {
        println!();
        println!(
            "  {}  No matching .{} files found to convert.",
            "✖".bright_red().bold(),
            from_ext
        );
        println!("  {}", "│".dimmed());
        return;
    }

    let total_input_bytes: u64 = files_to_convert
        .iter()
        .filter_map(|p| fs::metadata(p).ok().map(|m| m.len()))
        .sum();
    let free_disk = mconv::converter::get_available_disk_space(&target_dir);

    let num_cpus = num_cpus_detected();
    let (auto_rec, worker_options) = match category {
        MediaCategory::Video => {
            let rec = num_cpus.clamp(2, 3);
            (
                rec,
                vec![
                    PromptOption::with_hint(
                        format!("Automatic (Recommended: {} workers)", rec),
                        "Optimal video balance: avoids disk freeze, overheating & memory pressure",
                    ),
                    PromptOption::with_hint(
                        "Conservative (2 workers)",
                        "Smooth multitasking: keeps PC quiet, responsive, low heat",
                    ),
                    PromptOption::with_hint(
                        "Sequential (1 worker)",
                        "Safest: minimal disk space, RAM usage & thermal load",
                    ),
                    PromptOption::with_hint(
                        "Aggressive (4 workers)",
                        "High throughput for fast NVMe SSDs with plenty of free space",
                    ),
                    PromptOption::with_hint(
                        format!("Max Power ({} workers)", num_cpus),
                        "Warning: High disk I/O, heavy RAM usage, may cause system lag",
                    ),
                ],
            )
        }
        _ => {
            let rec = num_cpus.min(6);
            (
                rec,
                vec![
                    PromptOption::with_hint(
                        format!("Automatic (Recommended: {} workers)", rec),
                        "Fast multi-threaded processing",
                    ),
                    PromptOption::new("2 parallel workers"),
                    PromptOption::new("4 parallel workers"),
                    PromptOption::new("1 sequential worker"),
                ],
            )
        }
    };

    let workers_choice = match select_option(
        "Parallel conversion workers",
        Some("Select concurrency level based on your system & disk workload"),
        &worker_options,
        0,
    ) {
        Some(c) => match category {
            MediaCategory::Video => match c {
                0 => auto_rec,
                1 => 2,
                2 => 1,
                3 => 4,
                4 => num_cpus,
                _ => auto_rec,
            },
            _ => match c {
                0 => auto_rec,
                1 => 2,
                2 => 4,
                3 => 1,
                _ => auto_rec,
            },
        },
        None => return,
    };

    let gpu = mconv::converter::detect_gpu_backend();
    let summary = ConversionSummary {
        folder: &target_dir,
        recursive: rec_choice,
        from_ext,
        to_ext: &to_ext,
        category: &category.to_string(),
        file_count: files_to_convert.len(),
        total_input_size: total_input_bytes,
        free_disk_space: free_disk,
        delete_originals,
        workers: workers_choice,
        gpu_backend: gpu.name(),
        log_path: logger.path(),
    };

    print_summary(&summary);

    let confirm = select_yes_no(
        "Ready to begin conversion?",
        Some("No changes are made until you confirm"),
        true,
    );

    if confirm != Some(true) {
        println!(
            "  {}  {}",
            "ℹ".bright_yellow().bold(),
            "Conversion cancelled. No files were modified.".yellow()
        );
        println!("  {}", "│".dimmed());
        return;
    }

    println!();
    println!(
        "  {} Converting {} file(s) with {} parallel worker(s):",
        "»".bright_cyan().bold(),
        files_to_convert.len().to_string().bold().bright_white(),
        workers_choice.to_string().bold().bright_cyan()
    );
    println!(
        "  {}",
        "──────────────────────────────────────────────────────────────".dimmed()
    );

    let config = ConversionConfig {
        to_ext,
        category,
        delete_originals,
        workers: workers_choice,
    };

    let (converted, skipped, failed, elapsed) =
        run_conversion(&files_to_convert, &config, Arc::clone(logger));

    println!(
        "  {}",
        "──────────────────────────────────────────────────────────────".dimmed()
    );
    println!();
    println!(
        "  {} {}",
        "✔".bright_green().bold(),
        "Batch processing complete!".bold().bright_white()
    );
    println!(
        "  {}  Converted:  {}",
        "│".dimmed(),
        converted.to_string().bold().bright_green()
    );
    println!(
        "  {}  Skipped:    {}",
        "│".dimmed(),
        skipped.to_string().bold().bright_yellow()
    );
    println!(
        "  {}  Failed:     {}",
        "│".dimmed(),
        failed.to_string().bold().bright_red()
    );
    println!("  {}  Total Time: {:.1} seconds", "│".dimmed(), elapsed);
    println!(
        "  {}  Audit Log:  {}",
        "│".dimmed(),
        logger.path().display().to_string().dimmed()
    );
    println!();
}

fn flow_find() {
    let target_dir = match select_folder_flow() {
        Some(d) => d,
        None => return,
    };

    let recurse_opts = vec![
        PromptOption::new("No  (top-level folder only)"),
        PromptOption::new("Yes (include all subfolders)"),
    ];
    let rec_choice = match select_option("Include subfolders?", None, &recurse_opts, 0) {
        Some(c) => c == 1,
        None => return,
    };

    let ext_stats = scan_extensions(&target_dir, rec_choice);
    if ext_stats.is_empty() {
        println!(
            "  {}  No files found in directory.",
            "✖".bright_red().bold()
        );
        return;
    }

    let options: Vec<PromptOption> = ext_stats
        .iter()
        .map(|s| PromptOption::with_hint(format!(".{}", s.ext), format!("{} file(s)", s.count)))
        .collect();

    let sel = match select_option("Find files of which extension?", None, &options, 0) {
        Some(s) => s,
        None => return,
    };

    let ext = &ext_stats[sel].ext;
    let matched = find_matching_files(&target_dir, ext, rec_choice);

    println!();
    println!(
        "  {} Found {} file(s) with extension .{}:",
        "✔".bright_green().bold(),
        matched.len(),
        ext
    );
    for (i, file) in matched.iter().enumerate().take(30) {
        println!("  {}  {: >2}. {}", "│".dimmed(), i + 1, file.display());
    }
    if matched.len() > 30 {
        println!(
            "  {}  ... and {} more files",
            "│".dimmed(),
            matched.len() - 30
        );
    }
    println!();
}

fn flow_view_log(logger: &Logger) {
    let log_path = logger.path();
    if log_path.exists() {
        if let Ok(content) = fs::read_to_string(log_path) {
            println!();
            print_section_header("Audit Log Content", Some(&log_path.display().to_string()));
            let lines: Vec<&str> = content.lines().collect();
            let start_idx = if lines.len() > 25 {
                lines.len() - 25
            } else {
                0
            };
            for line in &lines[start_idx..] {
                println!("  {} {}", "│".dimmed(), line);
            }
            if lines.is_empty() {
                println!("  {} (log is currently empty)", "│".dimmed());
            }
            println!();
            return;
        }
    }
    println!("  {} Log file not found or empty.", "ℹ".yellow());
}

fn num_cpus_detected() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
}
