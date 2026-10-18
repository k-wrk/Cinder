use super::dir_size;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BrowserItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub selected: bool,
}

pub fn scan_browsers() -> Vec<BrowserItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };

    let browser_caches = [
        ("Google Chrome Cache", home.join("Library/Caches/Google/Chrome")),
        ("Google Chrome AppSupport Cache", home.join("Library/Application Support/Google/Chrome/Default/Cache")),
        ("Arc Browser Cache", home.join("Library/Caches/company.thebrowser.Browser")),
        ("Arc Browser Sparkle Cache", home.join("Library/Caches/company.thebrowser.Browser.Sparkle")),
        ("Brave Browser Cache", home.join("Library/Caches/BraveSoftware/Brave-Browser")),
        ("Brave Browser Support Cache", home.join("Library/Application Support/BraveSoftware/Brave-Browser/Default/Cache")),
        ("Safari Cache", home.join("Library/Caches/com.apple.Safari")),
        ("Mozilla Firefox Cache", home.join("Library/Caches/Firefox")),
        ("Mozilla General Cache", home.join("Library/Caches/Mozilla")),
        ("Zen Browser Cache", home.join("Library/Caches/Zen")),
        ("DuckDuckGo Browser Cache", home.join("Library/Caches/com.duckduckgo.macos.browser")),
    ];

    for (name, path) in browser_caches {
        if path.exists() {
            let size = dir_size(&path);
            if size > 0 {
                list.push(BrowserItem {
                    name: name.to_string(),
                    path,
                    size,
                    selected: false,
                });
            }
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}
