# 🏁 Phase 7: 终极冲刺与架构压测 (Final Sprint)

> 本阶段致力于完成 PRD v2.1 架构下的最后几项收尾工作。核心目标是打通前端设置与后端网关的最后 100 米，处理流式请求的边界异常，并完成集成压测。

---

## ✅ Task 7.1: 完成 `set_proxy_budget_limit` 预算上限接口
**状态**: ✅ 已完成  
**目标文件**: `src-tauri/src/proxy.rs` & `src-tauri/src/main.rs`  
**需求描述**: 
允许前端动态修改代理层的成本拦截阈值。当用户在 `CostMonitor.vue` 调高预算上限时，后端必须具备“解除熔断”的自愈能力。

### 🤖 AI 开发提示词 (Prompt)
```text
【当前任务：实现 set_proxy_budget_limit 后端逻辑】
请在 src-tauri/src/proxy.rs 中补充 set_proxy_budget_limit Tauri Command。
需求细节：
1. 接收参数: `limit_usd` (f64) 和 `state` (Tauri State, 即 ProxyState)。
2. 将 `limit_usd` 转换为内部原子级计费单位（例如乘以 10000 转换为 i64）。
3. 存入 `state.budget_limit` (使用 AtomicI64::store 保证线程安全)。
4. 熔断自愈逻辑：比较当前已消费金额 (`total_cost`) 与新的 `budget_limit`。
   - 如果 total_cost < new_limit，请自动解除 circuit_broken 状态 (store false)。
   - 否则保持熔断状态。
5. 返回一个 Result<String, String>，说明更新状态。
最后，请告诉我需要在 src-tauri/src/main.rs 的 invoke_handler 中添加什么代码来注册它。

✅ Task 7.2: Setup Wizard 代理网关双重启动融合 (原 Task 2.2 收尾)
状态: ✅ 已完成
目标文件: src/components/setup/LaunchGateway.vue
需求描述:
在安装向导的最后一步，不能仅仅启动 OpenClaw，而是要按顺序完成“强行劫持配置 -> 启动本地 Proxy -> 启动 OpenClaw”，并在 UI 上给用户明确的安全接管感知。
🤖 AI 开发提示词 (Prompt)
code
Text
【当前任务：Setup Wizard 代理网关双重启动融合】
请修改 src/components/setup/LaunchGateway.vue 组件。
需求细节：
1. UI 调整：在界面上展示两个独立的状态灯（使用 Tailwind CSS 样式）：
   - 🛡️ 视控舱安全网关 (Port: 18788) -> [⏳ 等待启动 / ✅ 运行中]
   - 🦞 OpenClaw 核心引擎 (Port: 18789) ->[⏳ 等待启动 / ✅ 运行中]
2. 核心异步拉起逻辑：当用户点击“启动引擎”大按钮时，按严格顺序执行以下 invoke：
   - await invoke('configure_openclaw_proxy', { port: 18788 }); // 强制劫持底层 LLM Base URL
   - await invoke('start_proxy', { port: 18788 }); // 拉起本地 API 代理服务器
   - 更新网关状态灯为运行中。
   - await invoke('start_gateway', { port: 18789 }); // 拉起 OpenClaw 后台进程
   - 更新引擎状态灯为运行中。
3. 全部就绪后，延时 1.5 秒，使用 vue-router 跳转进入 /dashboard。
请确保代码具有完善的 try-catch 错误处理，失败时在界面显示红色的错误提示。
✅ Task 7.3: 代理层流式传输 (Streaming) 强制降级处理
状态: ✅ 已完成
目标文件: src-tauri/src/proxy.rs (请求拦截中间件)
需求描述:
如果 OpenClaw 开启了流式输出 (stream: true)，由于 SSE 数据的分块特性，会导致我们的 Proxy 拦截器无法一次性解析完整的 JSON 进而引发解析失败或死锁。必须在请求发给大模型之前，强行篡改 Body 禁用流式。
🤖 AI 开发提示词 (Prompt)
code
Text
【当前任务：代理层流式传输强行降级拦截】
请修改 src-tauri/src/proxy.rs 中处理 HTTP POST 转发的 handler 函数（即处理 OpenClaw 发来请求的地方）。
需求细节：
1. 在读取到原始请求的 body bytes 后，尝试使用 `serde_json::from_slice` 将其解析为可变的 `serde_json::Value`。
2. 查找 JSON 对象中是否存在 `"stream"` 字段。
3. 如果存在且为布尔值，强行将其修改为 `false` (即 Value::Bool(false))。
4. 将篡改后的 JSON 重新序列化为 bytes，然后再使用 reqwest 发送给真实的大模型 API (如 Anthropic/OpenAI)。
5. 如果 body 不是合法的 JSON，记录一条 warn 日志并原样转发（不作拦截报错，保证高可用性）。
请给出健壮的 Rust 篡改逻辑代码。
✅ Task 7.4: 架构链路自动化压测验证 (HITL & Cost)
状态: ✅ 已完成
目标文件: tests/mock_hitl_request.sh (新建测试脚本)
需求描述:
提供一个可执行的测试脚本，用于在不启动 OpenClaw 的情况下，直接向 Proxy 网关发送伪造的大模型高危响应，以此验证 Vue 前端的 HITL 拦截弹窗和 Cost Monitor 计费器是否能被瞬间唤醒。
🤖 AI 开发提示词 (Prompt)
code
Text
【当前任务：编写架构链路集成压测脚本】
请在项目根目录新建一个文件 `tests/mock_hitl_request.sh`。
需求细节：
这是一个 bash 脚本，使用 curl 向 http://127.0.0.1:18788/v1/messages 发送 POST 请求。
请求体需要模拟一个完美的大模型返回结果（假设它是 Anthropic 格式或者被 proxy 处理兼容的格式）：
1. 包含一句话的文本思考流。
2. 包含一个 tool_use，工具名称是 "bash"，命令是 "rm -rf /"。
3. 包含 usage 字段：input_tokens: 15000, output_tokens: 500。
请在文件头部用注释说明：执行此脚本后，预期 ClawStudio 的 UI 会立即弹出红色拦截框，并且右上角油表会增加相应费用。如果用户在 UI 点拒绝，终端的 curl 应收到含有 error 的响应。
code
Code
### 💡 如何高效清空这些 ⏳ 任务？
1. 直接在 Cursor 的 Chat 侧边栏，新建一个会话（Chat）。
2. 输入 `@src-tauri/src/proxy.rs` 或其他对应的目标文件。
3. 直接粘贴上方对应的 `[🤖 AI 开发提示词]` 代码块。
4. AI 生成代码后，点击 `Apply` 检查差异，没问题直接保存，并将对应的 ⏳ 改为 ✅。