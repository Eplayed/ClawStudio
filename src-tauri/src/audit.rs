// audit.rs - Audit Logging and Compliance Module
// Implements Phase 4 of ClawStudio v2.0 roadmap

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub session_id: String,
    pub action_type: ActionType,
    pub action_detail: serde_json::Value,
    pub screenshot_hash: Option<String>,
    pub cost_usd: f64,
    pub hitl_approved: Option<bool>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Screenshot,
    MouseClick,
    MouseMove,
    KeyPress,
    BashExec,
    FileRead,
    FileWrite,
    AgentStart,
    AgentStop,
    HitlApproval,
    CostThreshold,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditFilter {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub action_type: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostSummary {
    pub today: f64,
    pub week: f64,
    pub month: f64,
    pub budget_remaining: f64,
}

// ─── Audit State ───

pub struct AuditState {
    db_path: PathBuf,
    entries: Arc<Mutex<Vec<AuditEntry>>>,
}

impl AuditState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub async fn log_audit_entry(
    entry: AuditEntry,
    state: tauri::State<'_, AuditState>,
) -> Result<String, String> {
    log::info!("Logging audit entry: {} - {:?}", entry.agent_id, entry.action_type);
    
    let mut entries = state.entries.lock().await;
    let id = entry.id.clone();
    entries.push(entry);
    
    // TODO: Persist to SQLite
    
    Ok(id)
}

#[tauri::command]
pub async fn get_audit_logs(
    filter: AuditFilter,
    state: tauri::State<'_, AuditState>,
) -> Result<Vec<AuditEntry>, String> {
    let entries = state.entries.lock().await;
    
    let mut filtered: Vec<AuditEntry> = entries
        .iter()
        .filter(|e| {
            if let Some(agent_id) = &filter.agent_id {
                if e.agent_id != *agent_id {
                    return false;
                }
            }
            if let Some(action_type) = &filter.action_type {
                if format!("{:?}", e.action_type) != *action_type {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();
    
    // Apply limit
    if let Some(limit) = filter.limit {
        filtered.truncate(limit);
    }
    
    Ok(filtered)
}

#[tauri::command]
pub async fn export_audit_logs(
    format: String,
    state: tauri::State<'_, AuditState>,
) -> Result<Vec<u8>, String> {
    let entries = state.entries.lock().await;
    
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&*entries)
                .map_err(|e| format!("Failed to serialize: {}", e))?;
            Ok(json.into_bytes())
        }
        "csv" => {
            let mut csv = String::from("id,timestamp,agent_id,action_type,cost_usd,hitl_approved\n");
            for entry in entries.iter() {
                csv.push_str(&format!(
                    "{},{},{},{:?},{},{}\n",
                    entry.id,
                    entry.timestamp.to_rfc3339(),
                    entry.agent_id,
                    entry.action_type,
                    entry.cost_usd,
                    entry.hitl_approved.map(|b| b.to_string()).unwrap_or_default()
                ));
            }
            Ok(csv.into_bytes())
        }
        _ => Err("Unsupported export format".to_string()),
    }
}

#[tauri::command]
pub async fn verify_audit_integrity(
    entry_id: String,
    state: tauri::State<'_, AuditState>,
) -> Result<bool, String> {
    let entries = state.entries.lock().await;
    
    if let Some(entry) = entries.iter().find(|e| e.id == entry_id) {
        // Verify hash
        let computed_hash = compute_hash(entry);
        Ok(computed_hash == entry.hash)
    } else {
        Err("Entry not found".to_string())
    }
}

#[tauri::command]
pub async fn get_cost_summary(
    state: tauri::State<'_, AuditState>,
) -> Result<CostSummary, String> {
    let entries = state.entries.lock().await;
    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let week_start = (now - chrono::Duration::days(7)).date_naive().and_hms_opt(0, 0, 0).unwrap();
    
    let mut today = 0.0;
    let mut week = 0.0;
    let mut month = 0.0;
    
    for entry in entries.iter() {
        let cost = entry.cost_usd;
        let entry_time = entry.timestamp.naive_utc();
        
        if entry_time >= today_start {
            today += cost;
        }
        if entry_time >= week_start {
            week += cost;
        }
        month += cost;
    }
    
    Ok(CostSummary {
        today,
        week,
        month,
        budget_remaining: 5.0 - today, // Default $5 budget
    })
}

#[tauri::command]
pub async fn cleanup_old_audit_logs(
    days: u32,
    state: tauri::State<'_, AuditState>,
) -> Result<u64, String> {
    let mut entries = state.entries.lock().await;
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);
    
    let original_len = entries.len();
    entries.retain(|e| e.timestamp > cutoff);
    let removed = original_len - entries.len();
    
    log::info!("Cleaned up {} old audit logs", removed);
    Ok(removed as u64)
}

// ─── Helper Functions ───

fn compute_hash(entry: &AuditEntry) -> String {
    use sha2::{Sha256, Digest};
    
    let data = format!(
        "{}{}{}{}{}{}",
        entry.timestamp,
        entry.agent_id,
        entry.session_id,
        serde_json::to_string(&entry.action_detail).unwrap_or_default(),
        entry.cost_usd,
        entry.action_detail.to_string()
    );
    
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ─── Convenience Functions ───

pub fn create_entry(
    agent_id: &str,
    session_id: &str,
    action_type: ActionType,
    action_detail: serde_json::Value,
    cost_usd: f64,
) -> AuditEntry {
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now();
    
    let mut entry = AuditEntry {
        id: id.clone(),
        timestamp,
        agent_id: agent_id.to_string(),
        session_id: session_id.to_string(),
        action_type,
        action_detail,
        screenshot_hash: None,
        cost_usd,
        hitl_approved: None,
        hash: String::new(),
    };
    
    entry.hash = compute_hash(&entry);
    entry
}