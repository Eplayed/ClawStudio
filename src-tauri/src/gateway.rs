// gateway.rs - OpenClaw Gateway Daemon Manager
// Implements Phase 2 of ClawStudio v2.0 roadmap
// Manages Gateway lifecycle with proper cleanup on app exit

use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayHealth {
    pub healthy: bool,
    pub response_time_ms: u64,
    pub active_connections: Option<u32>,
}

// ─── Gateway State ───

pub struct GatewayState {
    pub process: Arc<Mutex<Option<Child>>>,
    pub port: Arc<Mutex<u16>>,
    pub start_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl GatewayState {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(18789)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub async fn start_gateway(
    port: u16,
    state: tauri::State<'_, GatewayState>,
) -> Result<GatewayStatus, String> {
    log::info!("Starting OpenClaw Gateway on port {}", port);
    
    let mut process_guard = state.process.lock().await;
    let mut port_guard = state.port.lock().await;
    let mut start_time_guard = state.start_time.lock().await;
    
    // Check if already running
    if process_guard.is_some() {
        return Err("Gateway is already running".to_string());
    }
    
    // Check if port is available
    if is_port_in_use(port).await {
        return Err(format!("Port {} is already in use", port));
    }
    
    // Start the Gateway process
    let child = Command::new("openclaw")
        .args(["gateway", "start", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Gateway: {}", e))?;
    
    let pid = child.id();
    *process_guard = Some(child);
    *port_guard = port;
    *start_time_guard = Some(std::time::Instant::now());
    
    log::info!("Gateway started with PID {}", pid);
    
    // Wait for health check to pass
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    for _ in 0..10 {
        if check_health(port).await?.healthy {
            log::info!("Gateway is healthy on port {}", port);
            return Ok(GatewayStatus {
                running: true,
                port,
                pid: Some(pid),
                uptime_secs: Some(0),
                version: get_openclaw_version().await.ok(),
            });
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    Err("Gateway started but health check failed".to_string())
}

#[tauri::command]
pub async fn stop_gateway(
    state: tauri::State<'_, GatewayState>,
) -> Result<(), String> {
    log::info!("Stopping OpenClaw Gateway");
    
    let mut process_guard = state.process.lock().await;
    let mut start_time_guard = state.start_time.lock().await;
    
    if let Some(mut child) = process_guard.take() {
        // Try graceful shutdown first
        let _ = run_gateway_command(&["gateway", "stop"]);
        
        // Wait a bit for graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Force kill if still running
        match child.try_wait() {
            Ok(Some(_)) => {
                log::info!("Gateway stopped gracefully");
            }
            Ok(None) => {
                log::warn!("Gateway didn't stop gracefully, killing...");
                child.kill().map_err(|e| format!("Failed to kill Gateway: {}", e))?;
            }
            Err(e) => {
                log::error!("Error checking Gateway status: {}", e);
            }
        }
    }
    
    *start_time_guard = None;
    log::info!("Gateway stopped");
    
    Ok(())
}

#[tauri::command]
pub async fn restart_gateway(
    state: tauri::State<'_, GatewayState>,
) -> Result<GatewayStatus, String> {
    log::info!("Restarting OpenClaw Gateway");
    
    // Stop first
    stop_gateway(state.clone()).await?;
    
    // Wait a moment
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Get the current port
    let port = *state.port.lock().await;
    
    // Start again
    start_gateway(port, state).await
}

#[tauri::command]
pub async fn gateway_health(
    state: tauri::State<'_, GatewayState>,
) -> Result<GatewayHealth, String> {
    let port = *state.port.lock().await;
    check_health(port).await
}

#[tauri::command]
pub async fn gateway_status(
    state: tauri::State<'_, GatewayState>,
) -> Result<GatewayStatus, String> {
    let process_guard = state.process.lock().await;
    let port_guard = state.port.lock().await;
    let start_time_guard = state.start_time.lock().await;
    
    let running = process_guard.is_some();
    let pid = process_guard.as_ref().map(|c| c.id());
    let uptime_secs = start_time_guard.map(|t| t.elapsed().as_secs());
    
    Ok(GatewayStatus {
        running,
        port: *port_guard,
        pid,
        uptime_secs,
        version: if running { get_openclaw_version().await.ok() } else { None },
    })
}

#[tauri::command]
pub async fn gateway_logs(tail: usize) -> Result<Vec<String>, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let log_file = home.join(".openclaw/logs/gateway.log");
    
    if !log_file.exists() {
        return Ok(vec!["No logs found".to_string()]);
    }
    
    // Read last N lines
    let content = std::fs::read_to_string(&log_file)
        .map_err(|e| format!("Failed to read log file: {}", e))?;
    
    let lines: Vec<String> = content
        .lines()
        .rev()
        .take(tail)
        .rev()
        .map(|s| s.to_string())
        .collect();
    
    Ok(lines)
}

#[tauri::command]
pub async fn check_openclaw_update() -> Result<Option<String>, String> {
    let current = get_openclaw_version().await?;
    let latest = get_latest_openclaw_version().await?;
    
    if current != latest {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn upgrade_openclaw(window: tauri::Window) -> Result<(), String> {
    log::info!("Upgrading OpenClaw...");
    
    // Emit progress
    let _ = window.emit("upgrade-progress", serde_json::json!({
        "step": "upgrade",
        "percent": 0,
        "message": "Starting upgrade..."
    }));
    
    let output = Command::new("npm")
        .args(["update", "-g", "openclaw"])
        .output()
        .map_err(|e| format!("Failed to upgrade OpenClaw: {}", e))?;
    
    if output.status.success() {
        let _ = window.emit("upgrade-progress", serde_json::json!({
            "step": "upgrade",
            "percent": 100,
            "message": "Upgrade complete!"
        }));
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Upgrade failed: {}", stderr))
    }
}

// ─── Helper Functions ───

async fn check_health(port: u16) -> Result<GatewayHealth, String> {
    let url = format!("http://127.0.0.1:{}/healthz", port);
    
    let start = std::time::Instant::now();
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    match client.get(&url).send().await {
        Ok(resp) => {
            let response_time_ms = start.elapsed().as_millis() as u64;
            
            if resp.status().is_success() {
                // Try to parse active connections from response
                let active_connections = resp.json::<serde_json::Value>().await
                    .ok()
                    .and_then(|v| v.get("activeConnections")?.as_u64() as Option<u32>);
                
                Ok(GatewayHealth {
                    healthy: true,
                    response_time_ms,
                    active_connections,
                })
            } else {
                Ok(GatewayHealth {
                    healthy: false,
                    response_time_ms,
                    active_connections: None,
                })
            }
        }
        Err(_) => Ok(GatewayHealth {
            healthy: false,
            response_time_ms: 0,
            active_connections: None,
        }),
    }
}

async fn is_port_in_use(port: u16) -> bool {
    use std::net::TcpStream;
    
    TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

async fn get_openclaw_version() -> Result<String, String> {
    let output = Command::new("openclaw")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to get OpenClaw version: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get version".to_string())
    }
}

async fn get_latest_openclaw_version() -> Result<String, String> {
    let output = Command::new("npm")
        .args(["view", "openclaw", "version"])
        .output()
        .map_err(|e| format!("Failed to check latest version: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get latest version".to_string())
    }
}

fn run_gateway_command(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("openclaw")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run openclaw command: {}", e))
}

// ─── Cleanup on Exit ───

pub fn cleanup_on_exit(state: &GatewayState) {
    // This is called synchronously from the Tauri exit handler
    // We need to use try_lock to avoid blocking
    if let Ok(mut process_guard) = state.process.try_lock() {
        if let Some(mut child) = process_guard.take() {
            log::info!("Cleaning up Gateway process on exit");
            let _ = child.kill();
        }
    }
}