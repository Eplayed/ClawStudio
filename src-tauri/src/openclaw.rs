// openclaw.rs - OpenClaw process lifecycle management
// Manages spawning, stopping, and event streaming from OpenClaw core

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProcess {
    pub id: String,
    pub name: String,
    pub pid: u32,
    pub status: String, // "running" | "paused" | "stopped"
    pub config_path: String,
}

pub struct OpenClawManager {
    processes: Arc<Mutex<HashMap<String, AgentProcess>>>,
}

impl OpenClawManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Start an OpenClaw agent as a child process
#[tauri::command]
pub async fn start_agent(
    state: tauri::State<'_, OpenClawManager>,
    window: tauri::Window,
    agent_id: String,
    agent_name: String,
    config_path: String,
) -> Result<AgentProcess, String> {
    // Spawn OpenClaw process using tokio::process::Command for async stdout
    let mut child = Command::new("openclaw")
        .args(["--config", &config_path, "--event-stream", "stdout"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start OpenClaw: {}", e))?;

    let pid = child.id().unwrap_or(0);
    let agent = AgentProcess {
        id: agent_id.clone(),
        name: agent_name,
        pid,
        status: "running".into(),
        config_path,
    };

    state
        .processes
        .lock()
        .await
        .insert(agent_id.clone(), agent.clone());

    // Take stdout handle for async reading
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout")?;

    // Spawn a task to read stdout events and emit to frontend
    let window_clone = window.clone();
    let agent_id_clone = agent_id.clone();
    tauri::async_runtime::spawn(async move {
        stream_events(stdout, agent_id_clone, window_clone).await;
        // Wait for child to exit after stream ends
        let _ = child.wait().await;
    });

    // Write PID file for crash recovery
    let pid_path = format!("/tmp/clawstudio_{}.pid", agent_id);
    std::fs::write(&pid_path, pid.to_string()).ok();

    log::info!("Agent {} started with PID {}", agent_id, pid);
    Ok(agent)
}

/// Read OpenClaw stdout line by line and emit structured events
async fn stream_events(
    stdout: tokio::process::ChildStdout,
    agent_id: String,
    window: tauri::Window,
) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        // Parse OpenClaw event JSON
        // Expected format: {"type":"action","tool":"mouse_move","params":{"x":342,"y":567}}
        let event_payload = serde_json::json!({
            "agent_id": agent_id,
            "raw": line,
        });
        window.emit("openclaw-event", event_payload).ok();
    }

    // Process ended
    window
        .emit(
            "openclaw-event",
            serde_json::json!({
                "agent_id": agent_id,
                "type": "process_exit",
            }),
        )
        .ok();
}

/// Stop an agent process by ID
#[tauri::command]
pub async fn stop_agent(
    state: tauri::State<'_, OpenClawManager>,
    agent_id: String,
) -> Result<(), String> {
    let mut procs = state.processes.lock().await;
    if let Some(agent) = procs.get_mut(&agent_id) {
        // Send SIGTERM
        #[cfg(unix)]
        {
            use std::process::Command as StdCommand;
            StdCommand::new("kill")
                .args(["-TERM", &agent.pid.to_string()])
                .output()
                .ok();
        }
        #[cfg(windows)]
        {
            use std::process::Command as StdCommand;
            StdCommand::new("taskkill")
                .args(["/PID", &agent.pid.to_string(), "/F"])
                .output()
                .ok();
        }

        agent.status = "stopped".into();

        // Clean up PID file
        let pid_path = format!("/tmp/clawstudio_{}.pid", agent_id);
        std::fs::remove_file(&pid_path).ok();

        log::info!("Agent {} stopped", agent_id);
    }
    Ok(())
}

/// List all managed agent processes
#[tauri::command]
pub async fn list_agents(
    state: tauri::State<'_, OpenClawManager>,
) -> Result<Vec<AgentProcess>, String> {
    let procs = state.processes.lock().await;
    Ok(procs.values().cloned().collect())
}

/// Get status of a specific agent
#[tauri::command]
pub async fn get_agent_status(
    state: tauri::State<'_, OpenClawManager>,
    agent_id: String,
) -> Result<AgentProcess, String> {
    let procs = state.processes.lock().await;
    procs
        .get(&agent_id)
        .cloned()
        .ok_or_else(|| format!("Agent {} not found", agent_id))
}

/// Check for orphan OpenClaw processes on startup
pub async fn check_orphan_processes(window: tauri::WebviewWindow) {
    let pid_dir = std::path::Path::new("/tmp");
    if let Ok(entries) = std::fs::read_dir(pid_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("clawstudio_") && name.ends_with(".pid") {
                if let Ok(pid_str) = std::fs::read_to_string(entry.path()) {
                    let pid_str = pid_str.trim();
                    log::warn!("Found orphan PID file: {} (PID: {})", name, pid_str);
                    window
                        .emit(
                            "orphan-process",
                            serde_json::json!({
                                "pid_file": name,
                                "pid": pid_str,
                            }),
                        )
                        .ok();
                }
            }
        }
    }
}
