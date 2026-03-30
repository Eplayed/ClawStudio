// setup.rs - Environment Detection and Installation Engine
// Implements Phase 1 of ClawStudio v2.0 roadmap

use std::process::Command;
use std::sync::Arc;
use tauri::{Manager, Window, Emitter};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvStatus {
    pub node_installed: bool,
    pub node_version: Option<String>,
    pub npm_installed: bool,
    pub npm_version: Option<String>,
    pub openclaw_installed: bool,
    pub openclaw_version: Option<String>,
    pub gateway_running: bool,
    pub gateway_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub step: String,
    pub percent: u32,
    pub message: String,
    pub log_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub api_provider: String,
    pub api_key: String,
    pub default_model: String,
    pub gateway_port: u16,
    pub use_mirror: bool,
}

// ─── Setup State ───

pub struct SetupState {
    pub env_status: Arc<Mutex<EnvStatus>>,
}

impl SetupState {
    pub fn new() -> Self {
        Self {
            env_status: Arc::new(Mutex::new(EnvStatus {
                node_installed: false,
                node_version: None,
                npm_installed: false,
                npm_version: None,
                openclaw_installed: false,
                openclaw_version: None,
                gateway_running: false,
                gateway_port: None,
            })),
        }
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub async fn check_environment() -> Result<EnvStatus, String> {
    log::info!("Checking environment...");
    
    let mut status = EnvStatus {
        node_installed: false,
        node_version: None,
        npm_installed: false,
        npm_version: None,
        openclaw_installed: false,
        openclaw_version: None,
        gateway_running: false,
        gateway_port: None,
    };
    
    // Check Node.js
    if let Ok(output) = run_command("node", &["--version"]) {
        if output.status.success() {
            status.node_installed = true;
            status.node_version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            log::info!("Node.js found: {:?}", status.node_version);
        }
    }
    
    // Check npm
    if let Ok(output) = run_command("npm", &["--version"]) {
        if output.status.success() {
            status.npm_installed = true;
            status.npm_version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            log::info!("npm found: {:?}", status.npm_version);
        }
    }
    
    // Check OpenClaw
    if let Ok(output) = run_command("openclaw", &["--version"]) {
        if output.status.success() {
            status.openclaw_installed = true;
            status.openclaw_version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            log::info!("OpenClaw found: {:?}", status.openclaw_version);
        }
    }
    
    // Check Gateway health
    if let Ok(running) = check_gateway_health(18789).await {
        status.gateway_running = running;
        status.gateway_port = if running { Some(18789) } else { None };
    }
    
    Ok(status)
}

#[tauri::command]
pub async fn install_node(window: Window) -> Result<(), String> {
    log::info!("Starting Node.js installation...");
    
    emit_progress(&window, "install_node", 0, "Starting Node.js installation...", None);
    
    #[cfg(target_os = "windows")]
    {
        // Windows: Use winget or download .msi
        emit_progress(&window, "install_node", 10, "Downloading Node.js installer...", None);
        
        // Try winget first
        let result = run_command_with_progress(
            &window,
            "winget",
            &["install", "OpenJS.NodeJS.LTS", "-e", "--silent"],
            "install_node",
        ).await;
        
        match result {
            Ok(_) => {
                emit_progress(&window, "install_node", 100, "Node.js installed successfully!", None);
                return Ok(());
            }
            Err(e) => {
                emit_progress(&window, "install_node", 20, &format!("Winget failed: {}, trying alternate method...", e), None);
                // Fallback: Download .msi manually
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        emit_progress(&window, "install_node", 10, "Installing via Homebrew...", None);
        
        let result = run_command_with_progress(
            &window,
            "brew",
            &["install", "node@22"],
            "install_node",
        ).await;
        
        match result {
            Ok(_) => {
                emit_progress(&window, "install_node", 100, "Node.js installed successfully!", None);
                return Ok(());
            }
            Err(e) => {
                emit_progress(&window, "install_node", 50, &format!("Homebrew failed: {}", e), None);
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        emit_progress(&window, "install_node", 10, "Installing via NodeSource...", None);
        
        // Use NodeSource setup script
        let script = "curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash - && sudo apt-get install -y nodejs";
        let result = run_command_with_progress(
            &window,
            "bash",
            &["-c", script],
            "install_node",
        ).await;
        
        match result {
            Ok(_) => {
                emit_progress(&window, "install_node", 100, "Node.js installed successfully!", None);
                return Ok(());
            }
            Err(e) => {
                return Err(format!("Failed to install Node.js: {}", e));
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn install_openclaw(window: Window, use_mirror: bool) -> Result<(), String> {
    log::info!("Starting OpenClaw installation (mirror: {})", use_mirror);
    
    emit_progress(&window, "install_openclaw", 0, "Starting OpenClaw installation...", None);
    
    let mut args = vec!["install", "-g", "openclaw@latest"];
    
    if use_mirror {
        args.push("--registry=https://registry.npmmirror.com");
        emit_progress(&window, "install_openclaw", 5, "Using China mirror for faster download...", None);
    }
    
    emit_progress(&window, "install_openclaw", 10, "Running npm install...", None);
    
    let result = run_command_with_progress(
        &window,
        "npm",
        &args,
        "install_openclaw",
    ).await;
    
    match result {
        Ok(_) => {
            emit_progress(&window, "install_openclaw", 100, "OpenClaw installed successfully!", None);
            Ok(())
        }
        Err(e) => {
            emit_progress(&window, "install_openclaw", 0, &format!("Installation failed: {}", e), None);
            Err(format!("Failed to install OpenClaw: {}", e))
        }
    }
}

#[tauri::command]
pub async fn configure_openclaw(config: SetupConfig, window: Window) -> Result<(), String> {
    log::info!("Configuring OpenClaw with provider: {}", config.api_provider);
    
    emit_progress(&window, "configure", 0, "Configuring OpenClaw...", None);
    
    // Run openclaw onboard in non-interactive mode
    let args = vec![
        "onboard",
        "--non-interactive",
        "--auth-choice", &config.api_provider,
        "--secret-input-mode", "plaintext",
    ];
    
    emit_progress(&window, "configure", 50, "Running onboard process...", None);
    
    // Note: In production, we'd pipe the API key to stdin
    // For now, we'll write the config directly
    let config_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".openclaw");
    
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    
    let config_file = config_dir.join("openclaw.json");
    let config_json = serde_json::json!({
        "agents": {
            "defaults": {
                "model": config.default_model,
            }
        },
        "gateway": {
            "port": config.gateway_port
        }
    });
    
    std::fs::write(&config_file, serde_json::to_string_pretty(&config_json).unwrap())
        .map_err(|e| e.to_string())?;
    
    emit_progress(&window, "configure", 100, "Configuration saved!", None);
    
    Ok(())
}

#[tauri::command]
pub async fn start_gateway_from_setup(window: Window, port: u16) -> Result<(), String> {
    log::info!("Starting OpenClaw Gateway on port {}", port);
    
    emit_progress(&window, "start_gateway", 0, "Starting Gateway...", None);
    
    let args = vec!["gateway", "start", "--port", &port.to_string()];
    
    let result = run_command("openclaw", &args)?;
    
    if result.status.success() {
        // Wait for gateway to be healthy
        emit_progress(&window, "start_gateway", 50, "Waiting for Gateway to be ready...", None);
        
        for i in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if check_gateway_health(port).await? {
                emit_progress(&window, "start_gateway", 100, "Gateway started successfully!", None);
                return Ok(());
            }
            emit_progress(&window, "start_gateway", 50 + (i * 5) as u32, "Waiting for Gateway...", None);
        }
        
        Err("Gateway failed to start within 10 seconds".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Failed to start Gateway: {}", stderr))
    }
}

#[tauri::command]
pub async fn uninstall_openclaw(window: Window, remove_data: bool, remove_node: bool) -> Result<(), String> {
    log::info!("Uninstalling OpenClaw (remove_data: {}, remove_node: {})", remove_data, remove_node);
    
    emit_progress(&window, "uninstall", 0, "Starting uninstallation...", None);
    
    // Stop Gateway first
    emit_progress(&window, "uninstall", 10, "Stopping Gateway...", None);
    let _ = run_command("openclaw", &["gateway", "stop"]);
    
    // Remove data if requested
    if remove_data {
        emit_progress(&window, "uninstall", 30, "Removing OpenClaw data...", None);
        if let Some(home) = dirs::home_dir() {
            let openclaw_dir = home.join(".openclaw");
            if openclaw_dir.exists() {
                std::fs::remove_dir_all(&openclaw_dir).map_err(|e| e.to_string())?;
            }
        }
    }
    
    // Uninstall OpenClaw
    emit_progress(&window, "uninstall", 50, "Uninstalling OpenClaw CLI...", None);
    run_command("npm", &["uninstall", "-g", "openclaw"])?;
    
    // Remove Node.js if requested
    if remove_node {
        emit_progress(&window, "uninstall", 70, "Removing Node.js...", None);
        #[cfg(target_os = "windows")]
        {
            let _ = run_command("winget", &["uninstall", "OpenJS.NodeJS.LTS", "--silent"]);
        }
        #[cfg(target_os = "macos")]
        {
            let _ = run_command("brew", &["uninstall", "node@22"]);
        }
        #[cfg(target_os = "linux")]
        {
            let _ = run_command("sudo", &["apt-get", "remove", "-y", "nodejs"]);
        }
    }
    
    emit_progress(&window, "uninstall", 100, "Uninstallation complete!", None);
    
    Ok(())
}

// ─── Helper Functions ───

fn run_command(cmd: &str, args: &[&str]) -> Result<std::process::Output, String> {
    log::info!("Running command: {} {:?}", cmd, args);
    
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))
}

async fn run_command_with_progress(
    window: &Window,
    cmd: &str,
    args: &[&str],
    step: &str,
) -> Result<(), String> {
    use std::process::Stdio;
    use std::io::{BufRead, BufReader};
    
    log::info!("Running command with progress: {} {:?}", cmd, args);
    
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;
    
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut percent = 10u32;
    
    // Read stdout
    while let Some(Ok(line)) = stdout_reader.next() {
        log::info!("[{}] {}", cmd, line);
        percent = (percent + 5).min(95);
        emit_progress(window, step, percent, &line, Some(&line));
    }
    
    // Read stderr
    while let Some(Ok(line)) = stderr_reader.next() {
        log::warn!("[{}] {}", cmd, line);
        emit_progress(window, step, percent, &line, Some(&line));
    }
    
    let status = child.wait().map_err(|e| e.to_string())?;
    
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command {} failed with status: {}", cmd, status))
    }
}

async fn check_gateway_health(port: u16) -> Result<bool, String> {
    let url = format!("http://127.0.0.1:{}/healthz", port);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    
    match client.get(&url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

fn emit_progress(window: &Window, step: &str, percent: u32, message: &str, log_line: Option<&str>) {
    let progress = InstallProgress {
        step: step.to_string(),
        percent,
        message: message.to_string(),
        log_line: log_line.map(|s| s.to_string()),
    };
    
    if let Err(e) = window.emit("setup-progress", &progress) {
        log::error!("Failed to emit progress: {}", e);
    }
}