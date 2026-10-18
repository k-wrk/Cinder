use super::dir_size;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppSupportItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub orphan: bool,
    pub selected: bool,
}

pub fn scan_app_support() -> Vec<AppSupportItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };
    let app_support_dir = home.join("Library/Application Support");

    let system_folders = [
        "Apple", "com.apple.TCC", "SyncServices", "AddressBook", "Quick Look",
        "AppStore", "MobileSync", "iCloud", "com.apple.sharedfilelist",
        "com.apple.spotlight", "com.apple.touristd", "NotificationCenter", "Dock",
        "OpenDirectory", "DiskImages", "CrashReporter", "Accounts", "CloudDocs",
        "CoreParsec", "SearchParty", "SpamSieve", "Adobe", "Microsoft", "Oracle",
        "Helper", "com.apple.spotlight-server",
    ];

    if let Ok(entries) = fs::read_dir(&app_support_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if system_folders.contains(&name.as_str()) || name.starts_with("com.apple.") {
                    continue;
                }

                let path = entry.path();
                let size = dir_size(&path);
                if size > 10 * 1024 * 1024 {
                    // Only show dirs larger than 10MB
                    let installed = is_app_installed(&name);
                    list.push(AppSupportItem {
                        name,
                        path,
                        size,
                        orphan: !installed,
                        selected: false,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| {
        if a.orphan && !b.orphan {
            std::cmp::Ordering::Less
        } else if !a.orphan && b.orphan {
            std::cmp::Ordering::Greater
        } else {
            b.size.cmp(&a.size)
        }
    });

    list
}

pub fn is_app_installed(folder_name: &str) -> bool {
    let mut clean_name = folder_name.to_lowercase();
    if clean_name.contains("zoom") {
        clean_name = "zoom".to_string();
    } else if clean_name.contains("stremio") {
        clean_name = "stremio".to_string();
    } else if clean_name.contains("postman") {
        clean_name = "postman".to_string();
    } else if clean_name.contains("steam") {
        clean_name = "steam".to_string();
    } else if clean_name.contains("discord") {
        clean_name = "discord".to_string();
    } else if clean_name.contains("slack") {
        clean_name = "slack".to_string();
    }

    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Applications"));
    }

    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let app_name = entry.file_name().to_string_lossy().to_lowercase();
                if app_name.contains(&clean_name) && app_name.ends_with(".app") {
                    return true;
                }
            }
        }
    }

    false
}
