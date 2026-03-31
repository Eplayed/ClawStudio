// channels.rs - Channel management for Telegram/Discord/WeChat/Slack
// Handles adding, removing, and testing message channel connections

use serde::{Deserialize, Serialize};
use std::process::Command as StdCommand;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub channel_type: String,  // "telegram" | "discord" | "wechat" | "slack"
    pub name: String,
    pub connected: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel_type: String,
    pub bot_token: Option<String>,
    pub api_key: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTestResult {
    pub success: bool,
    pub message: String,
}

// ─── Helper Functions ───

fn get_config_path() -> std::path::PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".openclaw").join("openclaw.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("openclaw.json"))
}

fn read_openclaw_config() -> Result<serde_json::Value, String> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
}

fn write_openclaw_config(config: &serde_json::Value) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

// ─── Tauri Commands ───

/// List all configured channels
#[tauri::command]
pub async fn list_channels() -> Result<Vec<ChannelInfo>, String> {
    let config = read_openclaw_config()?;
    
    let mut channels = Vec::new();
    
    // Check Telegram
    if let Some(telegram) = config.get("channels").and_then(|c| c.get("telegram")) {
        channels.push(ChannelInfo {
            id: "telegram".to_string(),
            channel_type: "telegram".to_string(),
            name: "Telegram".to_string(),
            connected: telegram.get("bot_token").is_some(),
            config: telegram.clone(),
        });
    }
    
    // Check Discord
    if let Some(discord) = config.get("channels").and_then(|c| c.get("discord")) {
        channels.push(ChannelInfo {
            id: "discord".to_string(),
            channel_type: "discord".to_string(),
            name: "Discord".to_string(),
            connected: discord.get("bot_token").is_some(),
            config: discord.clone(),
        });
    }
    
    // Check WeChat (placeholder - requires enterprise wechat)
    channels.push(ChannelInfo {
        id: "wechat".to_string(),
        channel_type: "wechat".to_string(),
        name: "WeChat".to_string(),
        connected: false,
        config: serde_json::json!({}),
    });
    
    // Check Slack
    if let Some(slack) = config.get("channels").and_then(|c| c.get("slack")) {
        channels.push(ChannelInfo {
            id: "slack".to_string(),
            channel_type: "slack".to_string(),
            name: "Slack".to_string(),
            connected: slack.get("bot_token").is_some(),
            config: slack.clone(),
        });
    }
    
    Ok(channels)
}

/// Add or update a channel configuration
#[tauri::command]
pub async fn add_channel(config: ChannelConfig) -> Result<(), String> {
    let mut openclaw_config = read_openclaw_config()?;
    
    // Ensure channels object exists
    if !openclaw_config.get("channels").is_some() {
        openclaw_config["channels"] = serde_json::json!({});
    }
    
    let channels = openclaw_config["channels"].as_object_mut()
        .ok_or("Failed to get channels object")?;
    
    let channel_config = serde_json::json!({
        "enabled": true,
        "bot_token": config.bot_token,
        "api_key": config.api_key,
        "channel_id": config.channel_id,
    });
    
    channels.insert(config.channel_type.clone(), channel_config);
    
    write_openclaw_config(&openclaw_config)?;
    
    log::info!("Channel {} added/updated", config.channel_type);
    Ok(())
}

/// Remove a channel configuration
#[tauri::command]
pub async fn remove_channel(channel_type: String) -> Result<(), String> {
    let mut openclaw_config = read_openclaw_config()?;
    
    if let Some(channels) = openclaw_config.get_mut("channels") {
        if let Some(obj) = channels.as_object_mut() {
            obj.remove(&channel_type);
        }
    }
    
    write_openclaw_config(&openclaw_config)?;
    
    log::info!("Channel {} removed", channel_type);
    Ok(())
}

/// Test a channel connection
#[tauri::command]
pub async fn test_channel(channel_type: String, config: ChannelConfig) -> Result<ChannelTestResult, String> {
    match channel_type.as_str() {
        "telegram" => {
            if let Some(token) = &config.bot_token {
                // Try to get bot info via Telegram API
                let url = format!("https://api.telegram.org/bot{}/getMe", token);
                let response = reqwest::get(&url).await;
                
                match response {
                    Ok(resp) if resp.status().is_success() => {
                        Ok(ChannelTestResult {
                            success: true,
                            message: "Telegram Bot 连接成功!".to_string(),
                        })
                    }
                    Ok(resp) => {
                        Ok(ChannelTestResult {
                            success: false,
                            message: format!("Telegram API 错误: HTTP {}", resp.status()),
                        })
                    }
                    Err(e) => {
                        Ok(ChannelTestResult {
                            success: false,
                            message: format!("连接失败: {}", e),
                        })
                    }
                }
            } else {
                Ok(ChannelTestResult {
                    success: false,
                    message: "请提供 Bot Token".to_string(),
                })
            }
        }
        "discord" => {
            if let Some(token) = &config.bot_token {
                // Try to get bot info via Discord API
                let url = "https://discord.com/api/v10/users/@me";
                let response = reqwest::Client::new()
                    .get(url)
                    .header("Authorization", format!("Bot {}", token))
                    .send()
                    .await;
                
                match response {
                    Ok(resp) if resp.status().is_success() => {
                        Ok(ChannelTestResult {
                            success: true,
                            message: "Discord Bot 连接成功!".to_string(),
                        })
                    }
                    Ok(resp) => {
                        Ok(ChannelTestResult {
                            success: false,
                            message: format!("Discord API 错误: HTTP {}", resp.status()),
                        })
                    }
                    Err(e) => {
                        Ok(ChannelTestResult {
                            success: false,
                            message: format!("连接失败: {}", e),
                        })
                    }
                }
            } else {
                Ok(ChannelTestResult {
                    success: false,
                    message: "请提供 Bot Token".to_string(),
                })
            }
        }
        "slack" => {
            if let Some(token) = &config.bot_token {
                // Try to get bot info via Slack API
                let url = "https://slack.com/api/auth.test";
                let response = reqwest::Client::new()
                    .post(url)
                    .header("Authorization", format!("Bearer {}", token))
                    .form(&[("token", token)])
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => {
                        let body = resp.text().await.unwrap_or_default();
                        if body.contains("\"ok\":true") {
                            Ok(ChannelTestResult {
                                success: true,
                                message: "Slack App 连接成功!".to_string(),
                            })
                        } else {
                            Ok(ChannelTestResult {
                                success: false,
                                message: "Slack 认证失败".to_string(),
                            })
                        }
                    }
                    Err(e) => {
                        Ok(ChannelTestResult {
                            success: false,
                            message: format!("连接失败: {}", e),
                        })
                    }
                }
            } else {
                Ok(ChannelTestResult {
                    success: false,
                    message: "请提供 Bot Token".to_string(),
                })
            }
        }
        "wechat" => {
            Ok(ChannelTestResult {
                success: false,
                message: "WeChat 连接需要企业微信账号，请在设置中手动配置".to_string(),
            })
        }
        _ => Err(format!("Unknown channel type: {}", channel_type)),
    }
}

/// Restart Gateway after channel changes
#[tauri::command]
pub async fn restart_gateway_for_channels() -> Result<(), String> {
    // Run gateway restart command
    let output = StdCommand::new("openclaw")
        .args(["gateway", "restart"])
        .output();
    
    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => Err(format!("Failed to restart gateway: {}", e)),
    }
}
