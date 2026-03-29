// db.rs - SQLite database operations
// Local persistence for agents, traces, and settings
//
// Architecture note: Since tauri-plugin-sql executes queries from the frontend,
// we use parameterized query builders here that return (sql, params) tuples.
// The frontend passes these to the plugin-sql execute() method.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

// ─── Data Models ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub system_prompt: String,
    pub computer_use: bool,
    pub network_access: bool,
    pub sandbox_id: Option<String>,
    pub file_whitelist: String,
    pub budget_limit: f64,
    pub token_limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRecord {
    pub id: String,
    pub agent_id: String,
    pub task_name: String,
    pub status: String, // "success" | "failed" | "cancelled"
    pub steps: i32,
    pub cost: f64,
    pub duration_sec: i64,
    pub log_json: String,
}

/// Parameterized query result for frontend to execute via plugin-sql
#[derive(Debug, Clone, Serialize)]
pub struct QueryPlan {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}

// ─── Migrations ───

pub async fn run_migrations(_app: &AppHandle) -> Result<(), String> {
    log::info!("Database migrations ready (executed via frontend plugin-sql)");
    Ok(())
}

// ─── Tauri Commands ───

/// Return migration SQL for frontend to execute via plugin-sql
#[tauri::command]
pub async fn init_database() -> Result<Vec<QueryPlan>, String> {
    let plans = vec![
        QueryPlan {
            sql: "CREATE TABLE IF NOT EXISTS agents (id TEXT PRIMARY KEY, name TEXT NOT NULL, avatar TEXT DEFAULT '🤖', system_prompt TEXT DEFAULT '', computer_use INTEGER DEFAULT 0, network_access INTEGER DEFAULT 1, sandbox_id TEXT, file_whitelist TEXT DEFAULT '', budget_limit REAL DEFAULT 1.0, token_limit INTEGER DEFAULT 100000, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)".into(),
            params: vec![],
        },
        QueryPlan {
            sql: "CREATE TABLE IF NOT EXISTS traces (id TEXT PRIMARY KEY, agent_id TEXT NOT NULL, task_name TEXT NOT NULL, status TEXT DEFAULT 'running', steps INTEGER DEFAULT 0, cost REAL DEFAULT 0.0, duration_sec INTEGER DEFAULT 0, log_json TEXT DEFAULT '[]', created_at DATETIME DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (agent_id) REFERENCES agents(id))".into(),
            params: vec![],
        },
        QueryPlan {
            sql: "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at DATETIME DEFAULT CURRENT_TIMESTAMP)".into(),
            params: vec![],
        },
        QueryPlan {
            sql: "CREATE TABLE IF NOT EXISTS sandbox_instances (id TEXT PRIMARY KEY, name TEXT NOT NULL, image TEXT NOT NULL, vnc_port INTEGER, status TEXT DEFAULT 'stopped', container_id TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)".into(),
            params: vec![],
        },
        // Default settings
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["default_model".into(), "claude-3-5-sonnet-20241022".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["cu_model".into(), "claude-3-5-sonnet-20241022".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["temperature".into(), "0.2".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["max_tokens".into(), "4096".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["hitl_enabled".into(), "true".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["perm_level".into(), "standard".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["budget_default".into(), "1.0".into()] },
        QueryPlan { sql: "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)".into(), params: vec!["zero_telemetry".into(), "true".into()] },
    ];
    Ok(plans)
}

/// Return SELECT query for agents
#[tauri::command]
pub async fn get_all_agents() -> Result<QueryPlan, String> {
    Ok(QueryPlan {
        sql: "SELECT * FROM agents ORDER BY created_at DESC".into(),
        params: vec![],
    })
}

/// Return parameterized INSERT for agent config (safe from SQL injection)
#[tauri::command]
pub async fn save_agent_config(agent: AgentConfig) -> Result<QueryPlan, String> {
    Ok(QueryPlan {
        sql: "INSERT OR REPLACE INTO agents (id, name, avatar, system_prompt, computer_use, network_access, sandbox_id, file_whitelist, budget_limit, token_limit) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".into(),
        params: vec![
            agent.id.into(),
            agent.name.into(),
            agent.avatar.into(),
            agent.system_prompt.into(),
            serde_json::Value::from(agent.computer_use as i32),
            serde_json::Value::from(agent.network_access as i32),
            agent.sandbox_id.map_or(serde_json::Value::Null, |s| s.into()),
            agent.file_whitelist.into(),
            serde_json::Value::from(agent.budget_limit),
            serde_json::Value::from(agent.token_limit),
        ],
    })
}

/// Return SELECT query for traces
#[tauri::command]
pub async fn get_all_traces() -> Result<QueryPlan, String> {
    Ok(QueryPlan {
        sql: "SELECT * FROM traces ORDER BY created_at DESC LIMIT 100".into(),
        params: vec![],
    })
}

/// Return parameterized INSERT for trace record
#[tauri::command]
pub async fn save_trace(trace: TraceRecord) -> Result<QueryPlan, String> {
    Ok(QueryPlan {
        sql: "INSERT INTO traces (id, agent_id, task_name, status, steps, cost, duration_sec, log_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?)".into(),
        params: vec![
            trace.id.into(),
            trace.agent_id.into(),
            trace.task_name.into(),
            trace.status.into(),
            serde_json::Value::from(trace.steps),
            serde_json::Value::from(trace.cost),
            serde_json::Value::from(trace.duration_sec),
            trace.log_json.into(),
        ],
    })
}
