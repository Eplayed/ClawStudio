# ClawStudio v2.1 — Skills（交接给新 AI）

本文件用于把项目交接给新的 AI：提供最小必要背景、常用入口、验证方式与可复用提示词模板。

## 项目速览

- 前端：Vue 3 + Pinia + Vite（目录 `src/`）
- 后端：Rust + Tauri（目录 `src-tauri/src/`）
- 关键能力（v2.1）：本地反向代理（Proxy:18788）作为 OpenClaw 与 LLM API 的“收费站 + 安检机”
- 默认端口：
  - Proxy：`127.0.0.1:18788`
  - OpenClaw Gateway：`127.0.0.1:18789`

## 关键模块地图

- 代理（核心）
  - `src-tauri/src/proxy.rs`：HTTP server、转发、响应解析、事件派发、HITL/熔断分支、Tauri commands
  - `src-tauri/src/proxy_state.rs`：计费累计、熔断触发、状态结构
- Gateway（生命周期）
  - `src-tauri/src/gateway.rs`：start/stop/restart/health/status + OpenClaw config 读写
- Setup（环境检测/配置劫持）
  - `src-tauri/src/setup.rs`：环境检测、安装/配置、configure_openclaw_proxy
- 前端状态与 UI
  - `src/stores/proxy.ts`：监听 `proxy:*` 事件并驱动 UI
  - `src/components/FuelGauge.vue`：预算油表（消费真实 cost）
  - `src/components/ThoughtLog.vue`：思维/动作日志（消费 proxy store）
  - `src/components/HITLBar.vue`：HITL 审批交互（待完善为 Modal）
  - `src/views/CostMonitor.vue`：费用监控页面（消费 proxy store）
  - `src/App.vue`：初始化 proxy store、触发 configure_openclaw_proxy

## 事件命名约定（后端 → 前端）

- `proxy:token_usage`：TokenUsage（含 cost / model / token 计数）
- `proxy:thinking`：Thinking（文本 + step）
- `proxy:action`：Action（tool + params + step）
- `proxy:hitl_request`：HitlRequest（request_id + tool + params）
- `proxy:hitl_response`：HitlResponse（approved + request_id）
- `proxy:circuit_breaker`：CircuitBreaker（reason + current_cost + limit）
- `proxy:status_change`：StatusChange（running + port）
- `proxy:error`：Error（message）
- `proxy:event`：兼容事件（不建议新代码依赖）

## 验证与测试（交付门槛）

### 后端

```bash
cd src-tauri
cargo test
```

### 前端

```bash
npm run build
npm test
```

## 可复用提示词模板

### Skill: Repo Survey（快速熟悉代码库）

```text
你接手的是 ClawStudio v2.1。请先输出：关键模块地图、数据流（OpenClaw → Proxy → LLM API → UI）、以及当前未完成任务列表。
必须阅读：src-tauri/src/proxy.rs、src-tauri/src/proxy_state.rs、src-tauri/src/gateway.rs、src/stores/proxy.ts。
输出要求：列出每个模块职责与关键函数入口，并给出下一步优先级建议。
```

### Skill: Rust Proxy Work（代理/HITL/熔断）

```text
你需要在 Rust/Tauri 后端实现或修复 v2.1 代理能力：
1) 任何对外行为变化必须同步更新 docs/prd-v2.1-implementation-plan.md。
2) 完成后必须通过 cargo test，并为关键逻辑补齐单元测试（优先 proxy_tests.rs）。
3) 严禁记录/打印 API Key 或包含敏感数据的完整请求体。
请给出：改动点列表、验收标准、以及最小测试方案。
```

### Skill: Frontend Integration（Pinia Store + UI）

```text
你需要把后端 proxy:* 事件接入前端 Pinia store，并驱动 UI 实时更新。
要求：
- init 只能注册一次监听（避免重复 listen）。
- 对日志类数组设置上限（避免内存无限增长）。
- 完成后必须 npm run build 通过；建议补 1 个 vitest 用例验证 store 状态机。
请给出：你改动了哪些组件、事件如何映射、以及验收步骤。
```

### Skill: Testing & Automation（测试与自动化）

```text
你需要补齐 v2.1 的测试覆盖与可回归验证流程：
1) 后端单元测试：计费、熔断、（可测试的）危险工具判定、响应解析。
2) 集成测试或脚本：/health 与 /status 不依赖真实密钥；真实密钥流程以文档说明提供。
完成后输出：测试清单、运行命令、以及如何在 CI 中执行。
```

