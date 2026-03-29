// keychain.rs - OS-native credential storage
// macOS: Keychain, Windows: Credential Manager, Linux: Secret Service

use keyring::Entry;
use serde::Serialize;

const SERVICE_NAME: &str = "clawstudio";

#[derive(Serialize)]
pub struct KeyTestResult {
    pub valid: bool,
    pub model: String,
    pub message: String,
}

/// Store an API key in the OS keychain
#[tauri::command]
pub fn save_api_key(provider: &str, key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
    entry.set_password(key).map_err(|e| e.to_string())?;
    log::info!("API key saved for provider: {}", provider);
    Ok(())
}

/// Retrieve an API key from the OS keychain
#[tauri::command]
pub fn get_api_key(provider: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| format!("Key not found for {}: {}", provider, e))
}

/// Delete an API key from the OS keychain
#[tauri::command]
pub fn delete_api_key(provider: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, provider).map_err(|e| e.to_string())?;
    entry.delete_password().map_err(|e| e.to_string())?;
    log::info!("API key deleted for provider: {}", provider);
    Ok(())
}

/// Test an API key by making a lightweight request
#[tauri::command]
pub async fn test_api_key(provider: &str, key: &str) -> Result<KeyTestResult, String> {
    // TODO: Implement actual API validation calls
    // For now, basic format validation
    match provider {
        "claude" => {
            if key.starts_with("sk-ant-") && key.len() > 20 {
                Ok(KeyTestResult {
                    valid: true,
                    model: "Claude 3.5 Sonnet".into(),
                    message: "Key format valid".into(),
                })
            } else {
                Ok(KeyTestResult {
                    valid: false,
                    model: String::new(),
                    message: "Invalid key format: expected sk-ant-...".into(),
                })
            }
        }
        "openai" => {
            if key.starts_with("sk-") && key.len() > 20 {
                Ok(KeyTestResult {
                    valid: true,
                    model: "GPT-4o".into(),
                    message: "Key format valid".into(),
                })
            } else {
                Ok(KeyTestResult {
                    valid: false,
                    model: String::new(),
                    message: "Invalid key format: expected sk-...".into(),
                })
            }
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}
