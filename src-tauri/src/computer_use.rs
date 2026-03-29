// computer_use.rs - Computer Use Runtime
// Manages Claude computer use tool loop with sandbox integration
// Handles screenshot capture, mouse/keyboard input, and HITL interception

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::vnc_client;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CUSession {
    pub id: String,
    pub agent_id: String,
    pub sandbox_id: String,
    pub vnc_port: u16,
    pub model: String,
    pub system_prompt: String,
    pub status: CUSessionStatus,
    pub messages: Vec<CUMessage>,
    pub total_steps: usize,
    pub total_cost: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CUSessionStatus {
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CUMessage {
    pub role: String, // "user" | "assistant"
    pub content: Vec<CUContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CUContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String, // "computer", "text_editor", "bash"
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
    },
    Image {
        source: ImageSource,
    },
    Thinking {
        thinking: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String, // "base64"
    pub media_type: String,  // "image/png"
    pub data: String,        // base64 encoded
}

// ─── Computer Use Actions ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ComputerAction {
    Screenshot,
    MouseMove { x: i32, y: i32 },
    LeftClick { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    DoubleClick { x: i32, y: i32 },
    Type { text: String },
    Key { key: String }, // "Return", "Tab", "Escape", etc.
    Scroll { x: i32, y: i32, direction: String }, // "up" | "down"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerActionResult {
    pub success: bool,
    pub screenshot: Option<String>, // base64 PNG
    pub error: Option<String>,
}

// ─── HITL Interception ───

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HITLAction {
    Allow,
    Block,
    AskUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HITLRequest {
    pub session_id: String,
    pub tool_name: String,
    pub action_description: String,
    pub screenshot: Option<String>, // base64 for preview
    pub timeout_sec: u32,
}

// ─── CU Runtime State ───

pub struct CURuntime {
    sessions: Arc<RwLock<HashMap<String, CUSession>>>,
    client: Client,
}

impl CURuntime {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            client: Client::new(),
        }
    }

    /// Start a new CU session
    pub async fn start_session(
        &self,
        agent_id: String,
        sandbox_id: String,
        vnc_port: u16,
        system_prompt: String,
        model: String,
        api_key: String,
    ) -> Result<String, String> {
        let session_id = format!("cu-{}-{}", agent_id, chrono::Utc::now().timestamp());

        // 1. Take initial screenshot
        let screenshot = self
            .capture_screenshot(sandbox_id.clone(), vnc_port)
            .await?;

        // 2. Create initial message with screenshot
        let mut messages = vec![CUMessage {
            role: "user".to_string(),
            content: vec![
                CUContentBlock::Text {
                    text: "You are a helpful AI assistant with computer use capabilities. You can see the desktop and interact with it. Start by taking a screenshot to see the current state.".to_string(),
                },
                CUContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: "image/png".to_string(),
                        data: screenshot.clone(),
                    },
                },
            ],
        }];

        // 3. Create session
        let session = CUSession {
            id: session_id.clone(),
            agent_id,
            sandbox_id,
            vnc_port,
            model,
            system_prompt,
            status: CUSessionStatus::Running,
            messages,
            total_steps: 0,
            total_cost: 0.0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Execute one step of the tool loop
    pub async fn step(
        &self,
        session_id: String,
        api_key: String,
        perm_level: String,
    ) -> Result<CUStepResult, String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or("Session not found")?;

        if session.status != CUSessionStatus::Running {
            return Err("Session is not running".to_string());
        }

        // 1. Call Anthropic API
        let response = self
            .call_anthropic_api(
                &api_key,
                &session.model,
                &session.system_prompt,
                &session.messages,
            )
            .await?;

        // 2. Parse response
        let assistant_message = CUMessage {
            role: "assistant".to_string(),
            content: response.content.clone(),
        };

        session.messages.push(assistant_message.clone());
        session.total_steps += 1;

        // 3. Process tool uses
        let mut tool_results = Vec::new();
        let mut should_pause = false;

        for content in &response.content {
            match content {
                CUContentBlock::ToolUse { id, name, input } => {
                    // Check HITL interception
                    let hitl_action = self.check_hitl(name, input, &perm_level);

                    if hitl_action == HITLAction::AskUser {
                        should_pause = true;
                        // Emit HITL request to frontend
                        let action_desc = format!("{}: {}", name, serde_json::to_string(input).unwrap_or_default());
                        log::info!("[HITL] Pausing for user approval: {}", action_desc);
                        break;
                    } else if hitl_action == HITLAction::Block {
                        tool_results.push(CUContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: "Action blocked by HITL policy".to_string(),
                        });
                        continue;
                    }

                    // Execute tool
                    let result = self
                        .execute_tool(name, input, session.sandbox_id.clone(), session.vnc_port)
                        .await;

                    match result {
                        Ok(result_text) => {
                            tool_results.push(CUContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: result_text,
                            });
                        }
                        Err(e) => {
                            tool_results.push(CUContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: format!("Error: {}", e),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // 4. Add tool results to messages
        if !tool_results.is_empty() {
            session.messages.push(CUMessage {
                role: "user".to_string(),
                content: tool_results,
            });
        }

        // 5. Update cost (estimate based on tokens)
        let estimated_cost = self.estimate_step_cost(&response.content);
        session.total_cost += estimated_cost;

        Ok(CUStepResult {
            session_id,
            step: session.total_steps,
            response: assistant_message,
            paused: should_pause,
            cost: estimated_cost,
        })
    }

    /// Pause session (for HITL)
    pub async fn pause(&self, session_id: String) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or("Session not found")?;
        session.status = CUSessionStatus::Paused;
        Ok(())
    }

    /// Resume session after HITL approval
    pub async fn resume(
        &self,
        session_id: String,
        user_response: String,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or("Session not found")?;

        // Add user response to messages
        session.messages.push(CUMessage {
            role: "user".to_string(),
            content: vec![CUContentBlock::Text {
                text: user_response,
            }],
        });

        session.status = CUSessionStatus::Running;
        Ok(())
    }

    /// Stop session
    pub async fn stop(&self, session_id: String) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or("Session not found")?;
        session.status = CUSessionStatus::Completed;
        Ok(())
    }

    // ─── Private Methods ───

    async fn capture_screenshot(
        &self,
        sandbox_id: String,
        _vnc_port: u16,
    ) -> Result<String, String> {
        // Use VNC client via Docker exec
        log::info!("Capturing screenshot from sandbox: {}", sandbox_id);
        vnc_client::capture_screenshot_via_docker(&sandbox_id).await
    }

    async fn execute_tool(
        &self,
        tool_name: &str,
        input: &Value,
        _sandbox_id: String,
        _vnc_port: u16,
    ) -> Result<String, String> {
        match tool_name {
            "computer" => self.execute_computer_action(input, _sandbox_id, _vnc_port).await,
            "text_editor" => self.execute_text_editor_action(input, _sandbox_id).await,
            "bash" => self.execute_bash_action(input, _sandbox_id).await,
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn execute_computer_action(
        &self,
        input: &Value,
        sandbox_id: String,
        _vnc_port: u16,
    ) -> Result<String, String> {
        let action = input.get("action").and_then(|v| v.as_str()).ok_or("Missing action")?;

        match action {
            "screenshot" => {
                let screenshot = self.capture_screenshot(sandbox_id.clone(), _vnc_port).await?;
                Ok(format!("Screenshot captured: {} bytes", screenshot.len()))
            }
            "mouse_move" => {
                let x = input.get("x").and_then(|v| v.as_i64()).ok_or("Missing x")? as i32;
                let y = input.get("y").and_then(|v| v.as_i64()).ok_or("Missing y")? as i32;
                vnc_client::vnc_mouse_move(sandbox_id, x, y).await?;
                Ok(format!("Moved mouse to ({}, {})", x, y))
            }
            "left_click" => {
                let x = input.get("x").and_then(|v| v.as_i64()).ok_or("Missing x")? as i32;
                let y = input.get("y").and_then(|v| v.as_i64()).ok_or("Missing y")? as i32;
                vnc_client::vnc_click(sandbox_id, x, y, 1).await?;
                Ok(format!("Left clicked at ({}, {})", x, y))
            }
            "right_click" => {
                let x = input.get("x").and_then(|v| v.as_i64()).ok_or("Missing x")? as i32;
                let y = input.get("y").and_then(|v| v.as_i64()).ok_or("Missing y")? as i32;
                vnc_client::vnc_click(sandbox_id, x, y, 3).await?;
                Ok(format!("Right clicked at ({}, {})", x, y))
            }
            "double_click" => {
                let x = input.get("x").and_then(|v| v.as_i64()).ok_or("Missing x")? as i32;
                let y = input.get("y").and_then(|v| v.as_i64()).ok_or("Missing y")? as i32;
                // Double click = two clicks
                vnc_client::vnc_click(sandbox_id.clone(), x, y, 1).await?;
                vnc_client::vnc_click(sandbox_id, x, y, 1).await?;
                Ok(format!("Double clicked at ({}, {})", x, y))
            }
            "type" => {
                let text = input.get("text").and_then(|v| v.as_str()).ok_or("Missing text")?.to_string();
                vnc_client::vnc_type_text(sandbox_id, text.clone()).await?;
                Ok(format!("Typed: {}", text))
            }
            "key" => {
                let key = input.get("key").and_then(|v| v.as_str()).ok_or("Missing key")?.to_string();
                vnc_client::vnc_press_key(sandbox_id, key.clone()).await?;
                Ok(format!("Pressed key: {}", key))
            }
            _ => Err(format!("Unknown computer action: {}", action)),
        }
    }

    async fn execute_text_editor_action(
        &self,
        input: &Value,
        _sandbox_id: String,
    ) -> Result<String, String> {
        let command = input.get("command").and_then(|v| v.as_str()).ok_or("Missing command")?;

        match command {
            "view" => {
                let path = input.get("path").and_then(|v| v.as_str()).ok_or("Missing path")?;
                // TODO: Read file from sandbox
                log::info!("View file: {}", path);
                Ok(format!("File content of {}", path))
            }
            "str_replace" => {
                let path = input.get("path").and_then(|v| v.as_str()).ok_or("Missing path")?;
                let old_str = input.get("old_str").and_then(|v| v.as_str()).ok_or("Missing old_str")?;
                let new_str = input.get("new_str").and_then(|v| v.as_str()).ok_or("Missing new_str")?;
                // TODO: Replace in file
                log::info!("Replace in {}: '{}' -> '{}'", path, old_str, new_str);
                Ok(format!("Replaced in {}", path))
            }
            "create" => {
                let path = input.get("path").and_then(|v| v.as_str()).ok_or("Missing path")?;
                let content = input.get("content").and_then(|v| v.as_str()).ok_or("Missing content")?;
                // TODO: Create file
                log::info!("Create file: {} ({} bytes)", path, content.len());
                Ok(format!("Created file: {}", path))
            }
            _ => Err(format!("Unknown text_editor command: {}", command)),
        }
    }

    async fn execute_bash_action(
        &self,
        input: &Value,
        _sandbox_id: String,
    ) -> Result<String, String> {
        let command = input.get("command").and_then(|v| v.as_str()).ok_or("Missing command")?;

        // TODO: Execute in sandbox via docker exec
        log::info!("Execute bash: {}", command);
        Ok(format!("Command output: {}", command))
    }

    async fn call_anthropic_api(
        &self,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        messages: &[CUMessage],
    ) -> Result<CUMessage, String> {
        let url = "https://api.anthropic.com/v1/messages";

        let request_body = json!({
            "model": model,
            "max_tokens": 8192,
            "system": system_prompt,
            "tools": [
                {
                    "name": "computer",
                    "description": "Interact with the desktop via screenshot, mouse, and keyboard",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["screenshot", "mouse_move", "left_click", "right_click", "double_click", "type", "key", "scroll"]
                            },
                            "x": { "type": "integer" },
                            "y": { "type": "integer" },
                            "text": { "type": "string" },
                            "key": { "type": "string" },
                            "direction": { "type": "string" }
                        },
                        "required": ["action"]
                    }
                },
                {
                    "name": "text_editor",
                    "description": "Read and edit files on the desktop",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string", "enum": ["view", "str_replace", "create"] },
                            "path": { "type": "string" },
                            "old_str": { "type": "string" },
                            "new_str": { "type": "string" },
                            "content": { "type": "string" }
                        },
                        "required": ["command", "path"]
                    }
                },
                {
                    "name": "bash",
                    "description": "Execute bash commands in the sandbox",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string" }
                        },
                        "required": ["command"]
                    }
                }
            ],
            "messages": messages.iter().map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content.iter().map(|c| {
                        match c {
                            CUContentBlock::Text { text } => json!({
                                "type": "text",
                                "text": text
                            }),
                            CUContentBlock::Image { source } => json!({
                                "type": "image",
                                "source": {
                                    "type": source.source_type,
                                    "media_type": source.media_type,
                                    "data": source.data
                                }
                            }),
                            _ => json!(null)
                        }
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        let response = self
            .client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("API error {}: {}", status, body));
        }

        let api_response: Value = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Parse content blocks
        let content_blocks = api_response
            .get("content")
            .and_then(|c| c.as_array())
            .ok_or("Missing content in response")?
            .iter()
            .map(|block| {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match block_type {
                    "text" => CUContentBlock::Text {
                        text: block.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                    },
                    "tool_use" => CUContentBlock::ToolUse {
                        id: block.get("id").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        name: block.get("name").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        input: block.get("input").cloned().unwrap_or(json!({})),
                    },
                    "thinking" => CUContentBlock::Thinking {
                        thinking: block.get("thinking").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                    },
                    _ => CUContentBlock::Text {
                        text: format!("Unknown block type: {}", block_type),
                    },
                }
            })
            .collect();

        Ok(CUMessage {
            role: "assistant".to_string(),
            content: content_blocks,
        })
    }

    fn check_hitl(&self, tool_name: &str, input: &Value, perm_level: &str) -> HITLAction {
        match tool_name {
            "computer" => {
                let action = input.get("action").and_then(|a| a.as_str()).unwrap_or("");
                match action {
                    "screenshot" | "mouse_move" => HITLAction::Allow,
                    "left_click" | "right_click" | "double_click" => {
                        if perm_level == "auto" {
                            HITLAction::Allow
                        } else {
                            HITLAction::Allow // Clicks are usually safe
                        }
                    }
                    "type" | "key" => {
                        // Typing in sensitive fields should be reviewed
                        if perm_level == "browse" {
                            HITLAction::AskUser
                        } else {
                            HITLAction::Allow
                        }
                    }
                    _ => HITLAction::Allow,
                }
            }
            "bash" => {
                let cmd = input.get("command").and_then(|c| c.as_str()).unwrap_or("");
                // Block dangerous commands
                if cmd.contains("rm -rf")
                    || cmd.contains("sudo")
                    || cmd.contains("curl | sh")
                    || cmd.contains("wget | sh")
                {
                    HITLAction::Block
                } else if perm_level == "browse" {
                    HITLAction::AskUser
                } else if perm_level == "standard" {
                    HITLAction::AskUser // Bash commands default to ask
                } else {
                    HITLAction::Allow // "auto" mode
                }
            }
            "text_editor" => {
                if perm_level == "browse" {
                    HITLAction::AskUser
                } else {
                    HITLAction::Allow
                }
            }
            _ => HITLAction::Allow,
        }
    }

    fn estimate_step_cost(&self, content: &[CUContentBlock]) -> f64 {
        // Rough estimate: 1 screenshot ≈ 1590 tokens ≈ $0.024 (Claude Sonnet)
        let mut cost = 0.0;
        for block in content {
            match block {
                CUContentBlock::Image { .. } => cost += 0.024,
                CUContentBlock::Text { text } => {
                    // Rough: 1 token ≈ 4 chars
                    let tokens = text.len() / 4;
                    cost += (tokens as f64) * 0.000003; // $3 per 1M tokens input
                }
                _ => {}
            }
        }
        cost
    }
}

// ─── Step Result ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CUStepResult {
    pub session_id: String,
    pub step: usize,
    pub response: CUMessage,
    pub paused: bool,
    pub cost: f64,
}

// ─── Tauri Commands ───

#[tauri::command]
pub async fn start_cu_session(
    agent_id: String,
    sandbox_id: String,
    vnc_port: u16,
    system_prompt: String,
    model: String,
    api_key: String,
    state: tauri::State<'_, CURuntime>,
) -> Result<String, String> {
    state
        .start_session(agent_id, sandbox_id, vnc_port, system_prompt, model, api_key)
        .await
}

#[tauri::command]
pub async fn cu_step(
    session_id: String,
    api_key: String,
    perm_level: String,
    state: tauri::State<'_, CURuntime>,
) -> Result<CUStepResult, String> {
    state.step(session_id, api_key, perm_level).await
}

#[tauri::command]
pub async fn pause_cu_session(
    session_id: String,
    state: tauri::State<'_, CURuntime>,
) -> Result<(), String> {
    state.pause(session_id).await
}

#[tauri::command]
pub async fn resume_cu_session(
    session_id: String,
    user_response: String,
    state: tauri::State<'_, CURuntime>,
) -> Result<(), String> {
    state.resume(session_id, user_response).await
}

#[tauri::command]
pub async fn stop_cu_session(
    session_id: String,
    state: tauri::State<'_, CURuntime>,
) -> Result<(), String> {
    state.stop(session_id).await
}
