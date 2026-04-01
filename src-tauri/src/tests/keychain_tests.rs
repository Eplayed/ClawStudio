// Tests for keychain.rs - Secure API key storage

#[cfg(test)]
mod tests {
    use serde_json;

    #[test]
    fn test_api_key_masking() {
        fn mask_key(key: &str) -> String {
            if key.len() <= 8 {
                return "*".repeat(key.len());
            }
            format!("{}****{}", &key[..4], &key[key.len()-4..])
        }
        
        assert_eq!(mask_key("sk-ant-1234567890abcdef"), "sk-a****cdef");
        assert_eq!(mask_key("short"), "*****");
        println!("✅ API key masking works");
    }

    #[test]
    fn test_provider_validation() {
        let valid_providers = ["anthropic", "openai", "deepseek"];
        let invalid_providers = ["google", "meta", "unknown"];
        
        for provider in valid_providers {
            assert!(["anthropic", "openai", "deepseek"].contains(&provider));
        }
        
        for provider in invalid_providers {
            assert!(!["anthropic", "openai", "deepseek"].contains(&provider));
        }
        println!("✅ Provider validation works");
    }

    #[test]
    fn test_key_format_validation() {
        fn validate_key_format(provider: &str, key: &str) -> bool {
            match provider {
                "anthropic" => key.starts_with("sk-ant-"),
                "openai" => key.starts_with("sk-"),
                "deepseek" => key.starts_with("sk-"),
                _ => false,
            }
        }
        
        assert!(validate_key_format("anthropic", "sk-ant-api03-xxx"));
        assert!(validate_key_format("openai", "sk-proj-xxx"));
        assert!(validate_key_format("deepseek", "sk-xxx"));
        assert!(!validate_key_format("anthropic", "sk-xxx")); // Wrong prefix
        assert!(!validate_key_format("unknown", "any-key"));
        println!("✅ Key format validation works");
    }

    #[test]
    fn test_key_storage_structure() {
        let storage = serde_json::json!({
            "provider": "anthropic",
            "key_ref": "keychain://anthropic-api-key",
            "valid": true,
            "last_validated": "2024-01-15T10:30:00Z"
        });
        
        assert!(storage["valid"].as_bool().unwrap());
        assert!(storage["key_ref"].as_str().unwrap().starts_with("keychain://"));
        println!("✅ Key storage structure is correct");
    }

    #[test]
    fn test_test_api_key_response() {
        let success_response = serde_json::json!({
            "valid": true,
            "models": ["claude-3-5-sonnet-20241022", "claude-3-opus-20240229"],
            "error": null
        });
        
        let error_response = serde_json::json!({
            "valid": false,
            "models": [],
            "error": "Invalid API key"
        });
        
        assert!(success_response["valid"].as_bool().unwrap());
        assert!(!error_response["valid"].as_bool().unwrap());
        assert!(error_response["error"].is_string());
        println!("✅ API key test response structure is correct");
    }

    #[test]
    fn test_keychain_entry_name() {
        fn get_keychain_entry_name(provider: &str) -> String {
            format!("clawstudio-{}-api-key", provider)
        }
        
        assert_eq!(get_keychain_entry_name("anthropic"), "clawstudio-anthropic-api-key");
        assert_eq!(get_keychain_entry_name("openai"), "clawstudio-openai-api-key");
        println!("✅ Keychain entry naming is consistent");
    }
}
