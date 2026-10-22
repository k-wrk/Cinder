use super::dir_size;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone)]
pub struct DevItem {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
    pub action_type: &'static str,
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
        return ("[DRY RUN] Would run brew cleanup --prune=all and autoremove".to_string(), Ok(()));
    }
    let output = Command::new("brew")
        .args(["cleanup", "-s", "--prune=all"])
        .output();
    let _ = Command::new("brew").args(["autoremove"]).output();
    match output {
        Ok(_) => ("Homebrew cleanup (--prune=all & autoremove) completed".to_string(), Ok(())),
        Err(e) => (format!("Failed to run brew cleanup: {}", e), Err(e.to_string())),
    }
}

fn clean_deno(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would run deno clean".to_string(), Ok(()));
    }
    let output = Command::new("deno").arg("clean").output();
    match output {
        Ok(_) => ("Deno cache cleared".to_string(), Ok(())),
        Err(e) => (format!("Failed to run deno clean: {}", e), Err(e.to_string())),
    }
}

fn clean_pnpm(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would run pnpm store prune".to_string(), Ok(()));
    }
    let output = Command::new("pnpm").args(["store", "prune"]).output();
    match output {
        Ok(_) => ("pnpm store pruned (unreferenced packages removed)".to_string(), Ok(())),
        Err(e) => (format!("Failed to run pnpm store prune: {}", e), Err(e.to_string())),
    }
}

fn clean_npm(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would run npm cache clean --force".to_string(), Ok(()));
    }
    let output = Command::new("npm").args(["cache", "clean", "--force"]).output();
    match output {
        Ok(_) => ("npm cache cleared".to_string(), Ok(())),
        Err(e) => (format!("Failed to run npm cache clean: {}", e), Err(e.to_string())),
    }
}

fn clean_composer(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would run composer clear-cache".to_string(), Ok(()));
    }
    let output = Command::new("composer").arg("clear-cache").output();
    match output {
        Ok(_) => ("Composer cache cleared".to_string(), Ok(())),
        Err(e) => (format!("Failed to run composer clear-cache: {}", e), Err(e.to_string())),
    }
}

pub fn scan_dev_tools() -> Vec<DevItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };

    let dev_paths: Vec<(
        &str,
        &str,
        PathBuf,
        &'static str,
        Option<fn(bool) -> (String, Result<(), String>)>,
    )> = vec![
        ("xcode", "Xcode Derived Data", home.join("Library/Developer/Xcode/DerivedData"), "TRASH", None),
        ("xcode_device_support", "Xcode iOS DeviceSupport Symbols", home.join("Library/Developer/Xcode/iOS DeviceSupport"), "TRASH", None),
        ("xcode_simulators", "Xcode iOS Simulators Cache", home.join("Library/Developer/CoreSimulator/Devices"), "TRASH", None),
        ("xcode_archives", "Xcode Archives", home.join("Library/Developer/Xcode/Archives"), "TRASH", None),
        ("go_mod", "Go Module Cache", home.join("go/pkg/mod"), "TRASH", None),
        ("go_build", "Go Build Cache", home.join("Library/Caches/go-build"), "TRASH", None),
        ("cargo_registry", "Rust Cargo Registry Cache", home.join(".cargo/registry/cache"), "TRASH", None),
        ("cargo_git", "Rust Cargo Git Cache", home.join(".cargo/git/db"), "TRASH", None),
        ("pip", "Python Pip Cache", home.join("Library/Caches/pip"), "TRASH", None),
        ("poetry", "Python Poetry Cache", home.join("Library/Caches/pypoetry"), "TRASH", None),
        ("cypress", "Cypress Browser Binaries Cache", home.join("Library/Caches/Cypress"), "TRASH", None),
        ("jetbrains", "JetBrains IDEs Caches", home.join("Library/Caches/JetBrains"), "TRASH", None),
        ("android_sdk", "Android SDK Cache (AVD/Simulators)", home.join("Library/Android"), "TRASH", None),
        ("android_avds", "Android Virtual Devices (AVD Files)", home.join(".android/avd"), "TRASH", None),
        ("yarn", "Yarn Package Cache (Node.js)", home.join("Library/Caches/Yarn"), "TRASH", None),
        ("npm", "npm Cache", home.join(".npm/_cacache"), if super::has_binary("npm") { "CLI: npm" } else { "TRASH" }, if super::has_binary("npm") { Some(clean_npm) } else { None }),
        ("npm_npx", "npx Temporary Binary Cache", home.join(".npm/_npx"), "TRASH", None),
        ("uv_cache", "Python uv Package Cache", home.join(".cache/uv"), "CLI: uv", Some(clean_uv)),
        ("phpactor", "Phpactor Caches (PHP Tooling)", home.join(".cache/phpactor"), "TRASH", None),
        ("gradle", "Gradle Caches (Java/Android Build)", home.join(".gradle/caches"), "TRASH", None),
        ("gradle_jdks", "Gradle Downloaded JDK Archives", home.join(".gradle/jdks"), "TRASH", None),
        ("cocoapods", "CocoaPods Cache", home.join("Library/Caches/CocoaPods"), "TRASH", None),
        ("duzzy_osrm", "Duzzy Server OSRM Data", home.join("Project/Gokzel/duzzy-server/data"), "TRASH", None),
        ("deno", "Deno Module & Compilation Cache", home.join("Library/Caches/deno"), if super::has_binary("deno") { "CLI: deno" } else { "TRASH" }, if super::has_binary("deno") { Some(clean_deno) } else { None }),
        ("pnpm_store", "pnpm Global Package Store", home.join("Library/pnpm/store"), if super::has_binary("pnpm") { "CLI: pnpm" } else { "TRASH" }, if super::has_binary("pnpm") { Some(clean_pnpm) } else { None }),
        ("pnpm_cache", "pnpm Metadata Cache", home.join("Library/Caches/pnpm"), "TRASH", None),
        ("playwright", "Playwright Browser Binaries", home.join("Library/Caches/ms-playwright"), "TRASH", None),
        ("playwright_go", "Playwright Go Binaries", home.join("Library/Caches/ms-playwright-go"), "TRASH", None),
        ("android_studio_caches", "Android Studio Orphan Caches", home.join("Library/Caches/Google/AndroidStudio2024.3"), "TRASH", None),
        ("konan", "Kotlin/Native Compiler Toolchains & Cache", home.join(".konan"), "TRASH", None),
        ("app_updaters", "Residual Application Updaters Cache", home.join("Library/Caches/antigravity-updater"), "TRASH", None),
        ("composer", "Composer Cache (PHP)", home.join("Library/Caches/composer"), if super::has_binary("composer") { "CLI: composer" } else { "TRASH" }, if super::has_binary("composer") { Some(clean_composer) } else { None }),
        ("brew", "Homebrew Downloads & Orphan Packages", home.join("Library/Caches/Homebrew"), "CLI: brew", Some(clean_brew)),
    ];

    for (id, name, path, action_type, custom_clean) in dev_paths {
        let mut size: u64 = 0;
        if id == "brew" {
            if super::has_binary("brew") {
                if path.exists() {
                    size = dir_size(&path);
                }
                if size == 0 {
                    size = 500 * 1024 * 1024;
                }
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
                selected: false,
                action_type,
                custom_clean,
            });
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_clean_dry_run_modes() {
        // DevTools cleaners must all implement safe dry-run modes returning Ok and "[DRY RUN]"
        let (msg, res) = clean_brew(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("brew cleanup"));

        let (msg, res) = clean_deno(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("deno clean"));

        let (msg, res) = clean_uv(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("uv cache"));

        let (msg, res) = clean_pnpm(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("pnpm store prune"));

        let (msg, res) = clean_npm(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("npm cache clean"));

        let (msg, res) = clean_composer(true);
        assert!(res.is_ok());
        assert!(msg.starts_with("[DRY RUN]"));
        assert!(msg.contains("composer clear-cache"));
    }

    #[test]
    fn test_scan_dev_tools_structure() {
        let tools = scan_dev_tools();
        // Check that any found item has valid fields and size > 0
        for item in &tools {
            assert!(!item.id.is_empty());
            assert!(!item.name.is_empty());
            assert!(item.size > 0);
            assert!(!item.action_type.is_empty());
            assert!(!item.selected);
        }
    }
}
