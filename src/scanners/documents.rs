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
    let doc_dir = home.join("Documents");
    let mut files = Vec::new();

    for entry in WalkDir::new(doc_dir)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden entries
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

    files.sort_by(|a, b| b.size.cmp(&a.size));
    if files.len() > 50 {
        files.truncate(50);
    }
    files
}
