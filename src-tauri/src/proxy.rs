use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use std::time::Duration;

use crate::proxy_state::{ProxyConfig, ProxyEvent, ProxyState};
use tauri::{AppHandle, Emitter, Manager, State};

/// 代理服务器核心模块
/// 
/// 功能:
/// - HTTP 代理服务器，监听本地请求
/// - 转发到 Anthropic API
/// - 解析响应，提取 token 使用量、思维流、动作流
/// - 支持 HITL 暂停和熔断

/// 启动代理服务器
pub async fn start_proxy_server(
    port: u16,
    config: ProxyConfig,
    event_sender: mpsc::Sender<ProxyEvent>,
) -> Result<(), String> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    // 创建 TCP 监听器
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;
    
    println!("[Proxy] Server listening on http://{}", addr);
    
    // 更新状态
    let state = Arc::new(ProxyState::new(config, event_sender.clone()));
    state.running.store(true, std::sync::atomic::Ordering::Relaxed);
    state.port.store(port, std::sync::atomic::Ordering::Relaxed);
    
    // 发送启动事件
    let _ = event_sender.send(ProxyEvent::StatusChange {
        running: true,
        port,
    }).await;
    
    // 主循环
    loop {
        match listener.accept().await {
            Ok((stream, client_addr)) => {
                let state = state.clone();
                let event_sender = event_sender.clone();
                
                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    let service = service_fn(move |req| {
                        handle_request(req, state.clone(), event_sender.clone())
                    });
                    
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        println!("[Proxy] Error serving connection {}: {}", client_addr, err);
                    }
                });
            }
            Err(e) => {
                println!("[Proxy] Accept error: {}", e);
            }
        }
    }
}

/// 停止代理服务器
pub async fn stop_proxy_server(state: Arc<ProxyState>) {
    state.running.store(false, std::sync::atomic::Ordering::Relaxed);
    println!("[Proxy] Server stopped");
}

/// 处理传入请求
async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
    event_sender: mpsc::Sender<ProxyEvent>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // 检查服务器是否运行
    if !state.running.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(Response::builder().status(hyper::StatusCode::SERVICE_UNAVAILABLE)
            .body(Full::new(Bytes::from("Proxy server is not running")))
            .unwrap());
    }
    
    // 检查熔断状态
    if state.is_circuit_broken() {
        let current = state.get_current_cost();
        let limit = state.budget_limit.load(std::sync::atomic::Ordering::Relaxed);
        return Ok(Response::builder().status(hyper::StatusCode::TOO_MANY_REQUESTS)
            .body(Full::new(Bytes::from(format!(
                "Circuit breaker active: budget exceeded (${:.2}/${:.2})",
                current, limit
            ))))
            .unwrap());
    }
    
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let method_for_match = method.clone();
    let method_for_log = method.clone();
    
    println!("[Proxy] {} {} from {}", method_for_log, path, req.uri());
    
    // 路由处理
    match (method_for_match, path.as_str()) {
        // Anthropic Messages API
        (Method::POST, "/v1/messages") => {
            handle_anthropic_messages(req, state, event_sender).await
        }
        // OpenAI Chat Completions (兼容)
        (Method::POST, "/v1/chat/completions") => {
            handle_openai_chat(req, state, event_sender).await
        }
        // 健康检查
        (Method::GET, "/health") => {
            Ok(Response::builder().status(hyper::StatusCode::OK)
                .body(Full::new(Bytes::from(r#"{"status":"ok"}"#)))
                .unwrap())
        }
        // 状态检查
        (Method::GET, "/status") => {
            let status = serde_json::json!({
                "running": state.running.load(std::sync::atomic::Ordering::Relaxed),
                "port": state.port.load(std::sync::atomic::Ordering::Relaxed),
                "total_cost": state.get_current_cost(),
                "budget_limit": state.budget_limit.load(std::sync::atomic::Ordering::Relaxed),
                "circuit_broken": state.is_circuit_broken(),
            });
            Ok(Response::builder().status(hyper::StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(status.to_string())))
                .unwrap())
        }
        // 未知路径
        _ => {
            Ok(Response::builder().status(hyper::StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from(format!(
                    "Not found: {} {}",
                    method, path
                ))))
                .unwrap())
        }
    }
}

/// 处理 Anthropic Messages API 请求
async fn handle_anthropic_messages(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
    event_sender: mpsc::Sender<ProxyEvent>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // 获取请求体
    let body = req.collect().await?.to_bytes();
    
    // 解析请求 JSON
    let request_json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Response::builder().status(hyper::StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!("Invalid JSON: {}", e))))
                .unwrap());
        }
    };
    
    // 提取 model
    let model = request_json.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-3-5-sonnet-20241022")
        .to_string();
    
    // 提取 messages 用于调试
    if let Some(messages) = request_json.get("messages") {
        let msg_count = messages.as_array().map(|a| a.len()).unwrap_or(0);
        println!("[Proxy] Request with {} messages, model: {}", msg_count, model);
    }
    
    // 转发到 Anthropic API
    let config = state.config.lock().await.clone();
    let api_key = get_api_key().await.unwrap_or_default();
    
    match forward_to_anthropic(&body, &model, &api_key, &config).await {
        Ok(response) => {
            // 解析响应
            let response_json: serde_json::Value = match serde_json::from_slice(&response) {
                Ok(v) => v,
                Err(e) => {
                    println!("[Proxy] Failed to parse response: {}", e);
                    return Ok(Response::builder().status(hyper::StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(Full::new(response))
                        .unwrap());
                }
            };
            
            // 提取使用量
            if let Some(usage) = response_json.get("usage") {
                let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let image_tokens = usage.get("image_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                
                let cost = crate::proxy_state::calculate_cost(&model, input_tokens, output_tokens, image_tokens);
                
                // 更新费用
                state.add_cost(cost);
                
                // 发送 Token 使用事件
                let _ = event_sender.send(ProxyEvent::TokenUsage {
                    input: input_tokens,
                    output: output_tokens,
                    image: image_tokens,
                    cost,
                    model: model.clone(),
                }).await;
                
                println!("[Proxy] Token usage: in={}, out={}, cost=${:.4}", 
                    input_tokens, output_tokens, cost);
            }
            
            // 提取思维流和动作流
            if let Some(stop_reason) = response_json.get("stop_reason").and_then(|v| v.as_str()) {
                if stop_reason == "tool_use" {
                    // 处理工具调用
                    if let Some(content) = response_json.get("content").and_then(|v| v.as_array()) {
                        for block in content {
                            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                                match block_type {
                                    "text" => {
                                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                            let step = state.next_thinking_step();
                                            let _ = event_sender.send(ProxyEvent::Thinking {
                                                text: text.to_string(),
                                                step,
                                            }).await;
                                        }
                                    }
                                    "tool_use" => {
                                        if let Some(tool) = block.get("name").and_then(|v| v.as_str()) {
                                            let params = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                                            let step = state.next_action_step();
                                            let _ = event_sender.send(ProxyEvent::Action {
                                                tool: tool.to_string(),
                                                params,
                                                step,
                                            }).await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(Response::builder().status(hyper::StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(response))
                .unwrap())
        }
        Err(e) => {
            let error_msg = format!("Forward failed: {}", e);
            println!("[Proxy] {}", error_msg);
            
            let _ = event_sender.send(ProxyEvent::Error {
                message: error_msg.clone(),
            }).await;
            
            Ok(Response::builder().status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(error_msg)))
                .unwrap())
        }
    }
}

/// 处理 OpenAI Chat Completions 请求
async fn handle_openai_chat(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
    event_sender: mpsc::Sender<ProxyEvent>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // 类似 Anthropic 处理，但使用 OpenAI 格式
    let body = req.collect().await?.to_bytes();
    
    let request_json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Response::builder().status(hyper::StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!("Invalid JSON: {}", e))))
                .unwrap());
        }
    };
    
    let model = request_json.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4o")
        .to_string();
    
    let config = state.config.lock().await.clone();
    let api_key = get_api_key().await.unwrap_or_default();
    
    match forward_to_openai(&body, &model, &api_key, &config).await {
        Ok(response) => {
            // 解析并提取使用量
            if let Ok(response_json) = serde_json::from_slice::<serde_json::Value>(&response) {
                if let Some(usage) = response_json.get("usage") {
                    let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let completion_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    
                    let cost = crate::proxy_state::calculate_cost(&model, prompt_tokens, completion_tokens, 0);
                    state.add_cost(cost);
                    
                    let _ = event_sender.send(ProxyEvent::TokenUsage {
                        input: prompt_tokens,
                        output: completion_tokens,
                        image: 0,
                        cost,
                        model: model.clone(),
                    }).await;
                }
            }
            
            Ok(Response::builder().status(hyper::StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(response))
                .unwrap())
        }
        Err(e) => {
            Ok(Response::builder().status(hyper::StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!("Forward failed: {}", e))))
                .unwrap())
        }
    }
}

/// 转发请求到 Anthropic API
async fn forward_to_anthropic(
    body: &Bytes,
    _model: &str,
    api_key: &str,
    config: &ProxyConfig,
) -> Result<Bytes, String> {
    let client = reqwest::Client::new();
    let target_url = format!("{}/v1/messages", config.target_url);
    
    let response = client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .body(body.clone())
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    let status = response.status();
    let body = response.bytes().await.map_err(|e| e.to_string())?;
    
    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, String::from_utf8_lossy(&body)));
    }
    Ok(body)
}
    
/// 转发请求到 OpenAI API
async fn forward_to_openai(
    body: &Bytes,
    _model: &str,
    api_key: &str,
    config: &ProxyConfig,
) -> Result<Bytes, String> {
    let client = reqwest::Client::new();
    
    // 解析并修改模型名称
    let mut request_json: serde_json::Value = serde_json::from_slice(body).map_err(|e| e.to_string())?;
    if let Some(obj) = request_json.as_object_mut() {
        if let Some(model) = obj.get("model").and_then(|v| v.as_str()) {
            let openai_model = match model {
                "gpt-4o" => "gpt-4o",
                "gpt-4-turbo" => "gpt-4-turbo-preview",
                "gpt-3.5-turbo" => "gpt-3.5-turbo",
                _ => model,
            };
            obj.insert("model".to_string(), serde_json::Value::String(openai_model.to_string()));
        }
    }
    let new_body = serde_json::to_vec(&request_json).map_err(|e| e.to_string())?;
    
    let target_url = format!("{}/v1/chat/completions", config.target_url);
    
    let response = client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(new_body)
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    let status = response.status();
    let body = response.bytes().await.map_err(|e| e.to_string())?;
    
    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, String::from_utf8_lossy(&body)));
    }
    
    Ok(body)
}

/// 从 keychain 获取 API Key
async fn get_api_key() -> Result<String, String> {
    // 优先从环境变量获取
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    
    // TODO: 从 keychain 模块获取
    Err("API key not found".to_string())
}

/// 从 keychain 获取 API Key (同步版本)
#[allow(dead_code)]
fn get_api_key_sync() -> Result<String, String> {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    
    Err("API key not found".to_string())
}

// ============================================================================
// Tauri Commands - Expose proxy functionality to the frontend
// ============================================================================


/// 全局代理状态
pub struct ProxyServerState {
    pub shutdown_tx: Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub running: Arc<std::sync::atomic::AtomicBool>,
    pub port: Arc<std::sync::atomic::AtomicU16>,
}

impl Default for ProxyServerState {
    fn default() -> Self {
        Self {
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        shutdown_tx: Arc::new(tokio::sync::Mutex::new(None)),
            port: Arc::new(std::sync::atomic::AtomicU16::new(18788)),
        }
    }
}

/// 启动代理服务器
#[tauri::command]
pub async fn start_proxy(
    app_handle: AppHandle,
    state: State<'_, ProxyServerState>,
    port: Option<u16>,
    budget_limit: Option<f64>,
    hitl_enabled: Option<bool>,
) -> Result<String, String> {
    // 检查是否已运行
    if state.running.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Proxy server is already running".to_string());
    }
    
    let port = port.unwrap_or(18788);
    state.port.store(port, std::sync::atomic::Ordering::Relaxed);
    
    // 创建事件通道 - 将代理事件发送到 Tauri 前端
    let (event_tx, mut event_rx) = mpsc::channel::<ProxyEvent>(100);
    
    // 创建 shutdown 通道
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let mut tx = state.shutdown_tx.lock().await; *tx = Some(shutdown_tx);
    
    // 配置
    let config = ProxyConfig {
        port,
        target_url: "https://api.anthropic.com".to_string(),
        budget_limit: budget_limit.unwrap_or(100.0),
        hitl_enabled: hitl_enabled.unwrap_or(false),
        circuit_breaker_enabled: true,
        request_timeout_secs: 120,
    };
    
    println!("[Proxy] Starting proxy server on port {}", port);
    
    // 启动服务器
    let running = state.running.clone();
    running.store(true, std::sync::atomic::Ordering::Relaxed);
    
    // 启动后台任务处理事件
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let _ = app_handle_clone.emit("proxy:event", event);
        }
    });
    
    // 启动服务器任务
    let running_clone = running.clone();
    tokio::spawn(async move {
        // 使用优雅 shutdown
        tokio::select! {
            result = start_proxy_server(port, config, event_tx) => {
                if let Err(e) = result {
                    println!("[Proxy] Server error: {}", e);
                }
            }
            _ = &mut shutdown_rx => {
                println!("[Proxy] Shutdown signal received");
            }
        }
        running_clone.store(false, std::sync::atomic::Ordering::Relaxed);
    });
    
    Ok(format!("Proxy server started on port {}", port))
}

/// 停止代理服务器
#[tauri::command]
pub async fn stop_proxy(state: State<'_, ProxyServerState>) -> Result<String, String> {
    if !state.running.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("Proxy server is not running".to_string());
    }
    
    // 发送 shutdown 信号
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
    
    state.running.store(false, std::sync::atomic::Ordering::Relaxed);
    
    Ok("Proxy server stopped".to_string())
}

/// 获取代理状态
#[tauri::command]
pub fn get_proxy_status(state: State<'_, ProxyServerState>) -> serde_json::Value {
    serde_json::json!({
        "running": state.running.load(std::sync::atomic::Ordering::Relaxed),
        "port": state.port.load(std::sync::atomic::Ordering::Relaxed),
    })
}

/// 重置费用计数
#[tauri::command]
pub fn reset_proxy_cost() -> Result<String, String> {
    // TODO: 连接到实际的 ProxyState
    Ok("Cost reset (not implemented)".to_string())
}
