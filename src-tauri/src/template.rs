// template.rs - Agent Template Export/Import
// Implements Phase 5 of ClawStudio v2.0 roadmap
// Supports .claw-template format and claw:// deep links

use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

// ─── Template Data Model ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub schema: String,              // "claw-template/v1"
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub system_prompt: String,
    pub model: String,
    pub computer_use: bool,
    pub sandbox_image: Option<String>,
    pub hitl_level: String,          // "browse" | "standard" | "auto"
    pub tags: Vec<String>,
    pub channels: Vec<String>,       // Enabled channels
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub budget_limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,   // Base64 preview image
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub verified: bool,
    pub downloads: u32,
    pub rating: f32,
}

// ─── Tauri Commands ───

#[tauri::command]
pub fn export_template(
    name: String,
    description: String,
    system_prompt: String,
    model: String,
    computer_use: bool,
    sandbox_image: Option<String>,
    hitl_level: String,
    tags: Vec<String>,
    channels: Vec<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    budget_limit: Option<f64>,
) -> Result<String, String> {
    log::info!("Exporting agent template: {}", name);
    
    let template = AgentTemplate {
        schema: "claw-template/v1".to_string(),
        name,
        description,
        author: "local-user".to_string(),
        version: "1.0.0".to_string(),
        created_at: Utc::now(),
        system_prompt,
        model,
        computer_use,
        sandbox_image,
        hitl_level,
        tags,
        channels,
        max_tokens,
        temperature,
        budget_limit,
        screenshot: None,
    };
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Failed to serialize template: {}", e))?;
    
    // Encode to Base64
    let encoded = general_purpose::STANDARD.encode(json.as_bytes());
    
    // Create deep link
    let deep_link = format!("claw://template/{}", encoded);
    
    Ok(deep_link)
}

#[tauri::command]
pub fn export_template_file(
    template: AgentTemplate,
    path: String,
) -> Result<String, String> {
    log::info!("Exporting template to file: {}", path);
    
    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Failed to serialize template: {}", e))?;
    
    let file_path = PathBuf::from(&path);
    let file_path = if file_path.extension().is_none() {
        file_path.with_extension("claw-template")
    } else {
        file_path
    };
    
    std::fs::write(&file_path, json)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn import_template(
    data: String,
) -> Result<AgentTemplate, String> {
    log::info!("Importing template from data");
    
    // Check if it's a deep link
    let json_str = if data.starts_with("claw://template/") {
        let encoded = data.strip_prefix("claw://template/").unwrap();
        let decoded = general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        String::from_utf8(decoded)
            .map_err(|e| format!("Failed to decode UTF-8: {}", e))?
    } else if data.starts_with("{") {
        // Raw JSON
        data
    } else {
        // Assume base64 encoded
        let decoded = general_purpose::STANDARD
            .decode(&data)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        String::from_utf8(decoded)
            .map_err(|e| format!("Failed to decode UTF-8: {}", e))?
    };
    
    // Parse JSON
    let template: AgentTemplate = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse template: {}", e))?;
    
    // Validate schema
    if template.schema != "claw-template/v1" {
        return Err(format!("Unsupported template schema: {}", template.schema));
    }
    
    Ok(template)
}

#[tauri::command]
pub fn import_template_file(
    path: String,
) -> Result<AgentTemplate, String> {
    log::info!("Importing template from file: {}", path);
    
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Check if it's base64 encoded
    let json_str = if content.starts_with("{") {
        content
    } else {
        let decoded = general_purpose::STANDARD
            .decode(&content.trim())
            .map_err(|e| format!("Failed to decode base64: {}", e))?;
        String::from_utf8(decoded)
            .map_err(|e| format!("Failed to decode UTF-8: {}", e))?
    };
    
    let template: AgentTemplate = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse template: {}", e))?;
    
    if template.schema != "claw-template/v1" {
        return Err(format!("Unsupported template schema: {}", template.schema));
    }
    
    Ok(template)
}

#[tauri::command]
pub fn generate_share_link(
    template: AgentTemplate,
) -> Result<String, String> {
    log::info!("Generating share link for template: {}", template.name);
    
    let json = serde_json::to_string(&template)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    
    let encoded = general_purpose::URL_SAFE.encode(json.as_bytes());
    
    Ok(format!("claw://template/{}", encoded))
}

#[tauri::command]
pub fn validate_template(
    template: AgentTemplate,
) -> Result<Vec<String>, String> {
    log::info!("Validating template: {}", template.name);
    
    let mut warnings = Vec::new();
    
    // Check required fields
    if template.system_prompt.is_empty() {
        warnings.push("System prompt is empty".to_string());
    }
    
    if template.model.is_empty() {
        warnings.push("Model is not specified".to_string());
    }
    
    // Check HITL level
    if !["browse", "standard", "auto"].contains(&template.hitl_level.as_str()) {
        warnings.push(format!("Unknown HITL level: {}", template.hitl_level));
    }
    
    // Check sandbox for computer use
    if template.computer_use && template.sandbox_image.is_none() {
        warnings.push("Computer use enabled but no sandbox image specified".to_string());
    }
    
    Ok(warnings)
}

// ─── Built-in Templates ───

pub fn get_builtin_templates() -> Vec<AgentTemplate> {
    vec![
        AgentTemplate {
            schema: "claw-template/v1".to_string(),
            name: "General Assistant".to_string(),
            description: "A general-purpose AI assistant for everyday tasks".to_string(),
            author: "ClawStudio".to_string(),
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            system_prompt: "You are a helpful AI assistant. Be concise and accurate in your responses.".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            computer_use: false,
            sandbox_image: None,
            hitl_level: "standard".to_string(),
            tags: vec!["general".to_string(), "assistant".to_string()],
            channels: vec![],
            max_tokens: Some(4096),
            temperature: Some(0.7),
            budget_limit: Some(5.0),
            screenshot: None,
        },
        AgentTemplate {
            schema: "claw-template/v1".to_string(),
            name: "Invoice Processor".to_string(),
            description: "Automatically extract and organize invoice data from PDFs".to_string(),
            author: "ClawStudio".to_string(),
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            system_prompt: "You are an invoice processing assistant. Extract key information from invoices including date, vendor, items, and total amount. Format the data as structured JSON.".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            computer_use: true,
            sandbox_image: Some("dorowu/ubuntu-desktop-lxde-vnc:focal".to_string()),
            hitl_level: "standard".to_string(),
            tags: vec!["finance".to_string(), "automation".to_string(), "pdf".to_string()],
            channels: vec![],
            max_tokens: Some(8192),
            temperature: Some(0.3),
            budget_limit: Some(10.0),
            screenshot: None,
        },
        AgentTemplate {
            schema: "claw-template/v1".to_string(),
            name: "Competitor Monitor".to_string(),
            description: "Monitor competitor websites for changes and generate reports".to_string(),
            author: "ClawStudio".to_string(),
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            system_prompt: "You are a competitive intelligence assistant. Monitor specified websites for changes in pricing, features, or announcements. Summarize findings in a structured report.".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            computer_use: true,
            sandbox_image: Some("dorowu/ubuntu-desktop-lxde-vnc:focal".to_string()),
            hitl_level: "browse".to_string(),
            tags: vec!["monitoring".to_string(), "research".to_string(), "automation".to_string()],
            channels: vec!["telegram".to_string()],
            max_tokens: Some(4096),
            temperature: Some(0.5),
            budget_limit: Some(5.0),
            screenshot: None,
        },
    ]
}

#[tauri::command]
pub fn list_builtin_templates() -> Result<Vec<TemplateMeta>, String> {
    let templates = get_builtin_templates();
    
    Ok(templates.iter().map(|t| TemplateMeta {
        id: uuid::Uuid::new_v4().to_string(),
        name: t.name.clone(),
        author: t.author.clone(),
        version: t.version.clone(),
        description: t.description.clone(),
        tags: t.tags.clone(),
        verified: t.author == "ClawStudio",
        downloads: 0,
        rating: 4.5,
    }).collect())
}

#[tauri::command]
pub fn get_builtin_template(name: String) -> Result<AgentTemplate, String> {
    let templates = get_builtin_templates();
    
    templates
        .into_iter()
        .find(|t| t.name == name)
        .ok_or_else(|| format!("Template not found: {}", name))
}