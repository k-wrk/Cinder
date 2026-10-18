use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;

/// Checks if a binary command is executable and present in the user's PATH
pub fn has_binary(bin_name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full_path = dir.join(bin_name);
            if full_path.is_file() {
                return true;
            }
        }
    }
    // Check common macOS locations (Homebrew / Apple Silicon / Intel)
    let common_paths = [
        format!("/opt/homebrew/bin/{}", bin_name),
        format!("/usr/local/bin/{}", bin_name),
        format!("/usr/bin/{}", bin_name),
        format!("/bin/{}", bin_name),
    ];
    for p in &common_paths {
        if Path::new(p).is_file() {
            return true;
        }
    }
    false
}

pub mod apps;
pub mod appsupport;
pub mod browsers;
pub mod containers;
pub mod devtools;
pub mod docker;
pub mod documents;
pub mod huggingface;
pub mod ollama;

/// Recursively calculates directory size (considering sparse files via allocated blocks)
pub fn dir_size<P: AsRef<Path>>(path: P) -> u64 {
    let mut total_size: u64 = 0;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                // Blocks * 512 gives actual disk allocated size (similar to Go implementation)
                let blocks = metadata.blocks();
                if blocks > 0 {
                    total_size += blocks * 512;
                } else {
                    total_size += metadata.len();
                }
            }
        }
    }
    total_size
}

/// Formats bytes into human-readable representation (B, KB, MB, GB, TB)
pub fn format_size(bytes: u64) -> String {
    const UNIT: f64 = 1024.0;
    let b = bytes as f64;
    if b < UNIT {
        return format!("{} B", bytes);
    }
    let units = ["KB", "MB", "GB", "TB"];
    let mut val = b / UNIT;
    let mut idx = 0;
    while val >= UNIT && idx < units.len() - 1 {
        val /= UNIT;
        idx += 1;
    }
    format!("{:.2} {}", val, units[idx])
}

/// Helper function to move a file or directory to macOS Trash using trash crate,
/// with fallback to moving into ~/.Trash.
pub fn move_to_trash<P: AsRef<Path>>(path: P, dry_run: bool) -> std::io::Result<()> {
    let p = path.as_ref();
    if !p.exists() {
        return Ok(());
    }
    if dry_run {
        // In dry-run mode, simulate success without touching the filesystem
        return Ok(());
    }
    if let Err(trash_err) = trash::delete(p) {
        // Fallback: move into ~/.Trash
        if let Some(home) = dirs::home_dir() {
            let trash_dir = home.join(".Trash");
            if let Some(file_name) = p.file_name() {
                let dest = trash_dir.join(file_name);
                if dest.exists() {
                    let _ = if dest.is_dir() {
                        fs::remove_dir_all(&dest)
                    } else {
                        fs::remove_file(&dest)
                    };
                }
                fs::rename(p, dest)?;
                return Ok(());
            }
        }
        return Err(std::io::Error::other(
            format!("Failed to trash item: {}", trash_err),
        ));
    }
    Ok(())
}
