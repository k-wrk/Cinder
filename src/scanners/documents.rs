use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
}

pub fn scan_documents() -> Vec<FileInfo> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let scan_dirs = vec![
        home.join("Documents"),
        home.join("Downloads"),
        home.join("dumps"),
    ];
    scan_documents_in_dirs(&scan_dirs)
}

pub fn scan_documents_in_dirs(scan_dirs: &[PathBuf]) -> Vec<FileInfo> {
    let mut files = Vec::new();

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                // If it's the root directory of the scan, don't filter it out
                if e.depth() == 0 {
                    return true;
                }
                // Skip hidden files/directories
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
        {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    files.push(FileInfo {
                        path: entry.path().to_path_buf(),
                        size: meta.len(),
                        selected: false,
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| b.size.cmp(&a.size));
    if files.len() > 50 {
        files.truncate(50);
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_scan_documents_skips_hidden_files() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let regular_file = tmp_dir.path().join("report.pdf");
        let hidden_file = tmp_dir.path().join(".DS_Store");
        let hidden_env = tmp_dir.path().join(".env");

        {
            let mut f1 = File::create(&regular_file).unwrap();
            f1.write_all(vec![1u8; 1024].as_slice()).unwrap();

            let mut f2 = File::create(&hidden_file).unwrap();
            f2.write_all(b"hidden").unwrap();

            let mut f3 = File::create(&hidden_env).unwrap();
            f3.write_all(b"SECRET=123").unwrap();
        }

        let results = scan_documents_in_dirs(&[tmp_dir.path().to_path_buf()]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, regular_file);
        assert_eq!(results[0].size, 1024);
        assert!(!results[0].selected);
    }

    #[test]
    fn test_scan_documents_sorts_by_size_descending() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let small = tmp_dir.path().join("small.txt");
        let large = tmp_dir.path().join("large.bin");

        fs::write(&small, vec![0u8; 50]).unwrap();
        fs::write(&large, vec![0u8; 500]).unwrap();

        let results = scan_documents_in_dirs(&[tmp_dir.path().to_path_buf()]);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].path, large);
        assert_eq!(results[0].size, 500);
        assert_eq!(results[1].path, small);
        assert_eq!(results[1].size, 50);
    }
}
