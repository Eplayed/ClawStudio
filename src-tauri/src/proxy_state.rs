use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, oneshot};
use tokio::time::Instant;
use serde::{Deserialize, Serialize};

/// Proxy 状态管理模块
/// 管理代理服务器的运行时状态、费用统计、HITL 请求等

/// HITL (Human-In-The-Loop) 请求结构
#[derive(Debug)]
pub struct HitlRequest {
    pub request_id: String,
    pub tool: String,
    pub params: serde_json::Value,
    pub created_at: Instant,
    pub response_tx: oneshot::Sender<HitlResponse>,
}

/// HITL 响应类型
#[derive(Clone, Debug)]
pub enum HitlResponse {
    Approve,
    Reject { error_message: String },
    Timeout,
}

/// 代理事件 - 发送到前端
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ProxyEvent {
    /// Token 使用量更新
    TokenUsage {
        input: u32,
        output: u32,
        image: u32,
        cost: f64,
        model: String,
    },
    /// 思维流更新
    Thinking {
        text: String,
        step: u32,
    },
    /// 工具执行流
    Action {
        tool: String,
        params: serde_json::Value,
        step: u32,
    },
    /// HITL 暂停请求
    HitlRequest {
        request_id: String,
        tool: String,
        params: serde_json::Value,
    },
    /// HITL 响应
    HitlResponse {
        request_id: String,
        approved: bool,
        response: Option<serde_json::Value>,
    },
    /// 熔断触发
    CircuitBreaker {
        reason: String,
        current_cost: f64,
        limit: f64,
    },
    /// 代理状态变化
    StatusChange {
        running: bool,
        port: u16,
    },
    /// 错误事件
    Error {
        message: String,
    },
}

/// 代理配置
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProxyConfig {
    /// 代理监听端口
    pub port: u16,
    /// 目标 API 基础 URL
    pub target_url: String,
    /// 预算上限 ($)
    pub budget_limit: f64,
    /// 是否启用 HITL
    pub hitl_enabled: bool,
    /// 是否启用熔断
    pub circuit_breaker_enabled: bool,
    /// 请求超时 (秒)
    pub request_timeout_secs: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 18788,
            target_url: "https://api.anthropic.com".to_string(),
            budget_limit: 100.0,
            hitl_enabled: false,
            circuit_breaker_enabled: true,
            request_timeout_secs: 120,
        }
    }
}

/// 代理运行时状态
pub struct ProxyState {
    /// 监听端口
    pub port: Arc<std::sync::atomic::AtomicU16>,
    /// 运行状态
    pub running: Arc<std::sync::atomic::AtomicBool>,
    /// 累计费用
    pub total_cost: Arc<std::sync::atomic::AtomicI64>,
    /// 预算上限
    pub budget_limit: Arc<std::sync::atomic::AtomicI64>,
    /// 待处理的 HITL 请求
    pub hitl_pending: Arc<Mutex<HashMap<String, HitlRequest>>>,
    /// 事件发送器 - 发送到 Tauri 前端
    pub event_sender: mpsc::Sender<ProxyEvent>,
    /// 配置副本
    pub config: Arc<Mutex<ProxyConfig>>,
    /// 当前思考步骤
    pub thinking_step: Arc<std::sync::atomic::AtomicU32>,
    /// 当前动作步骤
    pub action_step: Arc<std::sync::atomic::AtomicU32>,
    /// 熔断状态
    pub circuit_broken: Arc<std::sync::atomic::AtomicBool>,
}

impl ProxyState {
    /// 创建新的代理状态
    pub fn new(config: ProxyConfig, event_sender: mpsc::Sender<ProxyEvent>) -> Self {
        Self {
            port: Arc::new(std::sync::atomic::AtomicU16::new(config.port)),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            total_cost: Arc::new(std::sync::atomic::AtomicI64::new(0)),
            budget_limit: Arc::new(std::sync::atomic::AtomicI64::new((config.budget_limit * 10000.0) as i64)),
            hitl_pending: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            config: Arc::new(Mutex::new(config)),
            thinking_step: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            action_step: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            circuit_broken: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 增加费用 (存储为 i64，单位: 0.0001 美元)
    pub fn add_cost(&self, cost: f64) {
        let cost_i64 = (cost * 10000.0) as i64;
        let new_total = self.total_cost.fetch_add(cost_i64, std::sync::atomic::Ordering::Relaxed) + cost_i64;
        
        // 检查是否超过预算
        let limit = self.budget_limit.load(std::sync::atomic::Ordering::Relaxed);
        if new_total > limit {
            self.trigger_circuit_breaker(format!("Budget exceeded: ${:.2} > ${:.2}", 
                new_total as f64 / 10000.0, limit as f64 / 10000.0));
        }
    }

    /// 重置费用计数
    pub fn reset_cost(&self) {
        self.total_cost.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取当前费用 (美元)
    pub fn get_current_cost(&self) -> f64 {
        self.total_cost.load(std::sync::atomic::Ordering::Relaxed) as f64 / 10000.0
    }

    /// 触发熔断
    pub fn trigger_circuit_breaker(&self, reason: String) {
        self.circuit_broken.store(true, std::sync::atomic::Ordering::Relaxed);
        
        let current_cost = self.get_current_cost();
        let limit = self.budget_limit.load(std::sync::atomic::Ordering::Relaxed) as f64 / 10000.0;
        
        let event = ProxyEvent::CircuitBreaker {
            reason,
            current_cost,
            limit,
        };
        
        // 异步发送事件，不阻塞
        let sender = self.event_sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(event).await;
        });
    }

    /// 重置熔断
    pub fn reset_circuit_breaker(&self) {
        self.circuit_broken.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// 熔断是否触发
    pub fn is_circuit_broken(&self) -> bool {
        self.circuit_broken.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取下一步思考编号
    pub fn next_thinking_step(&self) -> u32 {
        self.thinking_step.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// 获取下一步动作编号
    pub fn next_action_step(&self) -> u32 {
        self.action_step.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// 添加 HITL 请求
    pub async fn add_hitl_request(&self, request: HitlRequest) {
        let mut pending = self.hitl_pending.lock().await;
        pending.insert(request.request_id.clone(), request);
    }

    /// 移除 HITL 请求
    pub async fn remove_hitl_request(&self, request_id: &str) -> Option<HitlRequest> {
        let mut pending = self.hitl_pending.lock().await;
        pending.remove(request_id)
    }

    /// 获取待处理 HITL 请求数量
    pub async fn pending_hitl_count(&self) -> usize {
        let pending = self.hitl_pending.lock().await;
        pending.len()
    }
}

/// 模型定价信息
pub struct ModelPricing {
    pub model_id: String,
    pub input_price_per_m: f64,  // $/M tokens
    pub output_price_per_m: f64,
}

/// 获取模型定价
pub fn get_model_pricing(model: &str) -> Option<ModelPricing> {
    let pricing = match model {
        "claude-3-5-sonnet-20241022" | "claude-sonnet-4-20250514" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 3.0, output_price_per_m: 15.0 }
        }
        "claude-3-opus-20240229" | "claude-opus-4-20250514" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 15.0, output_price_per_m: 75.0 }
        }
        "claude-3-5-haiku-20240307" | "claude-haiku-4-20250514" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 0.8, output_price_per_m: 4.0 }
        }
        "gpt-4o" | "gpt-4o-2024-05-13" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 2.5, output_price_per_m: 10.0 }
        }
        "gpt-4-turbo-2024-04-09" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 10.0, output_price_per_m: 30.0 }
        }
        "gpt-3.5-turbo" => {
            ModelPricing { model_id: model.to_string(), input_price_per_m: 0.5, output_price_per_m: 1.5 }
        }
        _ => return None,
    };
    Some(pricing)
}

/// 计算请求费用
pub fn calculate_cost(model: &str, input_tokens: u32, output_tokens: u32, image_tokens: u32) -> f64 {
    let Some(pricing) = get_model_pricing(model) else {
        return 0.0;
    };
    
    let input_cost = (input_tokens as f64) / 1_000_000.0 * pricing.input_price_per_m;
    let output_cost = (output_tokens as f64) / 1_000_000.0 * pricing.output_price_per_m;
    let image_cost = (image_tokens as f64) / 1_000_000.0 * pricing.input_price_per_m; // 图片按输入计费
    
    input_cost + output_cost + image_cost
}
