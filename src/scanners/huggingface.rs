use super::dir_size;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HFItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

pub fn scan_hugging_face() -> Vec<HFItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };
    let hf_dir = home.join(".cache/huggingface");

    let hub_dir = hf_dir.join("hub");
    if let Ok(entries) = fs::read_dir(&hub_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && file_name.starts_with("models--") {
                let path = entry.path();
                let size = dir_size(&path);
                let name = parse_hf_model_name(&file_name);
                list.push(HFItem {
                    name: format!("Model: {}", name),
                    path,
                    size,
                });
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&hf_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && file_name != "hub" {
                let path = entry.path();
                let size = dir_size(&path);
                list.push(HFItem {
                    name: format!("Cache: {}", file_name),
                    path,
                    size,
                });
            }
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}

fn parse_hf_model_name(folder_name: &str) -> String {
    if let Some(stripped) = folder_name.strip_prefix("models--") {
        stripped.replace("--", "/")
    } else {
        folder_name.to_string()
    }
}
