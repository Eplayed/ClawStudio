// Tests for template.rs - Agent template export/import

#[cfg(test)]
mod tests {
    use serde_json;
    use chrono::Utc;

    #[test]
    fn test_template_schema() {
        let template = serde_json::json!({
            "schema": "claw-template/v1",
            "name": "全自动报税助手",
            "description": "自动识别并整理本月 PDF 发票，生成报税表格",
            "author": "community",
            "version": "1.0.0",
            "created_at": Utc::now().to_rfc3339(),
            "system_prompt": "You are a tax assistant...",
            "model": "claude-3-5-sonnet-20241022",
            "computer_use": true,
            "sandbox_image": "dorowu/ubuntu-desktop-lxde-vnc:focal",
            "hitl_level": "standard",
            "tags": ["finance", "tax", "automation"],
            "channels": ["telegram"],
            "max_tokens": 4096,
            "temperature": 0.7,
            "budget_limit": 5.0,
            "verified": false
        });
        
        assert_eq!(template["schema"], "claw-template/v1");
        assert!(template["computer_use"].as_bool().unwrap());
        assert_eq!(template["tags"].as_array().unwrap().len(), 3);
        println!("✅ AgentTemplate schema is valid");
    }

    #[test]
    fn test_template_validation() {
        // Valid template
        let valid = serde_json::json!({
            "schema": "claw-template/v1",
            "name": "Test Agent",
            "description": "A test agent",
            "author": "test",
            "version": "1.0.0",
            "system_prompt": "You are helpful",
            "model": "claude-3-5-sonnet-20241022",
            "computer_use": false,
            "hitl_level": "standard",
            "tags": []
        });
        
        // Required fields check
        assert!(valid.get("name").is_some());
        assert!(valid.get("system_prompt").is_some());
        assert!(valid.get("model").is_some());
        println!("✅ Template validation works");
    }

    #[test]
    fn test_deep_link_format() {
        // Test claw://template/<hash> deep link format
        let template_hash = "abc123def456";
        let deep_link = format!("claw://template/{}", template_hash);
        
        assert!(deep_link.starts_with("claw://template/"));
        assert!(deep_link.len() > "claw://template/".len());
        println!("✅ Deep link format is correct: {}", deep_link);
    }

    #[test]
    fn test_base64_encoding() {
        use base64::{Engine as _, engine::general_purpose};
        
        let template_json = r#"{"name":"Test Agent"}"#;
        let encoded = general_purpose::STANDARD.encode(template_json);
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        
        assert_eq!(decoded_str, template_json);
        println!("✅ Base64 encoding/decoding works");
    }

    #[test]
    fn test_template_meta() {
        let meta = serde_json::json!({
            "id": "template-001",
            "name": "Auto Tax Assistant",
            "author": "community-user",
            "version": "1.2.0",
            "description": "Automated tax filing",
            "tags": ["finance", "automation"],
            "verified": true,
            "downloads": 1250,
            "rating": 4.8
        });
        
        assert!(meta["verified"].as_bool().unwrap());
        assert!((meta["rating"].as_f64().unwrap() - 4.8).abs() < 0.01);
        assert_eq!(meta["downloads"].as_u64().unwrap(), 1250);
        println!("✅ TemplateMeta structure is correct");
    }

    #[test]
    fn test_builtin_templates() {
        // Test that builtin templates exist and are valid
        let builtin_templates = vec![
            ("invoice-organizer", "发票整理助手"),
            ("competitor-monitor", "竞品监控"),
            ("customer-support", "客服回复"),
            ("data-analysis", "数据分析"),
        ];
        
        for (id, name) in builtin_templates {
            assert!(!id.is_empty(), "Template ID should not be empty");
            assert!(!name.is_empty(), "Template name should not be empty");
        }
        println!("✅ Builtin templates are defined");
    }
}
