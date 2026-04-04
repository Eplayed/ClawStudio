// setup.rs - Environment detection and installation management
// Simplified version without window event emissions

use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub installed: bool,
    pub version: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawInfo {
    pub installed: bool,
    pub version: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub port: u16,
    pub uptime_sec: u64,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvStatus {
    pub node: NodeInfo,
    pub npm: NodeInfo,
    pub openclaw: OpenClawInfo,
    pub gateway: Option<GatewayStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub auth_provider: String,
    pub api_key: String,
    pub default_model: String,
    pub gateway_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallScope {
    pub stop_gateway: bool,
    pub remove_cli: bool,
    pub remove_config: bool,
    pub remove_node: bool,
}

// ─── Detection Functions ───

#[tauri::command]
pub fn detect_node() -> Result<NodeInfo, String> {
    let output = StdCommand::new("node").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let path = which::which("node").ok().map(|p| p.to_string_lossy().to_string());
            Ok(NodeInfo {
                installed: true,
                version: version.trim_start_matches('v').to_string(),
                path,
            })
        }
        _ => Ok(NodeInfo {
            installed: false,
            version: String::new(),
            path: None,
        }),
    }
}

#[tauri::command]
pub fn detect_npm() -> Result<NodeInfo, String> {
    let output = StdCommand::new("npm").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let path = which::which("npm").ok().map(|p| p.to_string_lossy().to_string());
            Ok(NodeInfo { installed: true, version, path })
        }
        _ => Ok(NodeInfo { installed: false, version: String::new(), path: None }),
    }
}

#[tauri::command]
pub fn detect_openclaw() -> Result<OpenClawInfo, String> {
    let output = StdCommand::new("openclaw").arg("--version").output();

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let path = which::which("openclaw").ok().map(|p| p.to_string_lossy().to_string());
            Ok(OpenClawInfo { installed: true, version, path })
        }
        _ => Ok(OpenClawInfo { installed: false, version: String::new(), path: None }),
    }
}

#[tauri::command]
pub async fn detect_gateway_status() -> Result<Option<GatewayStatus>, String> {
    let output = StdCommand::new("openclaw").args(["gateway", "status"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let running = stdout.contains("running");
            let port = extract_port(&stdout).unwrap_or(18789);
            let pid = extract_pid(&stdout);
            Ok(Some(GatewayStatus { running, port, uptime_sec: 0, pid }))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
pub async fn get_env_status() -> Result<EnvStatus, String> {
    let node = detect_node()?;
    let npm = detect_npm()?;
    let openclaw = detect_openclaw()?;
    let gateway = detect_gateway_status().await?;
    Ok(EnvStatus { node, npm, openclaw, gateway })
}

// ─── Installation Functions ───

#[tauri::command]
pub async fn install_node(_use_mirror: bool) -> Result<String, String> {
    log::info!("Installing Node.js...");

    #[cfg(target_os = "macos")]
    {
        let brew_check = StdCommand::new("which").arg("brew").output();
        let has_brew = brew_check.map(|o| o.status.success()).unwrap_or(false);

        if has_brew {
            let output = StdCommand::new("brew")
                .args(["install", "node@22"])
                .output();

            match output {
                Ok(o) if o.status.success() => return Ok("Node.js installed via Homebrew".to_string()),
                Ok(o) => return Err(String::from_utf8_lossy(&o.stderr).to_string()),
                Err(e) => return Err(format!("Failed to run brew: {}", e)),
            }
        } else {
            return Err("Homebrew not found. Please install Node.js manually from https://nodejs.org".to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = StdCommand::new("winget")
            .args(["install", "OpenJS.NodeJS.LTS", "--accept-source-agreements", "--accept-package-agreements"])
            .output();

        match output {
            Ok(o) if o.status.success() => return Ok("Node.js installed via winget".to_string()),
            Ok(o) => return Err(String::from_utf8_lossy(&o.stderr).to_string()),
            Err(e) => return Err(format!("Failed to run winget: {}. Please install Node.js manually", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = StdCommand::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs")
            .output();

        match output {
            Ok(o) if o.status.success() => return Ok("Node.js installed".to_string()),
            Ok(o) => return Err(String::from_utf8_lossy(&o.stderr).to_string()),
            Err(e) => return Err(format!("Failed: {}. Please install Node.js manually", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported platform. Please install Node.js manually".to_string())
    }
}

#[tauri::command]
pub async fn install_openclaw(use_mirror: bool) -> Result<String, String> {
    log::info!("Installing OpenClaw...");

    let args = if use_mirror {
        vec!["install", "-g", "openclaw@latest", "--registry=https://registry.npmmirror.com"]
    } else {
        vec!["install", "-g", "openclaw@latest"]
    };

    let output = StdCommand::new("npm").args(&args).output();

    match output {
        Ok(o) if o.status.success() => Ok("OpenClaw installed successfully".to_string()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => Err(format!("Failed to run npm: {}", e)),
    }
}

#[tauri::command]
pub async fn configure_openclaw(config: SetupConfig) -> Result<String, String> {
    log::info!("Configuring OpenClaw...");

    // Save API key to keychain
    let entry = keyring::Entry::new("clawstudio", &config.auth_provider)
        .map_err(|e| e.to_string())?;
    entry.set_password(&config.api_key).map_err(|e| e.to_string())?;

    Ok("Configuration saved".to_string())
}

#[tauri::command]
pub async fn configure_openclaw_proxy(
    proxy_port: u16,
    api_key: Option<String>,
    auth_provider: Option<String>,
    default_model: Option<String>,
) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .ok_or("Cannot determine home directory")?;
    
    // 读取现有配置
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    // 确保结构存在
    if !config["agents"].is_object() {
        config["agents"] = serde_json::json!({});
    }
    if !config["agents"]["defaults"].is_object() {
        config["agents"]["defaults"] = serde_json::json!({});
    }
    
    // 修改 API base URL
    config["agents"]["defaults"]["api_base"] = serde_json::json!(
        format!("http://127.0.0.1:{}/v1", proxy_port)
    );
    
    // 如果提供了 api_key，写入配置
    if let Some(ref key) = api_key {
        config["agents"]["defaults"]["api_key"] = serde_json::json!(key);
    }
    
    // 如果提供了 auth_provider，写入配置
    if let Some(ref provider) = auth_provider {
        config["agents"]["defaults"]["auth_provider"] = serde_json::json!(provider);
    }
    
    // 如果提供了 default_model，写入配置
    if let Some(ref model) = default_model {
        config["agents"]["defaults"]["model"] = serde_json::json!(model);
    }
    
    // 写入配置
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    // 如果同时提供了 api_key 和 auth_provider，保存到 keychain
    if let (Some(ref key), Some(ref provider)) = (&api_key, &auth_provider) {
        let entry = keyring::Entry::new("clawstudio", provider)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
        entry.set_password(key).map_err(|e| format!("Failed to save to keychain: {}", e))?;
        log::info!("API key saved to keychain for provider: {}", provider);
    }
        
    Ok(())
}

#[tauri::command]
pub async fn uninstall_openclaw(scope: UninstallScope) -> Result<String, String> {
    log::info!("Uninstalling OpenClaw...");

    if scope.stop_gateway {
        let _ = StdCommand::new("openclaw").args(["gateway", "stop"]).output();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    if scope.remove_config {
        let _ = StdCommand::new("openclaw").args(["uninstall", "--all", "--yes"]).output();
        if let Some(home) = dirs::home_dir() {
            let openclaw_dir = home.join(".openclaw");
            if openclaw_dir.exists() {
                let _ = std::fs::remove_dir_all(&openclaw_dir);
            }
        }
    }

    if scope.remove_cli {
        let _ = StdCommand::new("npm").args(["uninstall", "-g", "openclaw"]).output();
    }

    Ok("Uninstallation complete".to_string())
}

#[tauri::command]
pub async fn check_environment() -> Result<EnvStatus, String> {
    get_env_status().await
}

#[tauri::command]
pub async fn start_gateway_from_setup(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::gateway::GatewayState>,
    proxy_state: tauri::State<'_, crate::proxy::ProxyServerState>,
    port: Option<u16>,
) -> Result<String, String> {
    let port = port.unwrap_or(18789);
    match crate::gateway::start_gateway(app_handle, state, proxy_state, Some(port)).await {
        Ok(_) => Ok(format!("Gateway started on port {}", port)),
        Err(e) => Err(e),
    }
}

// ─── Setup State ───

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub step: String,
    pub percent: u8,
    pub message: String,
    pub log_line: String,
    pub success: bool,
}

pub struct SetupState {
    pub current_step: Arc<Mutex<u8>>,
    pub install_progress: Arc<Mutex<InstallProgress>>,
}

impl SetupState {
    pub fn new() -> Self {
        Self {
            current_step: Arc::new(Mutex::new(0)),
            install_progress: Arc::new(Mutex::new(InstallProgress {
                step: String::new(),
                percent: 0,
                message: String::new(),
                log_line: String::new(),
                success: true,
            })),
        }
    }
}

impl Default for SetupState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helper Functions ───

fn extract_port(output: &str) -> Option<u16> {
    let re = regex::Regex::new(r"port[:\s]+(\d+)|:(\d+)").ok()?;
    let caps = re.captures(output)?;
    let port_str = caps.get(1).or(caps.get(2))?.as_str();
    port_str.parse().ok()
}

fn extract_pid(output: &str) -> Option<u32> {
    let re = regex::Regex::new(r"PID[:\s]+(\d+)").ok()?;
    let caps = re.captures(output)?;
    let pid_str = caps.get(1)?.as_str();
    pid_str.parse().ok()
}
