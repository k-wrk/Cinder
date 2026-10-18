use super::dir_size;
use std::fs;
use std::process::Command;

pub const SAFE_KEYWORDS: &[&str] = &[
    "apple", "microsoft", "office", "onedrive", "adobe", "docker", "1password",
    "dropbox", "google", "chrome", "spotify", "slack", "telegram", "discord",
    "visualstudio", "vscode", "cursor", "intellij", "github", "brave", "safari",
];

pub fn is_safe_to_clean(folder_name: &str) -> bool {
    let name = folder_name.to_lowercase();
    for word in SAFE_KEYWORDS {
        if name.contains(word) {
            return false;
        }
    }
    true
}

fn check_mdfind_bundle_id(id: &str) -> bool {
    let output = Command::new("mdfind")
        .arg(format!("kMDItemCFBundleIdentifier == {}", id))
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        text.contains(".app")
    } else {
        false
    }
}

pub fn get_orphan_containers_size() -> u64 {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return 0,
    };
    let dirs = [
        home.join("Library/Containers"),
        home.join("Library/Group Containers"),
    ];

    let mut total_size: u64 = 0;
    for base_dir in dirs {
        let entries = match fs::read_dir(&base_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                if !is_safe_to_clean(&folder_name) {
                    continue;
                }

                let mut clean_name = folder_name.clone();
                if let Some(stripped) = clean_name.strip_prefix("group.") {
                    clean_name = stripped.to_string();
                }
                let parts: Vec<&str> = clean_name.splitn(2, '.').collect();
                if parts.len() == 2 && parts[0].len() == 10 {
                    clean_name = parts[1].to_string();
                }

                let found1 = check_mdfind_bundle_id(&folder_name);
                let found2 = check_mdfind_bundle_id(&clean_name);

                if !found1 && !found2 {
                    total_size += dir_size(entry.path());
                }
            }
        }
    }
    total_size
}

pub fn clean_orphan_containers(dry_run: bool) -> (String, std::io::Result<()>) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return ("No home directory".to_string(), Ok(())),
    };
    let dirs = [
        home.join("Library/Containers"),
        home.join("Library/Group Containers"),
    ];

    let mut count = 0;
    for base_dir in dirs {
        let entries = match fs::read_dir(&base_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                if !is_safe_to_clean(&folder_name) {
                    continue;
                }

                let mut clean_name = folder_name.clone();
                if let Some(stripped) = clean_name.strip_prefix("group.") {
                    clean_name = stripped.to_string();
                }
                let parts: Vec<&str> = clean_name.splitn(2, '.').collect();
                if parts.len() == 2 && parts[0].len() == 10 {
                    clean_name = parts[1].to_string();
                }

                let found1 = check_mdfind_bundle_id(&folder_name);
                let found2 = check_mdfind_bundle_id(&clean_name);

                if !found1 && !found2 {
                    let _ = super::move_to_trash(entry.path(), dry_run);
                    count += 1;
                }
            }
        }
    }
    if dry_run {
        (format!("[DRY RUN] Would move {} orphan containers to the Trash", count), Ok(()))
    } else {
        (format!("{} orphan containers moved to the Trash", count), Ok(()))
    }
}
