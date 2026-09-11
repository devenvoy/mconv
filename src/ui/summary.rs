use crate::converter::format_size;
use colored::*;
use std::path::Path;

pub struct ConversionSummary<'a> {
    pub folder: &'a Path,
    pub recursive: bool,
    pub from_ext: &'a str,
    pub to_ext: &'a str,
    pub category: &'a str,
    pub file_count: usize,
    pub total_input_size: u64,
    pub free_disk_space: Option<u64>,
    pub delete_originals: bool,
    pub workers: usize,
    pub gpu_backend: &'a str,
    pub log_path: &'a Path,
}

pub fn print_summary(summary: &ConversionSummary) {
    let top    = "  ╭─ Conversion Plan Summary ──────────────────────────────────────────╮".cyan().bold();
    let bottom = "  ╰────────────────────────────────────────────────────────────────────╯".cyan().bold();
    let border = "│".cyan().bold();

    println!("{}", top);
    println!("  {}  {: <14} {}", border, "Target Folder:".dimmed(), summary.folder.display().to_string().bright_white().bold());
    println!("  {}  {: <14} {}", border, "Subfolders:".dimmed(), if summary.recursive { "Included (Recursive scan)".bright_yellow() } else { "No (Top folder only)".bright_white() });
    println!("  {}  {: <14} {} {} {} ({})",
        border,
        "Conversion:".dimmed(),
        format!(".{}", summary.from_ext).cyan().bold(),
        "➔".bright_magenta(),
        format!(".{}", summary.to_ext).bright_green().bold(),
        summary.category.bright_cyan()
    );
    println!("  {}  {: <14} {} ({})",
        border,
        "Files Found:".dimmed(),
        format!("{} files", summary.file_count).bright_yellow().bold(),
        format_size(summary.total_input_size).dimmed()
    );

    if let Some(free) = summary.free_disk_space {
        let is_tight = free < summary.total_input_size;
        let free_str = format_size(free);
        let free_display = if is_tight {
            format!("{} ([!] TIGHT DISK SPACE)", free_str).bright_red().bold()
        } else {
            free_str.bright_green()
        };
        println!("  {}  {: <14} {}", border, "Free Disk Space:".dimmed(), free_display);

        if is_tight {
            println!("  {}  {: <14} {}", border, "Disk Notice:".bright_yellow().bold(), "Available drive space is close to batch size.".bright_yellow());
        }
    }

    println!("  {}  {: <14} {}", border, "Delete Original:".dimmed(), if summary.delete_originals { "YES (safe delete after verification)".bright_red().bold() } else { "NO (keep original files)".bright_green() });
    println!("  {}  {: <14} {}", border, "Concurrency:".dimmed(), format!("{} parallel worker(s)", summary.workers).bright_cyan());
    println!("  {}  {: <14} {}", border, "Acceleration:".dimmed(), summary.gpu_backend.bright_magenta().bold());
    println!("  {}  {: <14} {}", border, "Audit Log:".dimmed(), summary.log_path.display().to_string().dimmed());
    println!("{}", bottom);
    println!("  {}", "│".dimmed());
}
