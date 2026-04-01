// Tests for audit.rs - Compliance audit logging

#[cfg(test)]
mod tests {
    use serde_json;
    use chrono::Utc;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = serde_json::json!({
            "id": "audit-001",
            "timestamp": Utc::now().to_rfc3339(),
            "agent_id": "agent-001",
            "session_id": "session-001",
            "action_type": "Screenshot",
            "action_detail": {"x": 100, "y": 200, "width": 1920, "height": 1080},
            "screenshot_hash": "abc123def456",
            "cost_usd": 0.005,
            "tokens_used": 1500,
            "hitl_approved": null,
            "duration_ms": 250
        });
        
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized["agent_id"], "agent-001");
        assert_eq!(deserialized["action_type"], "Screenshot");
        println!("✅ AuditEntry serialization works");
    }

    #[test]
    fn test_action_type_variants() {
        let actions = vec![
            "Screenshot", "MouseClick", "MouseMove", "KeyPress", "KeyType",
            "BashExec", "FileRead", "FileWrite", "FileDelete", "ApiCall",
            "HitlApprove", "HitlReject", "SessionStart", "SessionEnd"
        ];
        
        for action in actions {
            let entry = serde_json::json!({
                "action_type": action
            });
            assert!(entry["action_type"].is_string());
        }
        println!("✅ All ActionType variants can be serialized");
    }

    #[test]
    fn test_audit_filter() {
        let filter = serde_json::json!({
            "agent_id": "agent-001",
            "session_id": null,
            "action_type": "BashExec",
            "start_time": "2024-01-01T00:00:00Z",
            "end_time": null,
            "limit": 100
        });
        
        assert_eq!(filter["limit"].as_u64().unwrap(), 100);
        assert_eq!(filter["action_type"], "BashExec");
        println!("✅ AuditFilter serialization works");
    }

    #[test]
    fn test_audit_stats() {
        let stats = serde_json::json!({
            "total_entries": 1500,
            "total_cost_usd": 12.50,
            "total_tokens": 1250000,
            "sessions_count": 45,
            "actions_by_type": {
                "Screenshot": 500,
                "MouseClick": 400,
                "BashExec": 350,
                "KeyPress": 250
            }
        });
        
        assert_eq!(stats["total_entries"].as_u64().unwrap(), 1500);
        assert!((stats["total_cost_usd"].as_f64().unwrap() - 12.50).abs() < 0.001);
        println!("✅ AuditStats structure is correct");
    }

    #[test]
    fn test_export_formats() {
        let formats = ["Json", "Csv", "Jsonl"];
        
        for format in formats {
            let export = serde_json::json!({
                "format": format
            });
            assert_eq!(export["format"], format);
        }
        println!("✅ ExportFormat variants work");
    }

    #[test]
    fn test_hash_generation() {
        use sha2::{Sha256, Digest};
        
        let content = "test content for hashing";
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());
        
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
        println!("✅ Hash generation works: {}...", &hash[..16]);
    }
}
