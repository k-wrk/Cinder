use super::dir_size;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone)]
pub struct DevItem {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub custom_clean: Option<fn(bool) -> (String, Result<(), String>)>,
}

fn clean_uv(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would clean uv cache".to_string(), Ok(()));
    }
    let output = Command::new("uv").args(["cache", "clean"]).output();
    match output {
        Ok(_) => ("uv cache cleared".to_string(), Ok(())),
        Err(e) => (format!("Failed to clean uv cache: {}", e), Err(e.to_string())),
    }
}

fn clean_brew(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would run brew cleanup".to_string(), Ok(()));
    }
    let output = Command::new("brew").args(["cleanup", "-s"]).output();
    match output {
        Ok(_) => ("Homebrew cleanup completed".to_string(), Ok(())),
        Err(e) => (format!("Failed to run brew cleanup: {}", e), Err(e.to_string())),
    }
}

pub fn scan_dev_tools() -> Vec<DevItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };

    let dev_paths: Vec<(&str, &str, PathBuf, Option<fn(bool) -> (String, Result<(), String>)>)> = vec![
        ("xcode", "Xcode Derived Data", home.join("Library/Developer/Xcode/DerivedData"), None),
        ("xcode_device_support", "Xcode iOS DeviceSupport Symbols", home.join("Library/Developer/Xcode/iOS DeviceSupport"), None),
        ("xcode_simulators", "Xcode iOS Simulators Cache", home.join("Library/Developer/CoreSimulator/Devices"), None),
        ("xcode_archives", "Xcode Archives", home.join("Library/Developer/Xcode/Archives"), None),
        ("go_mod", "Go Module Cache", home.join("go/pkg/mod"), None),
        ("go_build", "Go Build Cache", home.join("Library/Caches/go-build"), None),
        ("cargo_registry", "Rust Cargo Registry Cache", home.join(".cargo/registry/cache"), None),
        ("cargo_git", "Rust Cargo Git Cache", home.join(".cargo/git/db"), None),
        ("pip", "Python Pip Cache", home.join("Library/Caches/pip"), None),
        ("poetry", "Python Poetry Cache", home.join("Library/Caches/pypoetry"), None),
        ("cypress", "Cypress Browser Binaries Cache", home.join("Library/Caches/Cypress"), None),
        ("jetbrains", "JetBrains IDEs Caches", home.join("Library/Caches/JetBrains"), None),
        ("android_sdk", "Android SDK Cache (AVD/Simulators)", home.join("Library/Android"), None),
        ("android_avds", "Android Virtual Devices (AVD Files)", home.join(".android/avd"), None),
        ("yarn", "Yarn Package Cache (Node.js)", home.join("Library/Caches/Yarn"), None),
        ("npm", "npm Cache", home.join(".npm/_cacache"), None),
        ("uv_cache", "Python uv Package Cache", home.join(".cache/uv"), Some(clean_uv)),
        ("phpactor", "Phpactor Caches (PHP Tooling)", home.join(".cache/phpactor"), None),
        ("gradle", "Gradle Caches (Java/Android Build)", home.join(".gradle/caches"), None),
        ("cocoapods", "CocoaPods Cache", home.join("Library/Caches/CocoaPods"), None),
        ("duzzy_osrm", "Duzzy Server OSRM Data", home.join("Project/Gokzel/duzzy-server/data"), None),
        ("brew", "Homebrew System Cleanup", PathBuf::from("brew_cleanup"), Some(clean_brew)),
    ];

    for (id, name, path, custom_clean) in dev_paths {
        let mut size: u64 = 0;
        if id == "brew" {
            if super::has_binary("brew") {
                size = 200 * 1024 * 1024;
            }
        } else if path.exists() {
            size = dir_size(&path);
        }

        if size > 0 {
            list.push(DevItem {
                id: id.to_string(),
                name: name.to_string(),
                path,
                size,
                custom_clean,
            });
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}
