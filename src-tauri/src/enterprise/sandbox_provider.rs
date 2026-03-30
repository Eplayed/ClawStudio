// enterprise/sandbox_provider.rs - Sandbox Provider Interface
// CE: Local Docker
// EE: Cloud sandbox API

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub id: String,
    pub name: String,
    pub image: String,
    pub vnc_port: u16,
    pub memory_mb: u64,
    pub cpu_count: u64,
    pub environment: Vec<(String, String)>,
}

/// Sandbox status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStatus {
    pub id: String,
    pub status: SandboxState,
    pub vnc_url: Option<String>,
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxState {
    Running,
    Stopped,
    Paused,
    Error,
}

/// Sandbox provider trait
#[async_trait]
pub trait SandboxProvider: Send + Sync {
    /// Create a new sandbox
    async fn create(&self, config: SandboxConfig) -> Result<String, String>;
    
    /// Destroy a sandbox
    async fn destroy(&self, sandbox_id: &str) -> Result<(), String>;
    
    /// Get sandbox status
    async fn get_status(&self, sandbox_id: &str) -> Result<SandboxStatus, String>;
    
    /// List all sandboxes
    async fn list(&self) -> Result<Vec<SandboxStatus>, String>;
    
    /// Execute command in sandbox
    async fn exec(&self, sandbox_id: &str, command: &str) -> Result<String, String>;
    
    /// Take screenshot
    async fn screenshot(&self, sandbox_id: &str) -> Result<Vec<u8>, String>;
}

/// Local Docker sandbox provider (Community Edition)
pub struct LocalDockerProvider;

#[async_trait]
impl SandboxProvider for LocalDockerProvider {
    async fn create(&self, _config: SandboxConfig) -> Result<String, String> {
        // Implemented in docker.rs
        Ok("local-sandbox".to_string())
    }
    
    async fn destroy(&self, _sandbox_id: &str) -> Result<(), String> {
        Ok(())
    }
    
    async fn get_status(&self, _sandbox_id: &str) -> Result<SandboxStatus, String> {
        Err("Not implemented".to_string())
    }
    
    async fn list(&self) -> Result<Vec<SandboxStatus>, String> {
        Ok(vec![])
    }
    
    async fn exec(&self, _sandbox_id: &str, _command: &str) -> Result<String, String> {
        Ok("".to_string())
    }
    
    async fn screenshot(&self, _sandbox_id: &str) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}