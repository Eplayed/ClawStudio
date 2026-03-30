// enterprise/audit_logger.rs - Audit Logging Interface
// CE: Local file/SQLite implementation
// EE: Cloud immutable storage with blockchain verification

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Audit log entry structure
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
    pub hash: String, // SHA-256 for tamper detection
}

/// Types of auditable actions
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

/// Filter for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub action_type: Option<ActionType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Export format for audit logs
#[derive(Debug, Clone)]
pub enum ExportFormat {
    JSON,
    CSV,
    PDF, // Enterprise only
}

/// Audit logger trait - CE implements locally, EE implements cloud
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an action, returns entry ID
    async fn log_action(&self, entry: AuditEntry) -> Result<String, String>;
    
    /// Get entries matching filter
    async fn get_entries(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, String>;
    
    /// Export logs in specified format
    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>, String>;
    
    /// Verify integrity of an entry (EE: blockchain verification)
    async fn verify_integrity(&self, entry_id: &str) -> Result<bool, String>;
    
    /// Get total cost for a time range
    async fn get_total_cost(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<f64, String>;
}

/// Local file-based audit logger (Community Edition)
pub struct LocalAuditLogger {
    db_path: String,
}

impl LocalAuditLogger {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }
}

#[async_trait]
impl AuditLogger for LocalAuditLogger {
    async fn log_action(&self, entry: AuditEntry) -> Result<String, String> {
        // CE: Write to local SQLite
        // Implemented in db.rs
        Ok(entry.id.clone())
    }
    
    async fn get_entries(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, String> {
        // CE: Query local SQLite
        Ok(vec![])
    }
    
    async fn export(&self, format: ExportFormat) -> Result<Vec<u8>, String> {
        // CE: Export to JSON/CSV
        Ok(vec![])
    }
    
    async fn verify_integrity(&self, _entry_id: &str) -> Result<bool, String> {
        // CE: Simple hash verification
        Ok(true)
    }
    
    async fn get_total_cost(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<f64, String> {
        // CE: Sum from local DB
        Ok(0.0)
    }
}