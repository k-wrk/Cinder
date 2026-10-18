use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct OllamaItem {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Deserialize)]
struct OllamaManifest {
    config: Option<ManifestConfig>,
    #[serde(default)]
    layers: Vec<ManifestLayer>,
}

#[derive(Deserialize)]
struct ManifestConfig {
    #[serde(default)]
    size: u64,
    #[serde(default)]
    digest: String,
}

#[derive(Deserialize)]
struct ManifestLayer {
    #[serde(default)]
    size: u64,
    #[serde(default)]
    digest: String,
}

pub fn scan_ollama() -> Vec<OllamaItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };
    let manifests_dir = home.join(".ollama/models/manifests/registry.ollama.ai");

    if !manifests_dir.exists() {
        return list;
    }

    for entry in WalkDir::new(&manifests_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&manifests_dir) {
                let parts: Vec<&str> = rel.iter().filter_map(|p| p.to_str()).collect();
                let mut name = rel.to_string_lossy().to_string();

                if parts.len() >= 2 {
                    let model_name = parts[parts.len() - 2];
                    let tag_name = parts[parts.len() - 1];
                    if parts.len() == 3 && parts[0] == "library" {
                        name = format!("{}:{}", model_name, tag_name);
                    } else {
                        let prefix = parts[..parts.len() - 1].join("/");
                        name = format!("{}:{}", prefix, tag_name);
                    }
                }

                if let Ok(data) = fs::read(path) {
                    if let Ok(manifest) = serde_json::from_slice::<OllamaManifest>(&data) {
                        let mut total_size = manifest.config.as_ref().map(|c| c.size).unwrap_or(0);
                        for layer in &manifest.layers {
                            total_size += layer.size;
                        }

                        list.push(OllamaItem {
                            name,
                            path: path.to_path_buf(),
                            size: total_size,
                        });
                    }
                }
            }
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}

pub fn delete_ollama_model<P: AsRef<Path>>(manifest_path: P, dry_run: bool) -> std::io::Result<()> {
    if dry_run {
        return Ok(());
    }
    let path = manifest_path.as_ref();
    let data = fs::read(path)?;
    let manifest: OllamaManifest = serde_json::from_slice(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let home = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No home dir"))?;
    let blobs_dir = home.join(".ollama/models/blobs");

    // Move layers blobs
    for layer in &manifest.layers {
        if !layer.digest.is_empty() {
            let blob_file = blobs_dir.join(layer.digest.replace(':', "-"));
            if blob_file.exists() {
                let _ = super::move_to_trash(&blob_file, dry_run);
            }
        }
    }

    // Move config blob
    if let Some(config) = manifest.config {
        if !config.digest.is_empty() {
            let blob_file = blobs_dir.join(config.digest.replace(':', "-"));
            if blob_file.exists() {
                let _ = super::move_to_trash(&blob_file, dry_run);
            }
        }
    }

    // Move manifest itself
    let _ = super::move_to_trash(path, dry_run);
    Ok(())
}
