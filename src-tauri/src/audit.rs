// audit.rs - Audit logging for compliance and traceability
// Simplified version without trait objects

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Screenshot,
    MouseClick,
    MouseMove,
    KeyPress,
    KeyType,
    BashExec,
    FileRead,
    FileWrite,
    FileDelete,
    ApiCall,
    HitlApprove,
    HitlReject,
    SessionStart,
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub agent_id: String,
    pub session_id: String,
    pub action_type: ActionType,
    pub action_detail: serde_json::Value,
    pub screenshot_hash: Option<String>,
    pub cost_usd: f64,
    pub tokens_used: u64,
    pub hitl_approved: Option<bool>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub action_type: Option<ActionType>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub sessions_count: u64,
    pub actions_by_type: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Jsonl,
}

// ─── Audit Manager (Simplified) ───

pub struct AuditManager {
    db_path: PathBuf,
}

impl AuditManager {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".clawstudio").join("audit.db"))
            .unwrap_or_else(|| PathBuf::from("./audit.db"));

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        Self { db_path }
    }

    fn get_connection(&self) -> Result<rusqlite::Connection, String> {
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open audit database: {}", e))?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                action_detail TEXT NOT NULL,
                screenshot_hash TEXT,
                cost_usd REAL DEFAULT 0,
                tokens_used INTEGER DEFAULT 0,
                hitl_approved INTEGER,
                duration_ms INTEGER DEFAULT 0
            )",
            [],
        ).map_err(|e| format!("Failed to create audit table: {}", e))?;

        Ok(conn)
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tauri Commands ───

#[tauri::command]
pub async fn audit_log(
    state: tauri::State<'_, AuditManager>,
    entry: AuditEntry,
) -> Result<String, String> {
    let conn = state.get_connection()?;

    let id = entry.id.clone();
    let action_type = serde_json::to_string(&entry.action_type).unwrap_or_default();
    let action_detail = serde_json::to_string(&entry.action_detail).unwrap_or_default();

    conn.execute(
        "INSERT INTO audit_logs (id, timestamp, agent_id, session_id, action_type, action_detail, screenshot_hash, cost_usd, tokens_used, hitl_approved, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            entry.id,
            entry.timestamp,
            entry.agent_id,
            entry.session_id,
            action_type,
            action_detail,
            entry.screenshot_hash,
            entry.cost_usd,
            entry.tokens_used,
            entry.hitl_approved.map(|b| b as i32),
            entry.duration_ms,
        ],
    ).map_err(|e| format!("Failed to insert audit log: {}", e))?;

    Ok(id)
}

#[tauri::command]
pub async fn audit_get_entries(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
) -> Result<Vec<AuditEntry>, String> {
    let conn = state.get_connection()?;

    let limit = filter.limit.max(100);
    let mut sql = String::from("SELECT id, timestamp, agent_id, session_id, action_type, action_detail, screenshot_hash, cost_usd, tokens_used, hitl_approved, duration_ms FROM audit_logs WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(agent_id) = &filter.agent_id {
        sql.push_str(" AND agent_id = ?");
        params.push(agent_id.clone());
    }

    if let Some(session_id) = &filter.session_id {
        sql.push_str(" AND session_id = ?");
        params.push(session_id.clone());
    }

    sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
    
    let mut stmt = conn.prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).chain(std::iter::once(&limit as &dyn rusqlite::ToSql)).collect();

    let entries = stmt.query_map(&params_refs[..], |row| {
        let action_type_str: String = row.get(4)?;
        let action_detail_str: String = row.get(5)?;
        let hitl_approved: Option<i32> = row.get(9)?;

        Ok(AuditEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            agent_id: row.get(2)?,
            session_id: row.get(3)?,
            action_type: serde_json::from_str(&action_type_str).unwrap_or(ActionType::ApiCall),
            action_detail: serde_json::from_str(&action_detail_str).unwrap_or(serde_json::json!({})),
            screenshot_hash: row.get(6)?,
            cost_usd: row.get(7)?,
            tokens_used: row.get(8)?,
            hitl_approved: hitl_approved.map(|b| b != 0),
            duration_ms: row.get(10)?,
        })
    })
    .map_err(|e| format!("Failed to query entries: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect entries: {}", e))?;

    Ok(entries)
}

#[tauri::command]
pub async fn audit_get_stats(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
) -> Result<AuditStats, String> {
    let entries = audit_get_entries(state, AuditFilter { limit: 10000, ..filter }).await?;

    let mut actions_by_type = std::collections::HashMap::new();
    let mut sessions = std::collections::HashSet::new();
    let mut total_cost = 0.0;
    let mut total_tokens = 0u64;

    for entry in &entries {
        let type_str = format!("{:?}", entry.action_type);
        *actions_by_type.entry(type_str).or_insert(0) += 1;
        sessions.insert(entry.session_id.clone());
        total_cost += entry.cost_usd;
        total_tokens += entry.tokens_used;
    }

    Ok(AuditStats {
        total_entries: entries.len() as u64,
        total_cost_usd: total_cost,
        total_tokens,
        sessions_count: sessions.len() as u64,
        actions_by_type,
    })
}

#[tauri::command]
pub async fn audit_export(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
    format: ExportFormat,
) -> Result<Vec<u8>, String> {
    let entries = audit_get_entries(state, filter).await?;

    match format {
        ExportFormat::Json => {
            serde_json::to_vec_pretty(&entries)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))
        }
        ExportFormat::Jsonl => {
            let lines: Vec<String> = entries
                .iter()
                .map(|e| serde_json::to_string(e).unwrap_or_default())
                .collect();
            Ok(lines.join("\n").into_bytes())
        }
        ExportFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            for entry in &entries {
                wtr.serialize(entry)
                    .map_err(|e| format!("Failed to write CSV: {}", e))?;
            }
            wtr.into_inner()
                .map_err(|e| format!("Failed to finalize CSV: {}", e))
        }
    }
}

#[tauri::command]
pub async fn audit_cleanup(
    state: tauri::State<'_, AuditManager>,
    older_than_days: u32,
) -> Result<u64, String> {
    let conn = state.get_connection()?;

    let cutoff = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);
    let cutoff_str = cutoff.to_rfc3339();

    let affected = conn.execute(
        "DELETE FROM audit_logs WHERE timestamp < ?",
        [&cutoff_str],
    ).map_err(|e| format!("Failed to cleanup audit logs: {}", e))?;

    Ok(affected as u64)
}

// Aliases for main.rs
#[tauri::command]
pub async fn get_cost_summary(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
) -> Result<AuditStats, String> {
    audit_get_stats(state, filter).await
}

#[tauri::command]
pub async fn export_audit_logs(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
    format: ExportFormat,
) -> Result<Vec<u8>, String> {
    audit_export(state, filter, format).await
}

#[tauri::command]
pub async fn get_audit_logs(
    state: tauri::State<'_, AuditManager>,
    filter: AuditFilter,
) -> Result<Vec<AuditEntry>, String> {
    audit_get_entries(state, filter).await
}
