#[cfg(test)]
mod tests {
    use crate::proxy_state::{calculate_cost, ProxyConfig, ProxyState, ProxyEvent, is_dangerous_tool};
    use tokio::sync::mpsc;

    // ========== 费用计算测试 ==========

    #[test]
    fn test_cost_calculation_claude_sonnet() {
        // claude-3-5-sonnet-20241022: $3/M input, $15/M output
        let cost = calculate_cost("claude-3-5-sonnet-20241022", 1000, 500, 0);
        // 1000 tokens * ($3/1M) = 0.003
        // 500 tokens * ($15/1M) = 0.0075
        // total = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_claude_opus() {
        // claude-3-opus-20240229: $15/M input, $75/M output
        let cost = calculate_cost("claude-3-opus-20240229", 1000, 100, 0);
        // 1000 tokens * ($15/1M) = 0.015
        // 100 tokens * ($75/1M) = 0.0075
        // total = 0.0225
        assert!((cost - 0.0225).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_gpt4o() {
        // gpt-4o: $2.5/M input, $10/M output
        let cost = calculate_cost("gpt-4o", 1000, 500, 0);
        // 1000 tokens * ($2.5/1M) = 0.0025
        // 500 tokens * ($10/1M) = 0.005
        // total = 0.0075
        assert!((cost - 0.0075).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_with_image_tokens() {
        // 图片 token 也按输入价格计费
        let cost = calculate_cost("claude-3-5-sonnet-20241022", 500, 200, 1000);
        // 500 tokens * ($3/1M) = 0.0015
        // 200 tokens * ($15/1M) = 0.003
        // 1000 image tokens * ($3/1M) = 0.003
        // total = 0.0075
        assert!((cost - 0.0075).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_unknown_model() {
        // 未知模型返回 0
        let cost = calculate_cost("unknown-model", 1000, 500, 0);
        assert_eq!(cost, 0.0);
    }

    // ========== 熔断机制测试 ==========

    #[tokio::test]
    async fn test_circuit_breaker() {
        let (tx, _rx) = mpsc::channel::<ProxyEvent>(10);
        let config = ProxyConfig {
            port: 18788,
            target_url: "".to_string(),
            budget_limit: 0.01, // 10000 内部单位即 1 美分
            hitl_enabled: false,
            circuit_breaker_enabled: true,
            request_timeout_secs: 30,
        };

        let state = ProxyState::new(config, tx);
        
        // 增加 0.005，没超过 0.01
        state.add_cost(0.005);
        assert!(!state.is_circuit_broken());

        // 再增加 0.006，总计 0.011 > 0.01，触发熔断
        state.add_cost(0.006);
        assert!(state.is_circuit_broken());
        
        // 检查值
        let current_cost = state.get_current_cost();
        assert!((current_cost - 0.011).abs() < 0.0001);

        // 重置
        state.reset_cost();
        state.reset_circuit_breaker();
        assert!(!state.is_circuit_broken());
        assert_eq!(state.get_current_cost(), 0.0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_exact_limit() {
        // 刚好达到预算不应该触发熔断
        let (tx, _rx) = mpsc::channel::<ProxyEvent>(10);
        let config = ProxyConfig {
            port: 18788,
            target_url: "".to_string(),
            budget_limit: 0.01,
            hitl_enabled: false,
            circuit_breaker_enabled: true,
            request_timeout_secs: 30,
        };

        let state = ProxyState::new(config, tx);
        
        // 刚好达到预算
        state.add_cost(0.01);
        assert!(!state.is_circuit_broken());

        // 超过预算
        state.add_cost(0.0001);
        assert!(state.is_circuit_broken());
    }

    // ========== 高危工具检测测试 ==========

    #[test]
    fn test_dangerous_tool_bash() {
        assert!(is_dangerous_tool("bash"));
        assert!(is_dangerous_tool("bash_execute"));
        assert!(is_dangerous_tool("run_bash"));
    }

    #[test]
    fn test_dangerous_tool_file_operations() {
        assert!(is_dangerous_tool("file_write"));
        assert!(is_dangerous_tool("write_file"));
        assert!(is_dangerous_tool("create_file"));
        assert!(is_dangerous_tool("delete_file"));
        assert!(is_dangerous_tool("rm_file"));
    }

    #[test]
    fn test_dangerous_tool_editor() {
        assert!(is_dangerous_tool("str_replace_editor"));
        assert!(is_dangerous_tool("str_replace"));
        assert!(is_dangerous_tool("execute_script"));
        assert!(is_dangerous_tool("script_runner"));
    }

    #[test]
    fn test_dangerous_tool_system_commands() {
        assert!(is_dangerous_tool("run_command"));
        assert!(is_dangerous_tool("sudo"));
        assert!(is_dangerous_tool("chmod"));
        assert!(is_dangerous_tool("chown"));
    }

    #[test]
    fn test_safe_tools() {
        // 常见的安全工具不应该被标记
        assert!(!is_dangerous_tool("mouse_move"));
        assert!(!is_dangerous_tool("screenshot"));
        assert!(!is_dangerous_tool("type_text"));
        assert!(!is_dangerous_tool("click"));
        assert!(!is_dangerous_tool("scroll"));
        assert!(!is_dangerous_tool("read_file"));
        assert!(!is_dangerous_tool("list_directory"));
        assert!(!is_dangerous_tool("search"));
        assert!(!is_dangerous_tool("web_search"));
    }

    #[test]
    fn test_dangerous_tool_case_insensitive() {
        // 检测不区分大小写
        assert!(is_dangerous_tool("BASH"));
        assert!(is_dangerous_tool("bash"));
        assert!(is_dangerous_tool("Bash"));
        assert!(is_dangerous_tool("RUN_COMMAND"));
        assert!(is_dangerous_tool("SUDO"));
    }

    #[test]
    fn test_dangerous_tool_partial_match() {
        // 包含危险关键词的也应该被检测
        assert!(is_dangerous_tool("bash_tool"));
        assert!(is_dangerous_tool("my_rm_command"));
        assert!(is_dangerous_tool("dangerous_sudo_access"));
        assert!(is_dangerous_tool("file_write_tool"));
        assert!(is_dangerous_tool("execute_script_runner"));
    }
}