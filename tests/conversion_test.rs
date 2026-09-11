#![allow(clippy::needless_borrows_for_generic_args)]

use mconv::converter::{run_conversion, ConversionConfig};
use mconv::formats::{get_category, get_targets_for_category, MediaCategory};
use mconv::logger::Logger;
use mconv::scanner::{find_matching_files, scan_extensions};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

fn ensure_test_fixtures(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
    fs::create_dir_all(dir).unwrap();
    let audio1 = dir.join("audio1.wav");
    let audio2 = dir.join("audio2.wav");
    let video1 = dir.join("video1.mp4");
    let img1 = dir.join("img1.png");

    if !audio1.exists() {
        let _ = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=1000:duration=1",
                audio1.to_str().unwrap(),
            ])
            .output();
    }
    if !audio2.exists() {
        let _ = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=800:duration=1",
                audio2.to_str().unwrap(),
            ])
            .output();
    }
    if !video1.exists() {
        let _ = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=320x240:rate=10",
                "-t",
                "1",
                video1.to_str().unwrap(),
            ])
            .output();
    }
    if !img1.exists() {
        let _ = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=320x240",
                "-frames:v",
                "1",
                img1.to_str().unwrap(),
            ])
            .output();
    }
}

#[test]
fn test_categories_and_targets() {
    assert_eq!(get_category("mp4"), MediaCategory::Video);
    assert_eq!(get_category("mkv"), MediaCategory::Video);
    assert_eq!(get_category("wav"), MediaCategory::Audio);
    assert_eq!(get_category("mp3"), MediaCategory::Audio);
    assert_eq!(get_category("png"), MediaCategory::Image);
    assert_eq!(get_category("srt"), MediaCategory::Subtitle);

    let targets = get_targets_for_category(MediaCategory::Video, "mp4");
    assert!(targets.contains(&"webm"));
    assert!(targets.contains(&"mkv"));
    assert!(targets.contains(&"gif"));
    assert!(!targets.contains(&"mp4"));
}

#[test]
fn test_directory_scanning() {
    let test_dir = std::env::temp_dir().join("mconv_test_scan");
    ensure_test_fixtures(&test_dir);

    let stats = scan_extensions(&test_dir, false);

    let wav_stat = stats.iter().find(|s| s.ext == "wav");
    assert!(wav_stat.is_some());
    assert_eq!(wav_stat.unwrap().count, 2);

    let mp4_stat = stats.iter().find(|s| s.ext == "mp4");
    assert!(mp4_stat.is_some());
    assert_eq!(mp4_stat.unwrap().count, 1);

    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_audio_conversion_and_skip_safety() {
    let test_dir = std::env::temp_dir().join("mconv_test_audio");
    ensure_test_fixtures(&test_dir);

    let wav_files = find_matching_files(&test_dir, "wav", false);
    assert_eq!(wav_files.len(), 2);

    let logger = Arc::new(Logger::new());
    let config = ConversionConfig {
        to_ext: "mp3".to_string(),
        category: MediaCategory::Audio,
        delete_originals: false,
        workers: 2,
    };

    let mp3_1 = test_dir.join("audio1.mp3");
    let mp3_2 = test_dir.join("audio2.mp3");

    // Run conversion
    let (converted, skipped, failed, _elapsed) =
        run_conversion(&wav_files, &config, Arc::clone(&logger));
    assert_eq!(converted, 2);
    assert_eq!(skipped, 0);
    assert_eq!(failed, 0);

    assert!(mp3_1.exists(), "audio1.mp3 was not created!");
    assert!(mp3_2.exists(), "audio2.mp3 was not created!");
    assert!(fs::metadata(&mp3_1).unwrap().len() > 0);
    assert!(fs::metadata(&mp3_2).unwrap().len() > 0);

    // Second run: should SKIP both files because targets already exist!
    let (converted2, skipped2, failed2, _elapsed2) =
        run_conversion(&wav_files, &config, Arc::clone(&logger));
    assert_eq!(converted2, 0);
    assert_eq!(skipped2, 2);
    assert_eq!(failed2, 0);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_video_and_image_conversion() {
    let test_dir = std::env::temp_dir().join("mconv_test_video_img");
    ensure_test_fixtures(&test_dir);

    let logger = Arc::new(Logger::new());

    // Test Video to WebM (re-encode)
    let mp4_files = find_matching_files(&test_dir, "mp4", false);
    assert_eq!(mp4_files.len(), 1);

    let video_config = ConversionConfig {
        to_ext: "webm".to_string(),
        category: MediaCategory::Video,
        delete_originals: false,
        workers: 1,
    };

    let webm_target = test_dir.join("video1.webm");
    let (c, s, f, _) = run_conversion(&mp4_files, &video_config, Arc::clone(&logger));
    assert_eq!(c, 1);
    assert_eq!(s, 0);
    assert_eq!(f, 0);
    assert!(webm_target.exists());
    assert!(fs::metadata(&webm_target).unwrap().len() > 0);

    // Test Image PNG to JPG
    let png_files = find_matching_files(&test_dir, "png", false);
    assert_eq!(png_files.len(), 1);

    let img_config = ConversionConfig {
        to_ext: "jpg".to_string(),
        category: MediaCategory::Image,
        delete_originals: false,
        workers: 1,
    };

    let jpg_target = test_dir.join("img1.jpg");
    let (c2, s2, f2, _) = run_conversion(&png_files, &img_config, Arc::clone(&logger));
    assert_eq!(c2, 1);
    assert_eq!(s2, 0);
    assert_eq!(f2, 0);
    assert!(jpg_target.exists());
    assert!(fs::metadata(&jpg_target).unwrap().len() > 0);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_parallel_video_multiprogress() {
    let test_dir = std::env::temp_dir().join("mconv_test_multi");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    // Create 3 video files
    let mut files = vec![];
    for i in 1..=3 {
        let f = test_dir.join(format!("vid_{}.mp4", i));
        let _ = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=320x240:rate=25",
                "-t",
                "2",
                f.to_str().unwrap(),
            ])
            .output();
        files.push(f);
    }

    let logger = Arc::new(Logger::new());
    let config = ConversionConfig {
        to_ext: "mkv".to_string(),
        category: MediaCategory::Video,
        delete_originals: false,
        workers: 3,
    };

    let (c, s, f, elapsed) = run_conversion(&files, &config, Arc::clone(&logger));
    assert_eq!(c, 3, "Expected 3 converted files");
    assert_eq!(s, 0);
    assert_eq!(f, 0);
    assert!(elapsed > 0.0);

    for i in 1..=3 {
        let out = test_dir.join(format!("vid_{}.mkv", i));
        assert!(out.exists());
        assert!(fs::metadata(&out).unwrap().len() > 0);
    }

    let _ = fs::remove_dir_all(&test_dir);
}
