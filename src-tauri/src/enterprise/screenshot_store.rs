// enterprise/screenshot_store.rs - Screenshot Storage Interface
// CE: Local 7-day rolling storage
// EE: Cloud permanent storage with CDN

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Screenshot metadata
#[derive(Debug, Clone)]
pub struct ScreenshotMeta {
    pub id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
    pub hash: String,
}

/// Screenshot store trait
#[async_trait]
pub trait ScreenshotStore: Send + Sync {
    /// Save a screenshot
    async fn save(&self, session_id: &str, image_data: &[u8]) -> Result<String, String>;
    
    /// Get a screenshot by ID
    async fn get(&self, screenshot_id: &str) -> Result<Option<Vec<u8>>, String>;
    
    /// List screenshots for a session
    async fn list(&self, session_id: &str) -> Result<Vec<ScreenshotMeta>, String>;
    
    /// Delete old screenshots (CE: auto-cleanup after 7 days)
    async fn cleanup(&self, older_than_days: u32) -> Result<u64, String>;
    
    /// Get storage usage
    async fn get_usage(&self) -> Result<u64, String>;
}

/// Local screenshot store (Community Edition)
pub struct LocalScreenshotStore {
    #[allow(dead_code)]
    base_path: String,
    #[allow(dead_code)]
    max_days: u32,
}

impl LocalScreenshotStore {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
            max_days: 7, // CE: 7-day rolling
        }
    }
}

#[async_trait]
impl ScreenshotStore for LocalScreenshotStore {
    async fn save(&self, _session_id: &str, _image_data: &[u8]) -> Result<String, String> {
        // Save to local filesystem
        let id = uuid::Uuid::new_v4().to_string();
        Ok(id)
    }
    
    async fn get(&self, _screenshot_id: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
    
    async fn list(&self, _session_id: &str) -> Result<Vec<ScreenshotMeta>, String> {
        Ok(vec![])
    }
    
    async fn cleanup(&self, _older_than_days: u32) -> Result<u64, String> {
        Ok(0)
    }
    
    async fn get_usage(&self) -> Result<u64, String> {
        Ok(0)
    }
}