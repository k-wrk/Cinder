use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FontItem {
    pub family: String,
    pub count: usize,
    pub size: u64,
    pub files: Vec<PathBuf>,
    pub selected: bool,
}

pub fn scan_fonts() -> Vec<FontItem> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let fonts_dir = home.join("Library/Fonts");
    if !fonts_dir.exists() {
        return Vec::new();
    }

    let entries = match fs::read_dir(&fonts_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut family_map: HashMap<String, (usize, u64, Vec<PathBuf>)> = HashMap::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".ttf") && !file_name.ends_with(".otf") {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = meta.len();

        let family = extract_font_family(&file_name);
        let entry = family_map.entry(family).or_insert((0, 0, Vec::new()));
        entry.0 += 1;
        entry.1 += size;
        entry.2.push(path);
    }

    let mut list = Vec::new();
    for (family, (count, size, files)) in family_map {
        list.push(FontItem {
            family,
            count,
            size,
            files,
            selected: false,
        });
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}

fn extract_font_family(file_name: &str) -> String {
    let name = file_name
        .trim_end_matches(".ttf")
        .trim_end_matches(".otf");

    // Remove variation suffixes like -Bold, -Regular, -Italic, etc.
    let base = if let Some(idx) = name.find('-') {
        &name[..idx]
    } else {
        name
    };

    // Clean up common nerd font qualifiers to aggregate nicely
    let mut clean = base.to_string();
    for suffix in &[
        "NLNerdFontPropo",
        "NLNerdFontMono",
        "NLNerdFont",
        "NerdFontPropo",
        "NerdFontMono",
        "NerdFont",
    ] {
        if let Some(stripped) = clean.strip_suffix(suffix) {
            clean = stripped.to_string();
            break;
        }
    }

    if clean.is_empty() {
        base.to_string()
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_font_family_nerd_fonts() {
        assert_eq!(
            extract_font_family("IosevkaTermNerdFontPropo-Bold.ttf"),
            "IosevkaTerm"
        );
        assert_eq!(
            extract_font_family("IosevkaTermNerdFont-Regular.otf"),
            "IosevkaTerm"
        );
        assert_eq!(
            extract_font_family("JetBrainsMonoNLNerdFontMono-Italic.ttf"),
            "JetBrainsMono"
        );
        assert_eq!(
            extract_font_family("FiraCodeNerdFont-SemiBold.ttf"),
            "FiraCode"
        );
    }

    #[test]
    fn test_extract_font_family_standard_and_single_word() {
        assert_eq!(extract_font_family("Arial-Bold.ttf"), "Arial");
        assert_eq!(extract_font_family("Monofur.ttf"), "Monofur");
        assert_eq!(extract_font_family("CustomFont-Regular.otf"), "CustomFont");
    }
}
