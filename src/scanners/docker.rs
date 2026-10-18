use super::dir_size;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone)]
pub struct DockerItem {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub custom_clean: Option<fn(bool) -> (String, Result<(), String>)>,
}

fn clean_docker_system(dry_run: bool) -> (String, Result<(), String>) {
    if dry_run {
        return ("[DRY RUN] Would prune docker system".to_string(), Ok(()));
    }
    let output = Command::new("docker")
        .args(["system", "prune", "-a", "--volumes", "-f"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                ("Docker pruned successfully".to_string(), Ok(()))
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                (format!("Docker prune failed: {}", err_msg), Err(err_msg))
            }
        }
        Err(e) => (format!("Failed to execute docker: {}", e), Err(e.to_string())),
    }
}

pub fn scan_docker() -> Vec<DockerItem> {
    let mut list = Vec::new();
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return list,
    };

    // 1. Docker system prune option
    let docker_installed = super::has_binary("docker");

    if docker_installed {
        list.push(DockerItem {
            id: "docker_prune".to_string(),
            name: "Docker System Prune (Images, Containers, Volumes)".to_string(),
            path: PathBuf::from("docker_prune"),
            size: 500 * 1024 * 1024, // Mock 500MB
            custom_clean: Some(clean_docker_system),
        });
    }

    // 2. VM storage files
    let vm_paths = [
        home.join("Library/Containers/com.docker.docker/Data/vms/0/data/ActiveOS.raw"),
        home.join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"),
        home.join("Library/Containers/com.docker.docker/Data/vms/0/Docker.raw"),
        home.join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.qcow2"),
    ];

    for path in vm_paths {
        if path.exists() {
            let size = dir_size(&path);
            if size > 0 {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                list.push(DockerItem {
                    id: "vm_storage".to_string(),
                    name: format!("Docker VM Disk Image: {}", file_name),
                    path,
                    size,
                    custom_clean: None,
                });
            }
        }
    }

    // 3. ~/.docker configuration directory
    let docker_config_dir = home.join(".docker");
    if docker_config_dir.exists() {
        let size = dir_size(&docker_config_dir);
        if size > 0 {
            list.push(DockerItem {
                id: "config".to_string(),
                name: "Docker Config Directory (~/.docker)".to_string(),
                path: docker_config_dir,
                size,
                custom_clean: None,
            });
        }
    }

    // 4. Docker Containers folder
    let container_dir = home.join("Library/Containers/com.docker.docker");
    if container_dir.exists() {
        let size = dir_size(&container_dir);
        if size > 10 * 1024 * 1024 {
            list.push(DockerItem {
                id: "app_containers".to_string(),
                name: "Docker Desktop Containers Cache".to_string(),
                path: container_dir,
                size,
                custom_clean: None,
            });
        }
    }

    list.sort_by(|a, b| b.size.cmp(&a.size));
    list
}
