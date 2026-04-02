#[cfg(test)]
mod tests {
    use crate::proxy_state::{calculate_cost, ProxyConfig, ProxyState, ProxyEvent};
    use tokio::sync::mpsc;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_cost_calculation() {
        // claude-3-5-sonnet-20241022: $3/M input, $15/M output
        let cost = calculate_cost("claude-3-5-sonnet-20241022", 1000, 500, 0);
        // 1000 tokens * ($3/1M) = 0.003
        // 500 tokens * ($15/1M) = 0.0075
        // total = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

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

    // Since is_dangerous_tool is private in proxy.rs, we can't test it directly unless we move it to proxy_state.rs or make it pub.
    // For now, we can skip it or move it.
}