# ClawStudio Nova PRD v2.1 实现计划

## 概述

PRD v2.1 核心变更：从"标准流劫持 + Mock 数据"转向"本地反向代理"架构。

**核心理念：** ClawStudio 作为 OpenClaw 与 LLM API 之间的强制收费站和安检机。

## 架构图

```
[OpenClaw] → [ClawStudio Proxy:18788] → [LLM API]
                    ↓
            ┌──────┴──────┐
            │  拦截层      │
            ├─────────────┤
            │ • Token 提取 │ → 实时费用更新
            │ • HITL 检测  │ → 挂起高危请求
            │ • 熔断机制   │ → 超预算切断
            └─────────────┘
```

## 任务清单

### Phase 1: 核心代理模块 (P0)

#### Task 1.1: 创建 proxy.rs 基础结构 ✅已完成
- [x] 定义 ProxyConfig, ProxyState 结构体
- [x] 实现 ProxyServer 基础框架
- [x] 添加依赖: hyper, tower, http-body-util
- [x] 创建 proxy_state.rs 状态管理

**文件:** 
- `src-tauri/src/proxy.rs` - 代理核心模块 (~400行)
- `src-tauri/src/proxy_state.rs` - 状态管理 (~150行)

**核心结构体:**
```rust
// proxy_state.rs
pub struct ProxyState {
    pub port: u16,
    pub running: Arc<AtomicBool>,
    pub total_cost: Arc<AtomicF64>,
    pub budget_limit: Arc<AtomicF64>,
    pub hitl_pending: Arc<Mutex<HashMap<String, HitlRequest>>>,
    pub event_sender: tokio::sync::mpsc::Sender<ProxyEvent>,
}

pub struct HitlRequest {
    pub request_id: String,
    pub tool: String,
    pub params: serde_json::Value,
    pub created_at: Instant,
    pub response_tx: oneshot::Sender<HitlResponse>,
}

pub enum HitlResponse {
    Approve,
    Reject { error_message: String },
    Timeout,
}
```

#### Task 1.2: 实现 HTTP 代理服务 ⏳待开始
- [ ] 监听 127.0.0.1:18788
- [ ] 接收 POST /v1/messages 请求
- [ ] 转发到真实 API (api.anthropic.com)
- [ ] 返回响应给 OpenClaw
- [ ] 支持 /v1/chat/completions (OpenAI兼容)

**关键代码:**
```rust
// proxy.rs
pub async fn start_proxy_server(
    port: u16,
    event_sender: mpsc::Sender<ProxyEvent>,
) -> Result<(), String>

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
) -> Result<Response<Full<Bytes>>, Infallible>

async fn forward_to_anthropic(
    req: Request<Full<Bytes>>,
    api_key: &str,
) -> Result<Response<Full<Bytes>>, String>
```

#### Task 1.3: 响应解析与数据提取 ⏳待开始
- [ ] 解析 Anthropic API 响应格式
- [ ] 提取 `usage.input_tokens`, `usage.output_tokens`
- [ ] 提取 `content[].type = "text"` → 思维流
- [ ] 提取 `content[].type = "tool_use"` → 执行流
- [ ] 发送 Tauri 事件到前端

**事件定义:**
```rust
#[derive(Clone, Serialize)]
pub enum ProxyEvent {
    TokenUsage { input: u32, output: u32, image: u32, cost: f64 },
    Thinking { text: String, step: u32 },
    Action { tool: String, params: Value, step: u32 },
    HitlRequest { request_id: String, tool: String, params: Value },
    HitlResponse { request_id: String, approved: bool },
    CircuitBreaker { reason: String, current_cost: f64, limit: f64 },
}

// 费用计算 (基于模型)
const MODEL_PRICING: &[(&str, f64, f64)] = &[
    ("claude-3-5-sonnet-20241022", 3.0, 15.0),  // $3/M input, $15/M output
    ("claude-3-opus-20240229", 15.0, 75.0),
    ("claude-3-5-sonnet", 3.0, 15.0),
    ("gpt-4o", 2.5, 10.0),
    ("gpt-4-turbo", 10.0, 30.0),
];
```

### Phase 2: 配置劫持 (P0)

#### Task 2.1: 修改 setup.rs ⏳待开始
- [ ] 新增 `configure_openclaw_proxy(port: u16)` 命令
- [ ] 修改 `~/.openclaw/openclaw.json`
- [ ] 设置 `agents.defaults.api_base = "http://127.0.0.1:18788/v1"`
- [ ] 支持自定义端口和 API Key

**实现:**
```rust
#[tauri::command]
pub async fn configure_openclaw_proxy(
    proxy_port: u16,
    api_key: String,
    model: String,
) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw/openclaw.json");
    
    // 读取现有配置
    let mut config: serde_json::Value = ...;
    
    // 修改 API base URL
    config["agents"]["defaults"]["api_base"] = serde_json::json!(
        format!("http://127.0.0.1:{}/v1", proxy_port)
    );
    
    // 写入配置
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}
```

#### Task 2.2: 修改 Setup Wizard 前端 ⏳待开始
- [ ] Step 7 (LaunchGateway) 添加代理配置步骤
- [ ] 显示代理端口配置
- [ ] 调用 `configure_openclaw_proxy` 命令
- [ ] 添加"代理运行中"状态指示

**文件:** `src/components/setup/LaunchGateway.vue`

### Phase 3: Gateway 集成 (P0)

#### Task 3.1: 修改 gateway.rs ⏳待开始
- [ ] `start_gateway` 同时启动代理服务
- [ ] `stop_gateway` 同时停止代理服务  
- [ ] `gateway_status` 返回代理状态
- [ ] 添加代理状态到 GatewayState

**修改内容:**
```rust
// gateway.rs - 添加
use crate::proxy::{start_proxy_server, stop_proxy_server, ProxyState};

// 在 start_gateway 中:
let proxy_state = start_proxy_server(18788, app_handle.clone()).await?;
manage(proxy_state);

// 在 gateway_status 中:
#[derive(Serialize)]
pub struct GatewayStatus {
    pub gateway_running: bool,
    pub proxy_running: bool,      // 新增
    pub gateway_port: u16,
    pub proxy_port: u16,          // 新增
    pub total_cost: f64,         // 新增
    // ...
}
```

#### Task 3.2: 添加代理状态管理 ⏳待开始
- [ ] 创建 `ProxyState` 结构体 (使用 Tauri State 管理)
- [ ] 存储累计费用、请求计数、熔断状态
- [ ] 提供 Tauri 命令查询状态

**Tauri 命令:**
```rust
#[tauri::command]
pub async fn get_proxy_status(state: State<'_, ProxyState>) -> Result<ProxyStatus, String> {
    Ok(ProxyStatus {
        running: state.running.load(Ordering::Relaxed),
        port: state.port,
        total_cost: state.total_cost.load(Ordering::Relaxed),
        budget_limit: state.budget_limit.load(Ordering::Relaxed),
        pending_hitl: state.hitl_pending.lock().await.len(),
    })
}

#[tauri::command]
pub async fn set_budget_limit(limit: f64, state: State<'_, ProxyState>) -> Result<(), String> {
    state.budget_limit.store(limit, Ordering::Relaxed);
    Ok(())
}
```

### Phase 4: 前端事件监听 (P0)

#### Task 4.1: 创建代理事件 Store ⏳待开始
- [ ] 创建 `src/stores/proxy.ts`
- [ ] 定义 `tokenUsage`, `thinkingLog`, `actionLog`, `hitlPending` 状态
- [ ] 实现事件监听和状态更新

**文件:** `src/stores/proxy.ts`
```typescript
import { defineStore } from 'pinia'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface ProxyState {
  // Token 费用
  totalCost: number
  inputTokens: number
  outputTokens: number
  imageTokens: number
  // 日志
  thinkingLog: ThoughtEntry[]
  actionLog: ActionEntry[]
  // HITL
  hitlPending: HitlRequest | null
  // 熔断
  circuitBroken: boolean
}

export const useProxyStore = defineStore('proxy', {
  state: (): ProxyState => ({
    totalCost: 0,
    inputTokens: 0,
    outputTokens: 0,
    imageTokens: 0,
    thinkingLog: [],
    actionLog: [],
    hitlPending: null,
    circuitBroken: false,
  }),
  actions: {
    async init() {
      // 监听各事件
      await listen<TokenUsageEvent>('proxy:token_usage', (e) => this.updateCost(e.payload))
      await listen<ThinkingEvent>('proxy:thinking', (e) => this.addThinking(e.payload))
      await listen<ActionEvent>('proxy:action', (e) => this.addAction(e.payload))
      await listen<HitlEvent>('proxy:hitl_request', (e) => this.setHitl(e.payload))
      await listen<CircuitEvent>('proxy:circuit_breaker', (e) => this.setCircuit(e.payload))
    },
  }
})
```

#### Task 4.2: 改造 FuelGauge.vue ⏳待开始
- [ ] 监听 `proxy:token_usage` 事件
- [ ] 实时更新费用显示
- [ ] 移除模拟数据
- [ ] 使用 useProxyStore 替代模拟

**修改:** `src/components/FuelGauge.vue`
```typescript
import { useProxyStore } from '@/stores/proxy'
const proxyStore = useProxyStore()

// 替换原来的模拟数据
const current = computed(() => proxyStore.totalCost)
const limit = computed(() => settingsStore.budgetDefault)
```

#### Task 4.3: 改造 ThoughtLog.vue ⏳待开始
- [ ] 监听 `proxy:thinking` 和 `proxy:action` 事件
- [ ] 实时添加日志条目
- [ ] 移除 WebSocket 数据源（保留作为备选）
- [ ] 使用 useProxyStore

**修改:** `src/components/ThoughtLog.vue`
```typescript
import { useProxyStore } from '@/stores/proxy'
const proxyStore = useProxyStore()

// 替换原来的事件监听
const thinkingEntries = computed(() => proxyStore.thinkingLog)
const actionEntries = computed(() => proxyStore.actionLog)
```

### Phase 5: HITL 拦截机制 (P1)

#### Task 5.1: 高危操作检测 ⏳待开始
- [ ] 定义高危 tool 列表: `bash`, `str_replace`, `execute_script`, `file_write`
- [ ] 在响应解析时检测
- [ ] 发送 `proxy:hitl_request` 事件

**高危工具列表:**
```rust
const DANGEROUS_TOOLS: &[&str] = &[
    "bash",
    "str_replace_editor", 
    "str_replace",
    "execute_script",
    "script",
    "run_command",
    "file_write", 
    "write_file",
    "create_file",
];

fn is_dangerous_tool(tool_name: &str) -> bool {
    DANGEROUS_TOOLS.iter().any(|&d| tool_name.contains(d))
}
```

#### Task 5.2: 请求挂起与放行 ⏳待开始
- [ ] 使用 `oneshot::Sender` 挂起请求
- [ ] 前端发送 `hitl_approve` 或 `hitl_reject` 命令
- [ ] 超时自动拒绝 (30秒)

**实现:**
```rust
// 在 proxy.rs 中
let (tx, rx) = oneshot::channel::<HitlResponse>();

// 存储待审批请求
state.hitl_pending.lock().await.insert(request_id.clone(), HitlRequest {
    request_id: request_id.clone(),
    tool: tool.clone(),
    params: params.clone(),
    created_at: Instant::now(),
    response_tx: tx,
});

// 发送事件给前端
let _ = state.event_sender.send(ProxyEvent::HitlRequest {
    request_id: request_id.clone(),
    tool: tool.clone(),
    params: params.clone(),
}).await;

// 等待用户响应或超时
let response = tokio::time::timeout(Duration::from_secs(30), rx).await;
```

#### Task 5.3: 改造 HITLBar.vue ⏳待开始
- [ ] 监听 `proxy:hitl_request` 事件
- [ ] 显示审批 Modal
- [ ] 发送用户决定到后端

**新增 Tauri 命令:**
```rust
#[tauri::command]
pub async fn hitl_approve(request_id: String, state: State<'_, ProxyState>) -> Result<(), String> {
    if let Some(req) = state.hitl_pending.lock().await.remove(&request_id) {
        let _ = req.response_tx.send(HitlResponse::Approve);
    }
    Ok(())
}

#[tauri::command]
pub async fn hitl_reject(request_id: String, correction: Option<String>, state: State<'_, ProxyState>) -> Result<(), String> {
    if let Some(req) = state.hitl_pending.lock().await.remove(&request_id) {
        let _ = req.response_tx.send(HitlResponse::Reject { 
            error_message: correction.unwrap_or_else(|| "User rejected".to_string()) 
        });
    }
    Ok(())
}
```

#### Task 5.4: 拒绝响应构造 ⏳待开始
- [ ] 构造伪造错误 JSON
- [ ] 返回 `{"error": "User rejected this action"}` 或自定义错误
- [ ] OpenClaw 接收后安全停止

```rust
fn construct_rejection_response(tool: &str) -> String {
    serde_json::json!({
        "type": "error",
        "error": {
            "type": "api_error",
            "message": f!("Tool '{tool}' execution rejected by user")
        },
        "stop_reason": "tool_use_blocked"
    }).to_string()
}
```

### Phase 6: 熔断机制 (P1)

#### Task 6.1: 实现熔断逻辑 ⏳待开始
- [ ] 累计费用跟踪 (每请求后更新)
- [ ] 预算阈值检查 (在请求转发前)
- [ ] 超阈值返回 402 Payment Required
- [ ] 发送 `proxy:circuit_breaker` 事件
- [ ] 支持临时熔断恢复

**熔断逻辑:**
```rust
fn check_circuit_breaker(state: &ProxyState) -> bool {
    let current = state.total_cost.load(Ordering::Relaxed);
    let limit = state.budget_limit.load(Ordering::Relaxed);
    current >= limit
}

fn trigger_circuit_breaker(state: &ProxyState, app_handle: &AppHandle) {
    let current = state.total_cost.load(Ordering::Relaxed);
    let limit = state.budget_limit.load(Ordering::Relaxed);
    
    // 发送熔断事件
    let _ = app_handle.emit("proxy:circuit_breaker", serde_json::json!({
        "reason": "Budget limit exceeded",
        "current_cost": current,
        "limit": limit,
    }));
}
```

#### Task 6.2: 前端熔断通知 ⏳待开始
- [ ] 监听 `proxy:circuit_breaker` 事件
- [ ] 显示全屏警告 Modal
- [ ] 提供恢复选项（增加预算/重置）
- [ ] 阻止新请求发送

**组件:** 修改 `src/App.vue` 或新建 `CircuitBreakerModal.vue`
```vue
<template>
  <div v-if="circuitBroken" class="circuit-modal">
    <div class="warning-box">
      <h2>⚠️ 预算已超支</h2>
      <p>当前费用: ${{ currentCost }}</p>
      <p>预算上限: ${{ budgetLimit }}</p>
      <div class="actions">
        <button @click="increaseBudget">增加预算</button>
        <button @click="resetCost">重置计数</button>
      </div>
    </div>
  </div>
</template>
```

### Phase 7: 测试与优化 (P1)

#### Task 7.1: 单元测试 ⏳待开始
- [ ] 测试响应解析逻辑
- [ ] 测试费用计算
- [ ] 测试 HITL 检测
- [ ] 测试熔断阈值

**测试文件:** `src-tauri/tests/proxy_tests.rs`
```rust
#[test]
fn test_dangerous_tool_detection() {
    assert!(is_dangerous_tool("bash"));
    assert!(is_dangerous_tool("str_replace_editor"));
    assert!(!is_dangerous_tool("mouse_move"));
}

#[test]
fn test_cost_calculation() {
    let cost = calculate_cost("claude-3-5-sonnet-20241022", 1000, 500, 0);
    assert!((cost - 0.0105).abs() < 0.001); // $3/1M * 0.001 + $15/1M * 0.0005
}
```

#### Task 7.2: 集成测试 ⏳待开始
- [ ] 端到端测试 Setup Wizard
- [ ] 测试代理转发流程 (curl 测试)
- [ ] 测试 HITL 拦截流程
- [ ] 测试熔断触发

```bash
# 测试代理服务
curl -X POST http://127.0.0.1:18788/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_KEY" \
  -d '{"model":"claude-3-5-sonnet-20241022","max_tokens":100,"messages":[{"role":"user","content":"Hello"}]}'
```

#### Task 7.3: 性能优化 ⏳待开始
- [ ] 减少事件发送频率 (批量/节流)
- [ ] 优化响应解析 (流式处理)
- [ ] 内存使用优化 (限制日志条数)
- [ ] 连接池复用

---

## 文件变更清单

### 新增文件
```
src-tauri/src/
├── proxy.rs                    # 代理核心模块 (~400行)
├── proxy_state.rs              # 状态管理 (~150行)

src/
├── stores/
│   └── proxy.ts                # 前端代理状态 Store (~120行)
└── composables/
    └── useProxyEvents.ts       # 代理事件 Hook (~80行) [可选]
```

### 修改文件
```
src-tauri/src/
├── main.rs                     # + 注册 proxy 命令
├── lib.rs                      # + pub mod proxy
├── setup.rs                    # + configure_openclaw_proxy()
├── gateway.rs                  # + 启动/停止代理
├── Cargo.toml                  # + proxy 依赖

src/
├── components/
│   ├── FuelGauge.vue          # 使用 proxy store
│   ├── ThoughtLog.vue         # 使用 proxy store
│   ├── HITLBar.vue             # + HITL 事件
│   ├── GatewayStatusBar.vue   # + 代理状态
│   └── setup/LaunchGateway.vue # + 代理配置
├── views/
│   ├── SetupWizard.vue         # + 代理配置步骤
│   └── Settings.vue            # + 代理端口配置
└── stores/
    └── settings.ts             # + budget limit
```

## 依赖添加

```toml
# Cargo.toml
hyper = { version = "1.0", features = ["full"] }
hyper-util = "0.1"
http-body-util = "0.1"
tokio-util = { version = "0.7", features = ["io"] }
```

## 里程碑

---

## 实施日志

### 2026-04-02
- [x] ✅ 创建实现计划文档
- [x] ✅ 补全任务清单细节（Phase 1-7 全部补全）
- [x] ✅ 添加代码示例（结构体、函数签名、测试用例）
- [x] ✅ 补充文件变更清单（新增/修改文件列表）
- [x] ✅ 添加依赖到 Cargo.toml（hyper, hyper-util, http-body-util 等）
- [ ] ⏳ **下一步: Task 1.1 - 创建 proxy.rs 基础结构**
