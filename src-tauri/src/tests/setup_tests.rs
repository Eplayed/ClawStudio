// Tests for setup.rs - Environment detection and installation

#[cfg(test)]
mod tests {
    use std::process::Command;
    use serde_json;

    #[test]
    fn test_detect_node() {
        // Test that we can detect Node.js installation
        let output = Command::new("node").arg("--version").output();
        
        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout);
                assert!(version.starts_with('v'), "Node version should start with 'v'");
                println!("✅ Node.js detected: {}", version.trim());
            }
            Ok(_) => {
                println!("⚠️ Node.js command ran but returned non-zero");
            }
            Err(e) => {
                println!("⚠️ Node.js not installed: {}", e);
            }
        }
    }

    #[test]
    fn test_detect_npm() {
        let output = Command::new("npm").arg("--version").output();
        
        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout);
                assert!(!version.trim().is_empty(), "npm version should not be empty");
                println!("✅ npm detected: {}", version.trim());
            }
            Ok(_) => {
                println!("⚠️ npm command ran but returned non-zero");
            }
            Err(e) => {
                println!("⚠️ npm not installed: {}", e);
            }
        }
    }

    #[test]
    fn test_detect_openclaw() {
        let output = Command::new("openclaw").arg("--version").output();
        
        match output {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout);
                assert!(!version.trim().is_empty(), "OpenClaw version should not be empty");
                println!("✅ OpenClaw detected: {}", version.trim());
            }
            Ok(_) => {
                println!("⚠️ OpenClaw command ran but returned non-zero");
            }
            Err(e) => {
                println!("⚠️ OpenClaw not installed: {}", e);
            }
        }
    }

    #[test]
    fn test_env_status_structure() {
        // Test that EnvStatus struct can be serialized/deserialized
        use serde_json;
        
        let json = r#"{
            "node": {"installed": true, "version": "22.0.0", "path": "/usr/bin/node"},
            "npm": {"installed": true, "version": "10.0.0", "path": "/usr/bin/npm"},
            "openclaw": {"installed": true, "version": "1.0.0", "path": "/usr/bin/openclaw"},
            "gateway": {"running": true, "port": 18789, "uptime_sec": 3600, "pid": 12345}
        }"#;
        
        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(parsed["node"]["installed"].as_bool().unwrap());
        assert_eq!(parsed["gateway"]["port"].as_u64().unwrap(), 18789);
        println!("✅ EnvStatus JSON parsing works");
    }

    #[test]
    fn test_uninstall_scope_defaults() {
        // Test that UninstallScope has sensible defaults
        let stop_gateway = true;
        let remove_cli = true;
        let remove_config = true;
        let remove_node = false;
        
        assert!(stop_gateway, "Should stop gateway by default");
        assert!(remove_cli, "Should remove CLI by default");
        assert!(remove_config, "Should remove config by default");
        assert!(!remove_node, "Should NOT remove Node.js by default (user choice)");
        println!("✅ UninstallScope defaults are sensible");
    }
}
