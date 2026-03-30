// enterprise/auth_provider.rs - Authentication Provider Interface
// CE: Local API Key storage
// EE: SSO / SAML / OIDC

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// User authentication info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub teams: Vec<String>,
}

/// Auth provider trait
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Validate API key
    async fn validate_key(&self, provider: &str, key: &str) -> Result<bool, String>;
    
    /// Store API key securely
    async fn store_key(&self, provider: &str, key: &str) -> Result<(), String>;
    
    /// Retrieve API key
    async fn get_key(&self, provider: &str) -> Result<Option<String>, String>;
    
    /// Delete API key
    async fn delete_key(&self, provider: &str) -> Result<(), String>;
    
    /// Get current user info (EE: from SSO)
    async fn get_user(&self) -> Result<Option<UserInfo>, String>;
    
    /// Check if user has permission
    async fn has_permission(&self, permission: &str) -> Result<bool, String>;
}

/// Local API key auth provider (Community Edition)
pub struct LocalAuthProvider;

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    async fn validate_key(&self, provider: &str, key: &str) -> Result<bool, String> {
        // Simple validation - check if key is not empty and has correct prefix
        match provider {
            "anthropic" => Ok(key.starts_with("sk-ant-")),
            "openai" => Ok(key.starts_with("sk-")),
            _ => Ok(!key.is_empty()),
        }
    }
    
    async fn store_key(&self, _provider: &str, _key: &str) -> Result<(), String> {
        // Implemented in keychain.rs
        Ok(())
    }
    
    async fn get_key(&self, _provider: &str) -> Result<Option<String>, String> {
        Ok(None)
    }
    
    async fn delete_key(&self, _provider: &str) -> Result<(), String> {
        Ok(())
    }
    
    async fn get_user(&self) -> Result<Option<UserInfo>, String> {
        // CE: No user management
        Ok(None)
    }
    
    async fn has_permission(&self, _permission: &str) -> Result<bool, String> {
        // CE: All permissions granted locally
        Ok(true)
    }
}