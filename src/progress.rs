use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct ConversionStats {
    pub converted: AtomicUsize,
    pub skipped: AtomicUsize,
    pub failed: AtomicUsize,
    pub start_time: Instant,
}

impl Default for ConversionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionStats {
    pub fn new() -> Self {
        Self {
            converted: AtomicUsize::new(0),
            skipped: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn inc_ok(&self) {
        self.converted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_skip(&self) {
        self.skipped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_fail(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn summary(&self) -> (usize, usize, usize, f64) {
        (
            self.converted.load(Ordering::Relaxed),
            self.skipped.load(Ordering::Relaxed),
            self.failed.load(Ordering::Relaxed),
            self.start_time.elapsed().as_secs_f64(),
        )
    }
}

pub struct MultiProgressReporter {
    mp: Arc<MultiProgress>,
    overall_pb: ProgressBar,
    stats: Arc<ConversionStats>,
}

impl MultiProgressReporter {
    pub fn new(total: usize) -> Self {
        let mp = Arc::new(MultiProgress::new());
        let overall_pb = mp.add(ProgressBar::new(total as u64));

        let overall_style = ProgressStyle::default_bar()
            .template("  Overall: [{elapsed_precise}] [{bar:28.cyan/blue}] {pos}/{len} ({percent:>3}%) • ETA: {eta_precise}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓░");

        overall_pb.set_style(overall_style);
        overall_pb.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            mp,
            overall_pb,
            stats: Arc::new(ConversionStats::new()),
        }
    }

    pub fn create_file_progress(&self, display_name: &str) -> ProgressBar {
        let pb = self.mp.add(ProgressBar::new(100));
        let style = ProgressStyle::default_bar()
            .template("  {spinner:.bright_cyan} [{elapsed_precise}] [{bar:24.bright_magenta/blue}] {percent:>3}%  {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓░");

        pb.set_style(style);
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb.set_message(display_name.to_string());
        pb
    }

    pub fn complete_ok(
        &self,
        pb: ProgressBar,
        file_name: &str,
        out_name: &str,
        duration_secs: f64,
        size_str: &str,
        method: &str,
    ) {
        self.stats.inc_ok();
        self.overall_pb.inc(1);

        let finish_style = ProgressStyle::default_bar().template("{msg}").unwrap();
        pb.set_style(finish_style);

        let line = format!(
            "  {}  {} {} {} {} {} ({:.1}s, {}, {})",
            "│".dimmed(),
            "✔".bright_green().bold(),
            "[OK  ]".bright_green(),
            file_name.bright_white(),
            "➔".dimmed(),
            out_name.bright_cyan(),
            duration_secs,
            size_str.bright_yellow(),
            method.dimmed()
        );
        pb.finish_with_message(line);
    }

    pub fn complete_skip(&self, pb: ProgressBar, file_name: &str, reason: &str) {
        self.stats.inc_skip();
        self.overall_pb.inc(1);

        let finish_style = ProgressStyle::default_bar().template("{msg}").unwrap();
        pb.set_style(finish_style);

        let line = format!(
            "  {}  {} {} {} ({})",
            "│".dimmed(),
            "ℹ".bright_yellow().bold(),
            "[SKIP]".bright_yellow(),
            file_name.bright_white(),
            reason.dimmed()
        );
        pb.finish_with_message(line);
    }

    pub fn complete_fail(&self, pb: ProgressBar, file_name: &str, error: &str) {
        self.stats.inc_fail();
        self.overall_pb.inc(1);

        let finish_style = ProgressStyle::default_bar().template("{msg}").unwrap();
        pb.set_style(finish_style);

        let line = format!(
            "  {}  {} {} {} (reverted to original, {})",
            "│".dimmed(),
            "✖".bright_red().bold(),
            "[FAIL]".bright_red(),
            file_name.bright_white(),
            error.bright_red()
        );
        pb.finish_with_message(line);
    }

    pub fn finish(self) -> (usize, usize, usize, f64) {
        self.overall_pb.finish_and_clear();
        self.stats.summary()
    }
}
