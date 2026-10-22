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
pub mod fonts;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024 * 3), "3.00 TB");
    }

    #[test]
    fn test_has_binary() {
        // macOS standard system utilities must always exist
        assert!(has_binary("sh"));
        assert!(has_binary("ls"));
        // Random non-existent command must return false
        assert!(!has_binary("cinder_non_existent_binary_xyz_123"));
    }

    #[test]
    fn test_move_to_trash_dry_run_preserves_file() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("safe_test_file.txt");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"Important data that must never be deleted in dry run").unwrap();
        }

        assert!(file_path.exists());
        let res = move_to_trash(&file_path, true);
        assert!(res.is_ok());
        // In dry run, file must strictly remain on disk
        assert!(file_path.exists());
    }

    #[test]
    fn test_move_to_trash_dry_run_preserves_dir() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let sub_dir = tmp_dir.path().join("sub_folder");
        fs::create_dir(&sub_dir).unwrap();
        let inner_file = sub_dir.join("inner.txt");
        fs::write(&inner_file, b"content").unwrap();

        assert!(sub_dir.exists());
        let res = move_to_trash(&sub_dir, true);
        assert!(res.is_ok());
        assert!(sub_dir.exists());
        assert!(inner_file.exists());
    }

    #[test]
    fn test_move_to_trash_non_existent_path() {
        let path = Path::new("/path/that/definitely/does/not/exist/for/cinder_test");
        let res = move_to_trash(path, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_dir_size() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let sub_dir = tmp_dir.path().join("nested");
        fs::create_dir(&sub_dir).unwrap();

        let file1 = tmp_dir.path().join("file1.bin");
        let file2 = sub_dir.join("file2.bin");

        // Write 100 bytes to file1 and 200 bytes to file2
        fs::write(&file1, vec![0u8; 100]).unwrap();
        fs::write(&file2, vec![0u8; 200]).unwrap();

        let size = dir_size(tmp_dir.path());
        assert!(size >= 300);
    }
}
