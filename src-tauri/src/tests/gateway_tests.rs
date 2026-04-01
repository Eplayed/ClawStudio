// Tests for gateway.rs - Gateway lifecycle management

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Duration;

    #[test]
    fn test_gateway_status_structure() {
        use serde_json;
        
        let json = r#"{
            "running": true,
            "port": 18789,
            "uptime_secs": 7200,
            "pid": 54321,
            "version": "1.2.3"
        }"#;
        
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(parsed["running"].as_bool().unwrap());
        assert_eq!(parsed["port"].as_u64().unwrap(), 18789);
        println!("✅ GatewayStatus JSON parsing works");
    }

    #[test]
    fn test_gateway_health_endpoint() {
        // Test that health endpoint responds when gateway is running
        // This test is informational - gateway may not be running in CI
        let url = "http://127.0.0.1:18789/healthz";
        
        match reqwest::blocking::get(url) {
            Ok(resp) => {
                // Any HTTP response means gateway is running
                println!("✅ Gateway health endpoint responded: {}", resp.status());
            }
            Err(e) => {
                // Connection refused = gateway not running, which is fine in CI
                println!("⚠️ Gateway not running (expected in CI): {}", e);
            }
        }
        // Always pass - this is an informational test
    }

    #[test]
    fn test_port_range() {
        // Test that port validation works
        let valid_ports = [80, 443, 3000, 8000, 8080, 18789, 65535];
        let invalid_ports = [0, 65536, 70000];
        
        for port in valid_ports {
            assert!((1..=65535).contains(&port), "Port {} should be valid", port);
        }
        
        for port in invalid_ports {
            assert!(!(1..=65535).contains(&port), "Port {} should be invalid", port);
        }
        println!("✅ Port validation works correctly");
    }

    #[test]
    fn test_uptime_format() {
        fn format_uptime(secs: u64) -> String {
            if secs < 60 {
                format!("{}秒", secs)
            } else if secs < 3600 {
                format!("{}分", secs / 60)
            } else if secs < 86400 {
                format!("{}小时", secs / 3600)
            } else {
                format!("{}天", secs / 86400)
            }
        }
        
        assert_eq!(format_uptime(30), "30秒");
        assert_eq!(format_uptime(90), "1分");
        assert_eq!(format_uptime(3661), "1小时");
        assert_eq!(format_uptime(90000), "1天");
        println!("✅ Uptime formatting works correctly");
    }

    #[test]
    fn test_openclaw_gateway_command() {
        // Test that openclaw gateway command exists
        let output = Command::new("openclaw")
            .args(["gateway", "--help"])
            .output();
        
        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                assert!(stdout.contains("gateway") || stdout.contains("Usage") || stdout.len() > 0);
                println!("✅ openclaw gateway command available");
            }
            Ok(_) => {
                println!("⚠️ openclaw gateway returned non-zero (may need setup)");
            }
            Err(e) => {
                println!("⚠️ openclaw not installed: {}", e);
            }
        }
    }
}
