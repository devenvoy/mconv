use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ExtensionStats {
    pub ext: String,
    pub count: usize,
}

pub fn scan_extensions(dir: &Path, recursive: bool) -> Vec<ExtensionStats> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    collect_extensions(dir, recursive, &mut counts);

    let mut stats: Vec<ExtensionStats> = counts
        .into_iter()
        .map(|(ext, count)| ExtensionStats { ext, count })
        .collect();

    stats.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.ext.cmp(&b.ext)));
    stats
}

fn collect_extensions(dir: &Path, recursive: bool, counts: &mut BTreeMap<String, usize>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_ignored(&path) {
            continue;
        }

        if let Ok(ft) = entry.file_type() {
            if ft.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if !ext_str.is_empty() {
                        *counts.entry(ext_str).or_insert(0) += 1;
                    }
                }
            } else if recursive && ft.is_dir() {
                collect_extensions(&path, true, counts);
            }
        }
    }
}

pub fn find_matching_files(dir: &Path, target_ext: &str, recursive: bool) -> Vec<PathBuf> {
    let target_lower = target_ext.to_lowercase();
    let mut matched = Vec::new();
    collect_matching(dir, &target_lower, recursive, &mut matched);
    matched.sort();
    matched
}

fn collect_matching(dir: &Path, target_ext: &str, recursive: bool, matched: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_ignored(&path) {
            continue;
        }

        if let Ok(ft) = entry.file_type() {
            if ft.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == target_ext {
                        matched.push(path);
                    }
                }
            } else if recursive && ft.is_dir() {
                collect_matching(&path, target_ext, true, matched);
            }
        }
    }
}

fn is_ignored(path: &Path) -> bool {
    let lossy = path.to_string_lossy();
    if lossy.contains("/.mconv_tmp") || lossy.contains("/.mconv") {
        return true;
    }
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') && name_str != "." && name_str != ".." {
            return true;
        }
    }
    false
}
