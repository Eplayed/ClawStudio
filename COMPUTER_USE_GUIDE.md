# Computer Use Integration Guide

## 概述

ClawStudio Nova 现已集成 Computer Use (CU) 功能，允许 Claude AI Agent 通过 Anthropic Messages API 与沙盒桌面交互。

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                   ClawStudio Frontend                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Overwatch    │  │ VisualStream │  │  HITLBar     │  │
│  │ (CU Events)  │  │ (CU Screen)  │  │ (CU Approve) │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
│         │                 │                  │          │
│  ┌──────▼─────────────────▼──────────────────▼────────┐ │
│  │         CU Store (computer_use.ts)                 │ │
│  │  - Session management                             │ │
│  │  - Step execution                                 │ │
│  │  - HITL coordination                              │ │
│  └──────┬──────────────────────────────────────────┬─┘ │
└─────────┼──────────────────────────────────────────┼────┘
          │                                          │
          │ Tauri IPC                                │
          │                                          │
┌─────────▼──────────────────────────────────────────▼────┐
│              Rust Backend (Tauri)                       │
│  ┌──────────────────────────────────────────────────┐  │
│  │  CU Runtime (computer_use.rs)                    │  │
│  │  - Tool loop management                         │  │
│  │  - Anthropic API calls                          │  │
│  │  - HITL interception                            │  │
│  │  - Tool execution (computer/bash/text_editor)   │  │
│  └──────┬──────────────────────────────────────────┘  │
│         │                                              │
│  ┌──────▼──────────────┐  ┌──────────────────────┐   │
│  │  Docker Manager     │  │  VNC Screenshot      │   │
│  │  (Sandbox control)  │  │  (Screen capture)    │   │
│  └─────────────────────┘  └──────────────────────┘   │
│         │                                              │
│  ┌──────▼──────────────────────────────────────────┐  │
│  │  Anthropic API Client                          │  │
│  │  - Messages API v1                             │  │
│  │  - Tool definitions                            │  │
│  │  - Response parsing                            │  │
│  └──────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
          │
          │ HTTPS
          │
┌─────────▼──────────────────────────────────────────────┐
│         Anthropic Messages API                         │
│  - claude-3-5-sonnet-20241022 (recommended)           │
│  - Computer use tool definitions                      │
│  - Tool loop orchestration                            │
└──────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. Rust 后端 (`computer_use.rs`)

**主要类型：**
- `CUSession` - 会话状态
- `CUMessage` - 消息格式
- `CUContentBlock` - 内容块（text/tool_use/tool_result/image/thinking）
- `ComputerAction` - 桌面操作（screenshot/mouse/keyboard）
- `HITLAction` - HITL 拦截决策

**主要方法：**
```rust
// 启动会话
pub async fn start_session(...) -> Result<String, String>

// 执行一步 tool loop
pub async fn step(...) -> Result<CUStepResult, String>

// HITL 控制
pub async fn pause(session_id) -> Result<(), String>
pub async fn resume(session_id, user_response) -> Result<(), String>
pub async fn stop(session_id) -> Result<(), String>
```

**Tool Loop 流程：**
```
1. 调用 Anthropic API
2. 解析响应中的 tool_use
3. 检查 HITL 拦截规则
4. 执行工具（computer/bash/text_editor）
5. 收集 tool_result
6. 发送回 API
7. 重复直到 end_turn
```

### 2. 前端 TypeScript (`computer_use.ts` + `computer_use_store.ts`)

**API 客户端：**
```typescript
// 启动会话
await CUClient.startSession(agentId, sandboxId, vncPort, ...)

// 执行步骤
const result = await CUClient.step(sessionId, apiKey, permLevel)

// HITL 控制
await CUClient.pause(sessionId)
await CUClient.resume(sessionId, userResponse)
await CUClient.stop(sessionId)
```

**Pinia Store：**
```typescript
const cuStore = useCUStore()

// 启动会话
const sessionId = await cuStore.startSession(...)

// 执行步骤
const result = await cuStore.step(sessionId, apiKey, permLevel)

// 获取活跃会话
const session = cuStore.activeSession
```

## 使用流程

### 第 1 步：创建 Agent 并启用 Computer Use

在 `CreateAgentModal.vue` 中：
1. 打开 "Computer Use" 开关
2. 选择一个沙盒环境
3. 创建 Agent

### 第 2 步：启动 CU 会话

```typescript
// 在 Overwatch.vue 中
import { useCUStore } from '@/stores/computer_use'
import { useSettingsStore } from '@/stores/settings'

const cuStore = useCUStore()
const settingsStore = useSettingsStore()

// 启动会话
const sessionId = await cuStore.startSession(
  agentId,
  sandboxId,
  vncPort,
  systemPrompt,
  settingsStore.cuModel,  // 推荐 claude-3-5-sonnet-20241022
  apiKey
)
```

### 第 3 步：执行 Tool Loop

```typescript
// 循环执行步骤
while (true) {
  const result = await cuStore.step(
    sessionId,
    apiKey,
    settingsStore.permLevel  // 'browse' | 'standard' | 'auto'
  )

  // 更新 UI
  // - ThoughtLog 显示 result.response
  // - VisualStream 显示截图
  // - 成本累加 result.cost

  if (result.paused) {
    // 等待 HITL 审批
    break
  }

  if (result.response.content.some(c => c.type === 'end_turn')) {
    // 会话结束
    break
  }
}
```

### 第 4 步：HITL 审批

```typescript
// 用户在 HITLBar 中审批
if (userApproved) {
  await cuStore.resume(sessionId, "Approved by user")
  // 继续 tool loop
} else {
  await cuStore.pause(sessionId, "Rejected by user")
  // 或者
  await cuStore.stop(sessionId)
}
```

## HITL 拦截规则

### Computer 工具
- `screenshot` → 总是允许
- `mouse_move` → 总是允许
- `left_click` / `right_click` → 通常允许
- `type` / `key` → 如果 permLevel='browse' 则询问

### Bash 工具
- 危险命令（`rm -rf`, `sudo`, `curl | sh`）→ 总是拒绝
- 其他命令 → 根据 permLevel 决定
  - `browse` → 询问
  - `standard` → 询问
  - `auto` → 允许

### Text Editor 工具
- `browse` → 询问
- 其他 → 允许

## 成本计算

每步的成本包括：
- **输入 tokens**：文本 + 截图
  - 1 张截图 ≈ 1590 tokens ≈ $0.0048 (Claude Sonnet)
  - 文本：~4 chars/token
- **输出 tokens**：API 响应
  - 工具调用 + 思考过程

**优化建议：**
1. 降低截图频率（不是每步都截图）
2. 使用较小的分辨率（800×600 而非 1280×800）
3. 设置 token 限制（max_tokens: 4096）
4. 监控 budgetLimit，超限自动停止

## 集成检查清单

- [x] `computer_use.rs` - CU Runtime 实现
- [x] `computer_use.ts` - 前端 API 客户端
- [x] `computer_use_store.ts` - Pinia Store
- [x] `main.rs` - 注册 Tauri commands
- [x] `Cargo.toml` - 添加 reqwest 依赖
- [ ] VNC 截图实现（需要 noVNC WebSocket 集成）
- [ ] Docker exec 工具实现（bash/text_editor）
- [ ] Overwatch.vue 集成 CU UI
- [ ] HITLBar.vue 扩展 CU 审批
- [ ] ThoughtLog.vue 扩展 CU 事件渲染
- [ ] VisualStream.vue 集成 CU 截图流
- [ ] 成本计算集成 CU token 费用
- [ ] 数据库持久化 CU 会话历史

## 下一步

### 第 2 步：VNC 截图实现
需要通过 WebSocket 连接到 noVNC，获取 FramebufferUpdate 并转换为 PNG。

### 第 3 步：工具执行实现
- `computer` → VNC PointerEvent / KeyEvent
- `bash` → docker exec
- `text_editor` → docker exec + 文件操作

### 第 4 步：前端 UI 集成
- Overwatch 页面添加 CU 控制面板
- VisualStream 实时显示 CU 截图
- ThoughtLog 显示 CU 工具调用
- HITLBar 显示 CU 操作审批

### 第 5 步：测试与优化
- 端到端测试（创建 Agent → 启动 CU → 执行任务）
- 性能优化（截图压缩、缓存、并发）
- 成本优化（token 限制、采样率）

## 参考资源

- [Anthropic Computer Use API](https://docs.anthropic.com/en/docs/build-a-system-with-claude/computer-use)
- [noVNC Protocol](https://github.com/novnc/noVNC)
- [Docker API](https://docs.docker.com/engine/api/)
- [Tauri Commands](https://tauri.app/develop/calling-rust/)
