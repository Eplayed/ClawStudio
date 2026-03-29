// vnc_client.rs - VNC Screenshot Client
// Connects to noVNC WebSocket and captures screenshots

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// ─── VNC Protocol Constants ───

const VNC_FRAMEBUFFER_UPDATE: u8 = 3;
const VNC_SET_PIXEL_FORMAT: u8 = 0;
const VNC_SET_ENCODINGS: u8 = 2;
const VNC_FRAME_BUFFER_REQUEST: u8 = 3;

// ─── VNC Client ───

pub struct VNCClient {
    url: String,
    connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNCConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl VNCClient {
    pub fn new(config: VNCConfig) -> Self {
        let url = format!("ws://{}:{}/websockify", config.host, config.port);
        Self {
            url,
            connected: false,
        }
    }

    /// Connect to VNC server via WebSocket
    pub async fn connect(&mut self) -> Result<(), String> {
        log::info!("Connecting to VNC: {}", self.url);

        // For now, we'll use a simpler approach:
        // Connect to noVNC's WebSocket endpoint
        match connect_async(&self.url).await {
            Ok((ws_stream, _)) => {
                log::info!("VNC WebSocket connected");
                self.connected = true;
                Ok(())
            }
            Err(e) => {
                log::error!("VNC connection failed: {}", e);
                Err(format!("VNC connection failed: {}", e))
            }
        }
    }

    /// Capture screenshot from VNC
    /// Returns base64 encoded PNG
    pub async fn capture_screenshot(&self) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected to VNC".to_string());
        }

        // For now, return a placeholder
        // In production, this would:
        // 1. Send FramebufferUpdateRequest
        // 2. Receive raw pixel data
        // 3. Convert to PNG
        // 4. Encode as base64

        log::info!("Capturing VNC screenshot");
        Ok("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string())
    }

    /// Send mouse event to VNC
    pub async fn send_mouse_event(
        &self,
        x: i32,
        y: i32,
        button_mask: u8,
    ) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to VNC".to_string());
        }

        log::info!("VNC mouse event: ({}, {}) button={}", x, y, button_mask);
        // In production: send PointerEvent message via WebSocket
        Ok(())
    }

    /// Send keyboard event to VNC
    pub async fn send_key_event(&self, key: u32, down: bool) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to VNC".to_string());
        }

        log::info!("VNC key event: {} down={}", key, down);
        // In production: send KeyEvent message via WebSocket
        Ok(())
    }

    /// Disconnect from VNC
    pub fn disconnect(&mut self) {
        self.connected = false;
        log::info!("VNC disconnected");
    }
}

// ─── Screenshot via Docker exec (Alternative) ───

pub async fn capture_screenshot_via_docker(
    container_id: &str,
) -> Result<String, String> {
    use tokio::process::Command;

    log::info!("Capturing screenshot from container: {}", container_id);

    // Method 1: Use xdotool + import (if installed in container)
    // docker exec <container> import -window root /tmp/screenshot.png

    // Method 2: Use scrot (if installed)
    // docker exec <container> scrot /tmp/screenshot.png

    // For now, return a placeholder
    // In production:
    // 1. Execute screenshot command in container
    // 2. Copy file from container to host
    // 3. Read file and encode as base64

    let output = Command::new("docker")
        .args([
            "exec",
            container_id,
            "import",
            "-window",
            "root",
            "/tmp/screenshot.png",
        ])
        .output()
        .await;

    match output {
        Ok(_) => {
            // Read the screenshot file
            let file_output = Command::new("docker")
                .args([
                    "exec",
                    container_id,
                    "cat",
                    "/tmp/screenshot.png",
                ])
                .output()
                .await;

            match file_output {
                Ok(data) => {
                    if data.status.success() {
                        // Encode as base64
                        Ok(base64_encode(&data.stdout))
                    } else {
                        Err("Failed to read screenshot file".to_string())
                    }
                }
                Err(e) => Err(format!("Failed to read screenshot: {}", e)),
            }
        }
        Err(e) => {
            log::warn!("Docker screenshot failed: {}, using placeholder", e);
            // Return placeholder if docker exec fails
            Ok("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string())
        }
    }
}

// ─── Helper Functions ───

fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

// ─── Tauri Commands for VNC ───

#[tauri::command]
pub async fn vnc_connect(
    host: String,
    port: u16,
    password: Option<String>,
) -> Result<String, String> {
    let config = VNCConfig { host, port, password };
    let mut client = VNCClient::new(config);
    client.connect().await?;
    Ok("connected".to_string())
}

#[tauri::command]
pub async fn vnc_screenshot(
    sandbox_id: String,
) -> Result<String, String> {
    // Try docker method first
    capture_screenshot_via_docker(&sandbox_id).await
}

#[tauri::command]
pub async fn vnc_mouse_move(
    sandbox_id: String,
    x: i32,
    y: i32,
) -> Result<(), String> {
    use tokio::process::Command;

    let output = Command::new("docker")
        .args([
            "exec",
            &sandbox_id,
            "xdotool",
            "mousemove",
            &x.to_string(),
            &y.to_string(),
        ])
        .output()
        .await;

    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to move mouse: {}", e)),
    }
}

#[tauri::command]
pub async fn vnc_click(
    sandbox_id: String,
    x: i32,
    y: i32,
    button: u8, // 1=left, 2=middle, 3=right
) -> Result<(), String> {
    use tokio::process::Command;

    let button_str = match button {
        1 => "1",
        2 => "2",
        3 => "3",
        _ => "1",
    };

    // Move mouse first, then click
    let _ = Command::new("docker")
        .args([
            "exec",
            &sandbox_id,
            "xdotool",
            "mousemove",
            &x.to_string(),
            &y.to_string(),
        ])
        .output()
        .await;

    let output = Command::new("docker")
        .args([
            "exec",
            &sandbox_id,
            "xdotool",
            "click",
            button_str,
        ])
        .output()
        .await;

    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to click: {}", e)),
    }
}

#[tauri::command]
pub async fn vnc_type_text(
    sandbox_id: String,
    text: String,
) -> Result<(), String> {
    use tokio::process::Command;

    let output = Command::new("docker")
        .args([
            "exec",
            &sandbox_id,
            "xdotool",
            "type",
            &text,
        ])
        .output()
        .await;

    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to type text: {}", e)),
    }
}

#[tauri::command]
pub async fn vnc_press_key(
    sandbox_id: String,
    key: String,
) -> Result<(), String> {
    use tokio::process::Command;

    let output = Command::new("docker")
        .args([
            "exec",
            &sandbox_id,
            "xdotool",
            "key",
            &key,
        ])
        .output()
        .await;

    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to press key: {}", e)),
    }
}
