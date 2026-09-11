use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaCategory {
    Video,
    Audio,
    Image,
    Subtitle,
    Unknown,
}

impl fmt::Display for MediaCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaCategory::Video => write!(f, "video"),
            MediaCategory::Audio => write!(f, "audio"),
            MediaCategory::Image => write!(f, "image"),
            MediaCategory::Subtitle => write!(f, "subtitle"),
            MediaCategory::Unknown => write!(f, "unknown"),
        }
    }
}

pub const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "mpg", "mpeg", "m4v", "ts", "3gp",
];
pub const AUDIO_EXTS: &[&str] = &[
    "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "aiff",
];
pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];
pub const SUBTITLE_EXTS: &[&str] = &["srt", "vtt", "ass", "ssa", "sub"];

pub const VIDEO_TARGETS: &[&str] = &["mp4", "mkv", "webm", "avi", "mov", "flv", "gif"];
pub const AUDIO_TARGETS: &[&str] = &["mp3", "aac", "m4a", "flac", "wav", "ogg", "opus", "wma"];
pub const IMAGE_TARGETS: &[&str] = &["png", "jpg", "webp", "bmp", "gif", "tiff"];
pub const SUBTITLE_TARGETS: &[&str] = &["srt", "vtt", "ass"];

pub fn get_category(ext: &str) -> MediaCategory {
    let lower = ext.to_lowercase();
    let ext_str = lower.as_str();

    if VIDEO_EXTS.contains(&ext_str) {
        MediaCategory::Video
    } else if AUDIO_EXTS.contains(&ext_str) {
        MediaCategory::Audio
    } else if IMAGE_EXTS.contains(&ext_str) {
        MediaCategory::Image
    } else if SUBTITLE_EXTS.contains(&ext_str) {
        MediaCategory::Subtitle
    } else {
        MediaCategory::Unknown
    }
}

pub fn get_targets_for_category(category: MediaCategory, from_ext: &str) -> Vec<&'static str> {
    let from_lower = from_ext.to_lowercase();
    let targets = match category {
        MediaCategory::Video => VIDEO_TARGETS,
        MediaCategory::Audio => AUDIO_TARGETS,
        MediaCategory::Image => IMAGE_TARGETS,
        MediaCategory::Subtitle => SUBTITLE_TARGETS,
        MediaCategory::Unknown => &[],
    };

    targets
        .iter()
        .copied()
        .filter(|&t| t != from_lower.as_str())
        .collect()
}

pub fn get_encode_args(category: MediaCategory, to_ext: &str) -> Vec<String> {
    match category {
        MediaCategory::Video => match to_ext {
            "mp4" => vec![
                "-c:v",
                "libx264",
                "-crf",
                "22",
                "-preset",
                "fast",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-movflags",
                "+faststart",
            ],
            "mkv" => vec![
                "-c:v", "libx264", "-crf", "22", "-preset", "fast", "-c:a", "aac", "-b:a", "192k",
            ],
            "webm" => vec![
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
            ],
            "avi" => vec![
                "-c:v",
                "mpeg4",
                "-qscale:v",
                "5",
                "-c:a",
                "libmp3lame",
                "-qscale:a",
                "4",
            ],
            "mov" => vec![
                "-c:v", "libx264", "-crf", "22", "-preset", "fast", "-c:a", "aac", "-b:a", "192k",
            ],
            "flv" => vec![
                "-c:v", "libx264", "-crf", "23", "-preset", "fast", "-c:a", "aac", "-b:a", "128k",
            ],
            "gif" => vec!["-vf", "fps=10,scale=480:-1:flags=lanczos"],
            _ => vec!["-c:v", "libx264", "-c:a", "aac"],
        },
        MediaCategory::Audio => match to_ext {
            "mp3" => vec!["-c:a", "libmp3lame", "-q:a", "2"],
            "aac" | "m4a" => vec!["-c:a", "aac", "-b:a", "192k"],
            "flac" => vec!["-c:a", "flac"],
            "wav" => vec!["-c:a", "pcm_s16le"],
            "ogg" => vec!["-c:a", "libvorbis", "-q:a", "5"],
            "opus" => vec!["-c:a", "libopus", "-b:a", "128k"],
            "wma" => vec!["-c:a", "wmav2", "-b:a", "192k"],
            _ => vec!["-c:a", "aac"],
        },
        MediaCategory::Image => match to_ext {
            "jpg" | "jpeg" => vec!["-q:v", "2"],
            "webp" => vec!["-q:v", "80"],
            _ => vec![],
        },
        MediaCategory::Subtitle => vec![],
        MediaCategory::Unknown => vec![],
    }
    .into_iter()
    .map(String::from)
    .collect()
}
