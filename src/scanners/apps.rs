use super::dir_size;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AppSuggestion {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub last_used: Option<DateTime<Utc>>,
    pub never_used: bool,
}

pub fn scan_apps() -> Vec<AppSuggestion> {
    let mut list = Vec::new();
    let mut dirs = vec![PathBuf::from("/Applications")];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Applications"));
    }

    let now = Utc::now();

    for dir in dirs {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.ends_with(".app") {
                let name_lower = file_name.to_lowercase();
                if name_lower.contains("safari")
                    || name_lower.contains("system settings")
                    || name_lower.contains("finder")
                {
                    continue;
                }

                let app_path = entry.path();
                let size = dir_size(&app_path);

                let (last_used, never_used) = get_app_last_used(&app_path);

                let is_unused = if never_used {
                    true
                } else if let Some(dt) = last_used {
                    (now - dt).num_days() > 30
                } else {
                    true
                };

                // Only suggest if larger than 50MB and unused for > 30 days or never used
                if size > 50 * 1024 * 1024 && is_unused {
                    let clean_name = file_name.strip_suffix(".app").unwrap_or(&file_name).to_string();
                    list.push(AppSuggestion {
                        name: clean_name,
                        path: app_path,
                        size,
                        last_used,
                        never_used,
                    });
                }
            }
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}

fn get_app_last_used(app_path: &Path) -> (Option<DateTime<Utc>>, bool) {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemLastUsedDate", "-raw"])
        .arg(app_path)
        .output();

    let raw = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => return (None, true),
    };

    if raw.is_empty() || raw == "(null)" || raw.contains("could not find") {
        return (None, true);
    }

    // Try parsing ISO format "2006-01-02 15:04:05 +0000"
    if let Ok(dt) = DateTime::parse_from_str(&raw, "%Y-%m-%d %H:%M:%S %z") {
        return (Some(dt.with_timezone(&Utc)), false);
    }

    // Fallback: parse date only
    if let Some(date_part) = raw.split_whitespace().next() {
        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
            if let Some(naive_dt) = naive_date.and_hms_opt(0, 0, 0) {
                let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
                return (Some(dt), false);
            }
        }
    }

    (None, true)
}
