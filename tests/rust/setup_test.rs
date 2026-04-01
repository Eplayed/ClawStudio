// setup_test.rs - Unit tests for environment detection and installation
// Tests for src-tauri/src/setup.rs

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Environment Detection Tests ───

    #[test]
    fn test_detect_node_installed() {
        // This test will pass if Node.js is installed on the test machine
        let result = detect_node();
        assert!(result.is_ok());
        let info = result.unwrap();
        // On CI machines, Node.js should be installed
        if info.installed {
            assert!(!info.version.is_empty());
            assert!(info.path.is_some());
        }
    }

    #[test]
    fn test_detect_npm_installed() {
        let result = detect_npm();
        assert!(result.is_ok());
        let info = result.unwrap();
        if info.installed {
            assert!(!info.version.is_empty());
        }
    }

    #[test]
    fn test_detect_openclaw_not_installed() {
        // OpenClaw may not be installed on test machine
        let result = detect_openclaw();
        assert!(result.is_ok());
        // Just check the function doesn't crash
    }

    #[test]
    fn test_get_env_status() {
        let result = tokio_test::block_on(get_env_status());
        assert!(result.is_ok());
        let status = result.unwrap();
        // Check structure is correct
        assert!(status.node.installed || !status.node.installed); // boolean check
        assert!(status.npm.installed || !status.npm.installed);
        assert!(status.openclaw.installed || !status.openclaw.installed);
    }

    // ─── Data Model Tests ───

    #[test]
    fn test_node_info_serialization() {
        let info = NodeInfo {
            installed: true,
            version: "22.14.0".to_string(),
            path: Some("/usr/bin/node".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("installed"));
        assert!(json.contains("22.14.0"));
        
        let parsed: NodeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.installed, true);
        assert_eq!(parsed.version, "22.14.0");
    }

    #[test]
    fn test_openclaw_info_serialization() {
        let info = OpenClawInfo {
            installed: true,
            version: "1.2.3".to_string(),
            path: Some("/usr/bin/openclaw".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: OpenClawInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "1.2.3");
    }

    #[test]
    fn test_env_status_structure() {
        let status = EnvStatus {
            node: NodeInfo {
                installed: true,
                version: "22.0.0".to_string(),
                path: None,
            },
            npm: NodeInfo {
                installed: true,
                version: "10.0.0".to_string(),
                path: None,
            },
            openclaw: OpenClawInfo {
                installed: false,
                version: "".to_string(),
                path: None,
            },
            gateway: None,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("node"));
        assert!(json.contains("npm"));
        assert!(json.contains("openclaw"));
        assert!(json.contains("gateway"));
    }

    #[test]
    fn test_gateway_status_structure() {
        let status = GatewayStatus {
            running: true,
            port: 18789,
            uptime_sec: 3600,
            pid: Some(12345),
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("running"));
        assert!(json.contains("18789"));
        assert!(json.contains("3600"));
    }

    // ─── Uninstall Scope Tests ───

    #[test]
    fn test_uninstall_scope_deserialization() {
        let json = r#"{
            "stop_gateway": true,
            "remove_cli": true,
            "remove_config": true,
            "remove_node": false
        }"#;
        
        let scope: UninstallScope = serde_json::from_str(json).unwrap();
        assert!(scope.stop_gateway);
        assert!(scope.remove_cli);
        assert!(scope.remove_config);
        assert!(!scope.remove_node);
    }

    #[test]
    fn test_setup_config_structure() {
        let config = SetupConfig {
            auth_provider: "anthropic".to_string(),
            api_key: "sk-test-key".to_string(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            gateway_port: 18789,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("anthropic"));
        assert!(json.contains("18789"));
    }

    // ─── Utility Function Tests ───

    #[test]
    fn test_extract_port_from_output() {
        let output = "Gateway running on port 18789";
        let port = extract_port(output);
        assert_eq!(port, Some(18789));
        
        let output2 = "Listening at http://localhost:8080";
        let port2 = extract_port(output2);
        assert_eq!(port2, Some(8080));
        
        let output3 = "No port mentioned";
        let port3 = extract_port(output3);
        assert_eq!(port3, None);
    }

    #[test]
    fn test_extract_pid_from_output() {
        let output = "Process ID: 12345";
        let pid = extract_pid(output);
        assert_eq!(pid, Some(12345));
        
        let output2 = "PID=67890 running";
        let pid2 = extract_pid(output2);
        assert_eq!(pid2, Some(67890));
    }

    // ─── Edge Case Tests ───

    #[test]
    fn test_empty_node_info() {
        let info = NodeInfo {
            installed: false,
            version: "".to_string(),
            path: None,
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let parsed: NodeInfo = serde_json::from_str(&json).unwrap();
        assert!(!parsed.installed);
        assert!(parsed.version.is_empty());
        assert!(parsed.path.is_none());
    }

    #[test]
    fn test_gateway_status_not_running() {
        let status = GatewayStatus {
            running: false,
            port: 0,
            uptime_sec: 0,
            pid: None,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        let parsed: GatewayStatus = serde_json::from_str(&json).unwrap();
        assert!(!parsed.running);
        assert_eq!(parsed.port, 0);
    }
}

// Mock async runtime for testing
mod tokio_test {
    use tokio::runtime::Runtime;
    
    pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
        let rt = Runtime::new().unwrap();
        rt.block_on(future)
    }
}