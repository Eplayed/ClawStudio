// gateway.rs - Gateway lifecycle management
// Manages OpenClaw Gateway daemon: start/stop/restart/health/logs

use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;
use std::sync::Arc;
use tokio::sync::Mutex;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayHealth {
    pub running: bool,
    pub port: u16,
    pub version: String,
    pub uptime_sec: u64,
    pub connected_clients: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub version: String,
    pub config_path: String,
    pub log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

// ─── Gateway State ───

pub struct GatewayState {
    pub process: Arc<Mutex<Option<std::process::Child>>>,
    pub port: Arc<Mutex<u16>>,
}

impl GatewayState {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(18789)),
        }
    }
}

impl Default for GatewayState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tauri Commands ───

/// Start the Gateway daemon
#[tauri::command]
pub async fn start_gateway(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, GatewayState>,
    proxy_state: tauri::State<'_, crate::proxy::ProxyServerState>,
    port: Option<u16>,
) -> Result<GatewayHealth, String> {
    let port = port.unwrap_or(18789);

    // Check if already running
    if let Ok(health) = check_health(port).await {
        if health.running {
            // Also ensure proxy is running
            let _ = crate::proxy::start_proxy(app_handle, proxy_state, Some(18788), Some(100.0), Some(true)).await;
            return Ok(health);
        }
    }

    log::info!("Starting Gateway on port {}...", port);

    // Start gateway as background process
    #[cfg(unix)]
    let child = StdCommand::new("openclaw")
        .args(["gateway", "start", "--port", &port.to_string()])
        .spawn()
        .map_err(|e| format!("Failed to start gateway: {}", e))?;

    #[cfg(windows)]
    let child = StdCommand::new("cmd")
        .args(["/C", "openclaw", "gateway", "start", "--port", &port.to_string()])
        .spawn()
        .map_err(|e| format!("Failed to start gateway: {}", e))?;

    // Store process handle
    *state.process.lock().await = Some(child);
    *state.port.lock().await = port;
    
    // Start proxy server alongside gateway
    let _ = crate::proxy::start_proxy(app_handle, proxy_state, Some(18788), Some(100.0), Some(true)).await;

    // Wait for gateway to be ready
    for i in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Ok(health) = check_health(port).await {
            if health.running {
                log::info!("Gateway started successfully on port {}", port);
                return Ok(health);
            }
        }

        log::debug!("Waiting for gateway to start... ({}/30)", i + 1);
    }

    Err("Gateway failed to start within 15 seconds".to_string())
}

/// Stop the Gateway daemon
#[tauri::command]
pub async fn stop_gateway(
    state: tauri::State<'_, GatewayState>,
    proxy_state: tauri::State<'_, crate::proxy::ProxyServerState>,
) -> Result<(), String> {
    log::info!("Stopping Gateway...");

    // Stop proxy server
    let _ = crate::proxy::stop_proxy(proxy_state).await;

    // Kill stored process if any
    if let Some(mut child) = state.process.lock().await.take() {
        let _ = child.kill();
    }

    // Also run the CLI command to ensure complete stop
    let output = StdCommand::new("openclaw")
        .args(["gateway", "stop"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            log::info!("Gateway stopped successfully");
            Ok(())
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            log::warn!("Gateway stop returned: {}", stderr);
            Ok(()) // Still consider it stopped
        }
        Err(e) => {
            log::error!("Failed to stop gateway: {}", e);
            Err(format!("Failed to stop gateway: {}", e))
        }
    }
}

/// Restart the Gateway daemon
#[tauri::command]
pub async fn restart_gateway(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, GatewayState>,
    proxy_state: tauri::State<'_, crate::proxy::ProxyServerState>,
    port: Option<u16>,
) -> Result<GatewayHealth, String> {
    log::info!("Restarting Gateway...");

    stop_gateway(state.clone(), proxy_state.clone()).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    start_gateway(app_handle, state, proxy_state, port).await
}

/// Check Gateway health via HTTP
#[tauri::command]
pub async fn gateway_health(port: Option<u16>) -> Result<GatewayHealth, String> {
    let port = port.unwrap_or(18789);
    check_health(port).await
}

async fn check_health(port: u16) -> Result<GatewayHealth, String> {
    let url = format!("http://127.0.0.1:{}/healthz", port);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();

            // Parse health response
            Ok(GatewayHealth {
                running: true,
                port,
                version: extract_version(&body).unwrap_or_default(),
                uptime_sec: extract_uptime(&body).unwrap_or(0),
                connected_clients: 0,
            })
        }
        Ok(resp) => {
            Err(format!("Gateway unhealthy: HTTP {}", resp.status()))
        }
        Err(_) => {
            Ok(GatewayHealth {
                running: false,
                port,
                version: String::new(),
                uptime_sec: 0,
                connected_clients: 0,
            })
        }
    }
}

/// Get recent Gateway logs
#[tauri::command]
pub async fn gateway_logs(tail: Option<usize>) -> Result<Vec<String>, String> {
    let tail = tail.unwrap_or(200);

    // Try to read from log file
    if let Some(home) = dirs::home_dir() {
        let log_path = home.join(".openclaw").join("logs").join("gateway.log");

        if log_path.exists() {
            let content = std::fs::read_to_string(&log_path)
                .map_err(|e| format!("Failed to read log: {}", e))?;

            let lines: Vec<String> = content
                .lines()
                .rev()
                .take(tail)
                .map(String::from)
                .collect();

            return Ok(lines.into_iter().rev().collect());
        }
    }

    // Fallback: try CLI command
    let output = StdCommand::new("openclaw")
        .args(["gateway", "logs", "--tail", &tail.to_string()])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let logs = String::from_utf8_lossy(&o.stdout);
            Ok(logs.lines().map(String::from).collect())
        }
        _ => Err("Failed to get logs".to_string()),
    }
}

/// Get OpenClaw version
#[tauri::command]
pub fn get_openclaw_version() -> Result<String, String> {
    let output = StdCommand::new("openclaw")
        .arg("--version")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        Ok(_) => Err("OpenClaw not installed".to_string()),
        Err(_) => Err("OpenClaw command not found".to_string()),
    }
}

/// Get Gateway info
#[tauri::command]
pub fn get_gateway_info() -> Result<GatewayInfo, String> {
    let version = get_openclaw_version()?;

    let config_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let log_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("logs"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(GatewayInfo {
        version,
        config_path,
        log_path,
    })
}

/// Check for OpenClaw updates
#[tauri::command]
pub async fn check_openclaw_update() -> Result<UpdateInfo, String> {
    // Get current version
    let current = get_openclaw_version()?;

    // Get latest version from npm
    let output = StdCommand::new("npm")
        .args(["view", "openclaw", "version"])
        .output();

    let latest = match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => {
            // Try with registry
            let output = StdCommand::new("npm")
                .args(["view", "openclaw", "version", "--registry=https://registry.npmjs.org"])
                .output();

            String::from_utf8_lossy(&output.ok().unwrap().stdout).trim().to_string()
        }
    };

    let update_available = current != latest && !latest.is_empty();

    Ok(UpdateInfo {
        current_version: current,
        latest_version: latest,
        update_available,
    })
}

/// Upgrade OpenClaw
#[tauri::command]
pub async fn upgrade_openclaw(
    _app: tauri::AppHandle,
    use_mirror: bool,
) -> Result<(), String> {
    let args = if use_mirror {
        vec!["update", "-g", "openclaw", "--registry=https://registry.npmmirror.com"]
    } else {
        vec!["update", "-g", "openclaw"]
    };

    let output = StdCommand::new("npm").args(&args).output();

    match output {
        Ok(o) if o.status.success() => {
            log::info!("OpenClaw upgraded successfully");
            Ok(())
        }
        Ok(o) => Err(format!("Upgrade failed: {}", String::from_utf8_lossy(&o.stderr))),
        Err(e) => Err(format!("Failed to run npm update: {}", e)),
    }
}

/// Get OpenClaw configuration
#[tauri::command]
pub fn get_openclaw_config() -> Result<serde_json::Value, String> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .ok_or("Cannot determine home directory")?;

    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Set OpenClaw configuration
#[tauri::command]
pub fn set_openclaw_config(config: serde_json::Value) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .ok_or("Cannot determine home directory")?;

    // Create directory if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    log::info!("OpenClaw config updated");
    Ok(())
}

// ─── Helper Functions ───

fn extract_version(body: &str) -> Option<String> {
    // Try to parse JSON response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        json.get("version")?.as_str().map(String::from)
    } else {
        // Try regex for plain text
        let re = regex::Regex::new(r"version[:\s]+([^\s]+)").ok()?;
        let caps = re.captures(body)?;
        Some(caps.get(1)?.as_str().to_string())
    }
}

fn extract_uptime(body: &str) -> Option<u64> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        json.get("uptime_sec")?.as_u64()
    } else {
        let re = regex::Regex::new(r"uptime[:\s]+(\d+)").ok()?;
        let caps = re.captures(body)?;
        caps.get(1)?.as_str().parse().ok()
    }
}

// ─── Additional Tauri Commands ───

/// Get Gateway status (alias for gateway_health with more details)
#[tauri::command]
pub async fn gateway_status(port: Option<u16>) -> Result<serde_json::Value, String> {
    let health = gateway_health(port).await?;
    let info = get_gateway_info().ok();
    
    Ok(serde_json::json!({
        "health": health,
        "info": info,
    }))
}

/// Cleanup Gateway process on app exit
pub fn cleanup_on_exit(state: &GatewayState) {
    // Try to stop the gateway
    let output = std::process::Command::new("openclaw")
        .args(["gateway", "stop"])
        .output();
    
    if let Ok(o) = output {
        if o.status.success() {
            log::info!("Gateway stopped on exit");
        }
    }
}
