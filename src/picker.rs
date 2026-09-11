use std::path::{Path, PathBuf};
use std::process::Command;

pub fn pick_folder_native() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let script =
            r#"POSIX path of (choose folder with prompt "Select media folder for mconv:")"#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let p = PathBuf::from(path_str);
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("zenity")
            .args(&[
                "--file-selection",
                "--directory",
                "--title=Select media folder for mconv",
            ])
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let p = PathBuf::from(path_str);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }

        if let Ok(output) = Command::new("kdialog")
            .args(&["--getexistingdirectory", "."])
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let p = PathBuf::from(path_str);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"[System.Reflection.Assembly]::LoadWithPartialName("System.windows.forms") | Out-Null; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = "Select media folder for mconv"; if ($d.ShowDialog() -eq "OK") { Write-Output $d.SelectedPath }"#;
        let output = Command::new("powershell")
            .args(&["-NoProfile", "-Command", script])
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let p = PathBuf::from(path_str);
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

pub fn list_subdirectories(parent: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.') && name != ".mconv_tmp" && name != ".mconv" {
                        dirs.push(entry.path());
                    }
                }
            }
        }
    }
    dirs.sort();
    dirs
}
