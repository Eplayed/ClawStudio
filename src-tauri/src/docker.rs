// docker.rs - Docker sandbox management via bollard
// Creates/destroys isolated desktop containers with VNC for Computer Use

use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StatsOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const SANDBOX_IMAGE: &str = "dorowu/ubuntu-desktop-lxde-vnc:focal";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub vnc_port: u16,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStats {
    pub cpu_percent: f64,
    pub memory_used_mb: f64,
    pub memory_limit_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStatus {
    pub available: bool,
    pub version: String,
    pub containers_total: usize,
    pub containers_running: usize,
    pub message: String,
}

pub struct DockerManager {
    client: Arc<Mutex<Option<Docker>>>,
    next_port: Arc<Mutex<u16>>,
}

impl DockerManager {
    pub fn new() -> Self {
        let client = Docker::connect_with_local_defaults().ok();
        Self {
            client: Arc::new(Mutex::new(client)),
            next_port: Arc::new(Mutex::new(6080)),
        }
    }
}

/// Check if Docker is available and return engine info
#[tauri::command]
pub async fn check_docker(
    state: tauri::State<'_, DockerManager>,
) -> Result<DockerStatus, String> {
    let client_lock = state.client.lock().await;
    match &*client_lock {
        Some(docker) => match docker.version().await {
            Ok(ver) => {
                let containers = docker
                    .list_containers(Some(ListContainersOptions::<String> {
                        all: true,
                        ..Default::default()
                    }))
                    .await
                    .unwrap_or_default();

                let running = containers.iter().filter(|c| {
                    c.state.as_deref() == Some("running")
                }).count();

                Ok(DockerStatus {
                    available: true,
                    version: ver.version.unwrap_or_default(),
                    containers_total: containers.len(),
                    containers_running: running,
                    message: "Docker engine connected".into(),
                })
            }
            Err(e) => Ok(DockerStatus {
                available: false,
                version: String::new(),
                containers_total: 0,
                containers_running: 0,
                message: format!("Docker connection failed: {}", e),
            }),
        },
        None => Ok(DockerStatus {
            available: false,
            version: String::new(),
            containers_total: 0,
            containers_running: 0,
            message: "Docker not installed or daemon not running".into(),
        }),
    }
}

/// Create and start a new sandbox container with VNC
#[tauri::command]
pub async fn create_sandbox(
    state: tauri::State<'_, DockerManager>,
    name: String,
) -> Result<SandboxInfo, String> {
    let client_lock = state.client.lock().await;
    let docker = client_lock
        .as_ref()
        .ok_or("Docker not available")?;

    // Pull image if not present
    let mut pull_stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: SANDBOX_IMAGE,
            ..Default::default()
        }),
        None,
        None,
    );
    while let Some(result) = pull_stream.next().await {
        if let Err(e) = result {
            log::warn!("Image pull warning: {}", e);
        }
    }

    // Allocate VNC port
    let mut port = state.next_port.lock().await;
    let vnc_port = *port;
    *port += 1;

    // Container config
    let container_name = format!("clawstudio-{}", name);
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "80/tcp".to_string(),
        Some(vec![bollard::models::PortBinding {
            host_ip: Some("127.0.0.1".to_string()),
            host_port: Some(vnc_port.to_string()),
        }]),
    );

    let config = Config {
        image: Some(SANDBOX_IMAGE.to_string()),
        hostname: Some(container_name.clone()),
        host_config: Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            memory: Some(1024 * 1024 * 1024), // 1GB
            nano_cpus: Some(2_000_000_000),    // 2 CPU cores
            shm_size: Some(512 * 1024 * 1024), // 512MB shared memory
            ..Default::default()
        }),
        ..Default::default()
    };

    let container = docker
        .create_container(
            Some(CreateContainerOptions {
                name: &container_name,
                platform: None,
            }),
            config,
        )
        .await
        .map_err(|e| format!("Failed to create container: {}", e))?;

    docker
        .start_container(&container.id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| format!("Failed to start container: {}", e))?;

    let info = SandboxInfo {
        id: container.id,
        name: container_name,
        image: SANDBOX_IMAGE.to_string(),
        status: "running".into(),
        vnc_port,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    log::info!("Sandbox created: {} on port {}", info.name, vnc_port);
    Ok(info)
}

/// Destroy a sandbox container
#[tauri::command]
pub async fn destroy_sandbox(
    state: tauri::State<'_, DockerManager>,
    container_id: String,
) -> Result<(), String> {
    let client_lock = state.client.lock().await;
    let docker = client_lock.as_ref().ok_or("Docker not available")?;

    docker
        .remove_container(
            &container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| format!("Failed to remove container: {}", e))?;

    log::info!("Sandbox destroyed: {}", container_id);
    Ok(())
}

/// List all ClawStudio sandbox containers
#[tauri::command]
pub async fn list_sandboxes(
    state: tauri::State<'_, DockerManager>,
) -> Result<Vec<SandboxInfo>, String> {
    let client_lock = state.client.lock().await;
    let docker = client_lock.as_ref().ok_or("Docker not available")?;

    let mut filters = HashMap::new();
    filters.insert("name", vec!["clawstudio-"]);

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        }))
        .await
        .map_err(|e| e.to_string())?;

    let sandboxes = containers
        .into_iter()
        .map(|c| {
            let vnc_port = c
                .ports
                .as_ref()
                .and_then(|p| p.first())
                .and_then(|p| p.public_port)
                .unwrap_or(0) as u16;

            SandboxInfo {
                id: c.id.unwrap_or_default(),
                name: c.names.unwrap_or_default().first().cloned().unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                status: c.state.unwrap_or_default(),
                vnc_port,
                created_at: String::new(),
            }
        })
        .collect();

    Ok(sandboxes)
}

/// Get real-time resource stats for a sandbox
#[tauri::command]
pub async fn get_sandbox_stats(
    state: tauri::State<'_, DockerManager>,
    container_id: String,
) -> Result<SandboxStats, String> {
    let client_lock = state.client.lock().await;
    let docker = client_lock.as_ref().ok_or("Docker not available")?;

    let mut stats_stream = docker.stats(
        &container_id,
        Some(StatsOptions {
            stream: false,
            one_shot: true,
        }),
    );

    if let Some(Ok(stats)) = stats_stream.next().await {
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
            - stats.precpu_stats.cpu_usage.total_usage as f64;
        let sys_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        let cpu_count = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
        let cpu_percent = if sys_delta > 0.0 {
            (cpu_delta / sys_delta) * cpu_count * 100.0
        } else {
            0.0
        };

        let mem_usage = stats.memory_stats.usage.unwrap_or(0) as f64 / 1024.0 / 1024.0;
        let mem_limit = stats.memory_stats.limit.unwrap_or(0) as f64 / 1024.0 / 1024.0;

        Ok(SandboxStats {
            cpu_percent,
            memory_used_mb: mem_usage,
            memory_limit_mb: mem_limit,
        })
    } else {
        Err("Failed to read container stats".into())
    }
}
