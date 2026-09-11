use crate::formats::{get_encode_args, MediaCategory};
use crate::logger::Logger;
use crate::progress::MultiProgressReporter;
use indicatif::ProgressBar;
use rayon::ThreadPoolBuilder;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

pub struct ConversionConfig {
    pub to_ext: String,
    pub category: MediaCategory,
    pub delete_originals: bool,
    pub workers: usize,
}

pub fn check_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_media_duration(input: &Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .ok()
    } else {
        None
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn get_available_disk_space(_path: &Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c_path = CString::new(_path.as_os_str().as_bytes()).ok()?;
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
            Some(stat.f_bavail as u64 * stat.f_frsize as u64)
        } else {
            None
        }
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    AppleVideoToolbox,
    NvidiaNvenc,
    IntelQsv,
    None,
}

impl GpuBackend {
    pub fn name(&self) -> &'static str {
        match self {
            GpuBackend::AppleVideoToolbox => "Apple Silicon VideoToolbox (GPU/ASIC)",
            GpuBackend::NvidiaNvenc => "NVIDIA NVENC (GPU)",
            GpuBackend::IntelQsv => "Intel QuickSync (GPU)",
            GpuBackend::None => "Multi-Threaded CPU",
        }
    }
}

pub fn detect_gpu_backend() -> GpuBackend {
    static BACKEND: std::sync::OnceLock<GpuBackend> = std::sync::OnceLock::new();
    *BACKEND.get_or_init(|| {
        if let Ok(output) = Command::new("ffmpeg").arg("-encoders").output() {
            let str_out = String::from_utf8_lossy(&output.stdout);
            if str_out.contains("h264_videotoolbox") {
                return GpuBackend::AppleVideoToolbox;
            } else if str_out.contains("h264_nvenc") {
                return GpuBackend::NvidiaNvenc;
            } else if str_out.contains("h264_qsv") {
                return GpuBackend::IntelQsv;
            }
        }
        GpuBackend::None
    })
}

pub fn run_conversion(
    files: &[PathBuf],
    config: &ConversionConfig,
    logger: Arc<Logger>,
) -> (usize, usize, usize, f64) {
    let reporter = Arc::new(MultiProgressReporter::new(files.len()));

    let pool = ThreadPoolBuilder::new()
        .num_threads(config.workers)
        .build()
        .unwrap_or_else(|_| ThreadPoolBuilder::new().build().unwrap());

    pool.scope(|s| {
        for file in files {
            let file_clone = file.clone();
            let logger_clone = Arc::clone(&logger);
            let reporter_clone = Arc::clone(&reporter);
            let to_ext = config.to_ext.clone();
            let category = config.category;
            let delete_originals = config.delete_originals;

            s.spawn(move |_| {
                convert_single_file(
                    &file_clone,
                    &to_ext,
                    category,
                    delete_originals,
                    &logger_clone,
                    &reporter_clone,
                );
            });
        }
    });

    Arc::try_unwrap(reporter)
        .ok()
        .map(|r| r.finish())
        .unwrap_or((0, 0, 0, 0.0))
}

fn truncate_filename(name: &str, max_len: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() > max_len {
        let start = chars.len() - (max_len.saturating_sub(3));
        let tail: String = chars[start..].iter().collect();
        format!("...{}", tail)
    } else {
        name.to_string()
    }
}

fn convert_single_file(
    input: &Path,
    to_ext: &str,
    category: MediaCategory,
    delete_originals: bool,
    logger: &Logger,
    reporter: &MultiProgressReporter,
) {
    let start = Instant::now();
    let file_name = input
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let short_name = truncate_filename(&file_name, 35);

    let pb = reporter.create_file_progress(&short_name);

    if !input.is_file() {
        logger.fail(&format!("Not a regular file: {}", input.display()));
        reporter.complete_fail(pb, &short_name, "not a regular file");
        return;
    }

    let meta = match fs::metadata(input) {
        Ok(m) => m,
        Err(e) => {
            logger.fail(&format!(
                "Cannot read metadata: {} ({})",
                input.display(),
                e
            ));
            reporter.complete_fail(pb, &short_name, "cannot read file");
            return;
        }
    };

    if meta.len() == 0 {
        logger.fail(&format!(
            "Corrupted/empty file (0 bytes): {}",
            input.display()
        ));
        reporter.complete_fail(pb, &short_name, "empty file (0 bytes)");
        return;
    }

    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = match input.file_stem() {
        Some(s) => s.to_string_lossy().to_string(),
        None => {
            logger.fail(&format!("Invalid file stem: {}", input.display()));
            reporter.complete_fail(pb, &short_name, "invalid filename");
            return;
        }
    };

    let target_filename = format!("{}.{}", stem, to_ext);
    let target_path = parent.join(&target_filename);

    if target_path.exists() {
        logger.skip(&format!("Target already exists: {}", target_path.display()));
        reporter.complete_skip(pb, &short_name, "target exists");
        return;
    }

    let tmp_filename = format!(".mconv_tmp_{}.{}", stem, to_ext);
    let tmp_path = parent.join(&tmp_filename);
    let _ = fs::remove_file(&tmp_path);

    let duration = get_media_duration(input);

    let (success, method, last_error) = match category {
        MediaCategory::Video => {
            convert_video_pipeline(input, &tmp_path, to_ext, &pb, &short_name, duration)
        }
        MediaCategory::Audio => {
            convert_audio_pipeline(input, &tmp_path, to_ext, &pb, &short_name, duration)
        }
        MediaCategory::Image => {
            let encode_args = get_encode_args(category, to_ext);
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y")
                .arg("-nostdin")
                .arg("-i")
                .arg(input)
                .args(&encode_args)
                .args(&["-frames:v", "1", "-update", "1"])
                .args(&["-progress", "pipe:1", "-nostats"])
                .arg(&tmp_path);
            let (ok, err) = run_ffmpeg_pipe(&mut cmd, &pb, &short_name, None);
            (ok, "image conversion", err)
        }
        MediaCategory::Subtitle => {
            let mut cmd = Command::new("ffmpeg");
            cmd.arg("-y")
                .arg("-nostdin")
                .arg("-i")
                .arg(input)
                .args(&["-progress", "pipe:1", "-nostats"])
                .arg(&tmp_path);
            let (ok, err) = run_ffmpeg_pipe(&mut cmd, &pb, &short_name, None);
            (ok, "subtitle conversion", err)
        }
        MediaCategory::Unknown => {
            logger.fail(&format!("Unknown category: {}", input.display()));
            reporter.complete_fail(pb, &short_name, "unknown category");
            return;
        }
    };

    let valid_output = success
        && tmp_path.exists()
        && fs::metadata(&tmp_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false);

    if valid_output {
        if fs::rename(&tmp_path, &target_path).is_ok() {
            let duration_secs = start.elapsed().as_secs_f64();
            let size = fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);
            let size_str = format_size(size);

            logger.ok(&format!(
                "{} -> {} ({:.1}s, {})",
                input.display(),
                target_path.display(),
                duration_secs,
                method
            ));
            reporter.complete_ok(
                pb,
                &short_name,
                &target_filename,
                duration_secs,
                &size_str,
                method,
            );

            if delete_originals {
                let _ = fs::remove_file(input);
                logger.del(&format!("Deleted original: {}", input.display()));
            }
        } else {
            let _ = fs::remove_file(&tmp_path);
            logger.fail(&format!(
                "Atomic rename failed for: {}",
                target_path.display()
            ));
            reporter.complete_fail(pb, &short_name, "rename failed (original safe)");
        }
    } else {
        let _ = fs::remove_file(&tmp_path);
        let _ = fs::remove_file(&target_path);

        let err_display = if last_error
            .to_lowercase()
            .contains("no space left on device")
        {
            "DISK FULL: No space left on device".to_string()
        } else if last_error.is_empty() {
            "conversion failed (reverted to original)".to_string()
        } else {
            last_error
        };

        logger.fail(&format!(
            "Failed for: {} ({})",
            input.display(),
            err_display
        ));
        reporter.complete_fail(pb, &short_name, &err_display);
    }
}

fn convert_video_pipeline(
    input: &Path,
    tmp_path: &Path,
    to_ext: &str,
    pb: &ProgressBar,
    short_name: &str,
    duration: Option<f64>,
) -> (bool, &'static str, String) {
    if to_ext == "gif" {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(&[
            "-y",
            "-nostdin",
            "-i",
            &input.to_string_lossy(),
            "-vf",
            "fps=10,scale=480:-1:flags=lanczos",
            "-progress",
            "pipe:1",
            "-nostats",
        ])
        .arg(tmp_path);

        let (ok, err) = run_ffmpeg_pipe(&mut cmd, pb, short_name, duration);
        return (ok, "gif render", err);
    }

    let mut remux_cmd = Command::new("ffmpeg");
    remux_cmd.arg("-y").arg("-nostdin").arg("-i").arg(input);
    remux_cmd.args(&["-map", "0:v:0", "-map", "0:a?"]);
    remux_cmd.args(&["-c:v", "copy", "-c:a", "copy"]);
    if to_ext == "mp4" {
        remux_cmd.args(&["-movflags", "+faststart"]);
    }
    remux_cmd
        .args(&["-progress", "pipe:1", "-nostats"])
        .arg(tmp_path);

    let (remux_ok, remux_err) = run_ffmpeg_pipe(&mut remux_cmd, pb, short_name, duration);
    if remux_ok && is_valid_file(tmp_path) {
        return (true, "fast remux", String::new());
    }

    let _ = fs::remove_file(tmp_path);

    let mut remux_v_cmd = Command::new("ffmpeg");
    remux_v_cmd.arg("-y").arg("-nostdin").arg("-i").arg(input);
    remux_v_cmd.args(&["-map", "0:v:0", "-map", "0:a?"]);
    remux_v_cmd.args(&["-c:v", "copy", "-c:a", "aac", "-b:a", "192k"]);
    if to_ext == "mp4" {
        remux_v_cmd.args(&["-movflags", "+faststart"]);
    }
    remux_v_cmd
        .args(&["-progress", "pipe:1", "-nostats"])
        .arg(tmp_path);

    let (remux_v_ok, _) = run_ffmpeg_pipe(&mut remux_v_cmd, pb, short_name, duration);
    if remux_v_ok && is_valid_file(tmp_path) {
        return (true, "video remux (audio transcode)", String::new());
    }

    let _ = fs::remove_file(tmp_path);

    let mut enc_cmd = Command::new("ffmpeg");
    enc_cmd
        .arg("-y")
        .arg("-nostdin")
        .arg("-hwaccel")
        .arg("auto")
        .arg("-i")
        .arg(input);
    enc_cmd.args(&["-map", "0:v:0", "-map", "0:a?"]);

    let method = match detect_gpu_backend() {
        GpuBackend::AppleVideoToolbox if to_ext == "mp4" || to_ext == "mov" || to_ext == "mkv" => {
            enc_cmd.args(&[
                "-c:v",
                "h264_videotoolbox",
                "-b:v",
                "5M",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
            ]);
            "Apple Silicon GPU (VideoToolbox)"
        }
        GpuBackend::NvidiaNvenc if to_ext == "mp4" || to_ext == "mov" || to_ext == "mkv" => {
            enc_cmd.args(&[
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p4",
                "-cq",
                "22",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
            ]);
            "NVIDIA NVENC GPU"
        }
        GpuBackend::IntelQsv if to_ext == "mp4" || to_ext == "mov" || to_ext == "mkv" => {
            enc_cmd.args(&[
                "-c:v",
                "h264_qsv",
                "-global_quality",
                "22",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
            ]);
            "Intel QuickSync GPU"
        }
        _ if to_ext == "webm" => {
            enc_cmd.args(&[
                "-c:v",
                "libvpx-vp9",
                "-crf",
                "32",
                "-b:v",
                "0",
                "-deadline",
                "realtime",
                "-cpu-used",
                "4",
                "-c:a",
                "libopus",
                "-b:a",
                "128k",
            ]);
            "VP9 webm encode"
        }
        _ => {
            let encode_args = get_encode_args(MediaCategory::Video, to_ext);
            enc_cmd.args(&encode_args);
            "software fast encode"
        }
    };

    if to_ext == "mp4" {
        enc_cmd.args(&["-movflags", "+faststart"]);
    }
    enc_cmd
        .args(&["-progress", "pipe:1", "-nostats"])
        .arg(tmp_path);

    let (enc_ok, enc_err) = run_ffmpeg_pipe(&mut enc_cmd, pb, short_name, duration);
    let err = if enc_err.is_empty() {
        remux_err
    } else {
        enc_err
    };
    (enc_ok, method, err)
}

fn convert_audio_pipeline(
    input: &Path,
    tmp_path: &Path,
    to_ext: &str,
    pb: &ProgressBar,
    short_name: &str,
    duration: Option<f64>,
) -> (bool, &'static str, String) {
    let encode_args = get_encode_args(MediaCategory::Audio, to_ext);
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin");
    cmd.args(&["-vn", "-i"]).arg(input);
    cmd.args(&encode_args);
    cmd.args(&["-progress", "pipe:1", "-nostats"]).arg(tmp_path);

    let (ok, err) = run_ffmpeg_pipe(&mut cmd, pb, short_name, duration);
    (ok, "audio transcode", err)
}

fn is_valid_file(path: &Path) -> bool {
    path.exists() && fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}

fn run_ffmpeg_pipe(
    cmd: &mut Command,
    pb: &ProgressBar,
    display_name: &str,
    total_duration: Option<f64>,
) -> (bool, String) {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (false, format!("spawn error: {}", e)),
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let err_handle = std::thread::spawn(move || {
        let mut lines = Vec::new();
        if let Some(pipe) = stderr {
            let reader = BufReader::new(pipe);
            for line in reader.lines().map_while(Result::ok) {
                let t = line.trim().to_string();
                if !t.is_empty() {
                    lines.push(t);
                    if lines.len() > 10 {
                        lines.remove(0);
                    }
                }
            }
        }
        lines
    });

    if let Some(pipe) = stdout {
        let reader = BufReader::new(pipe);

        for line in reader.lines().map_while(Result::ok) {
            if let Some(rest) = line.strip_prefix("out_time_us=") {
                if let Ok(us) = rest.trim().parse::<u64>() {
                    let current_secs = us as f64 / 1_000_000.0;
                    if let Some(dur) = total_duration {
                        if dur > 0.0 {
                            let pct = ((current_secs / dur) * 100.0).clamp(0.0, 99.0);
                            pb.set_position(pct as u64);
                        }
                    }
                }
            } else if let Some(speed_str) = line.strip_prefix("speed=") {
                let s = speed_str.trim();
                if !s.is_empty() && s != "N/A" {
                    pb.set_message(format!("{} • {}", display_name, s));
                }
            }
        }
    }

    let status = child.wait().map(|s| s.success()).unwrap_or(false);
    let err_lines = err_handle.join().unwrap_or_default();

    let err_summary = if !status {
        err_lines
            .iter()
            .rev()
            .find(|l| {
                l.contains("Error")
                    || l.contains("Invalid")
                    || l.contains("failed")
                    || l.contains("cannot")
            })
            .cloned()
            .unwrap_or_else(|| {
                err_lines
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "unknown error".to_string())
            })
    } else {
        String::new()
    };

    (status, err_summary)
}
