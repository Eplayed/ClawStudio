# ClawStudio v2.0 - 增量开发计划与 AI 编程提示词

> **文档说明**：本计划基于 `Eplayed/ClawStudio` 现有的 Tauri + Vue 3 骨架制定。目标是打通底层 Rust 进程控制与前端 Vue 的交互闭环，实现“开箱即用”和“可视化监控”的核心卖点。

## 📅 第一部分：任务拆解与开发计划 (Phases)

### Phase 1: 核心地基 —— 环境向导与依赖检测 (Setup Engine)
*由于 ClawStudio 的定位是小白一键可用，必须接管 Node.js 和 OpenClaw 的安装。*
*   **功能描述**：
    *   在 `src-tauri` 中新增 `setup.rs` 模块，利用 Rust 的 `which` 或 `std::process::Command` 检测宿主机环境。
    *   在前端完善 `SetupWizard.vue`，实现环境检测的状态机（Loading -> 成功/失败 -> 引导安装）。
    *   实现 Tauri Event 推送，将底层的安装进度（stdout 日志流）实时打印在前端的“伪终端”UI 中。

### Phase 2: 核心引擎 —— Gateway 守护进程控制 (Daemon Manager)
*ClawStudio 需要像 Docker Desktop 管理 dockerd 一样管理 openclaw 进程。*
*   **功能描述**：
    *   在 `src-tauri` 中新增 `gateway.rs`。实现 `start_gateway`、`stop_gateway` 和 `health_check`。
    *   使用 Rust 的 `std::process::Child` 将 OpenClaw 作为后台子进程拉起，并持有关联的 Handle (通过 `std::sync::Mutex` 存入 Tauri 的 AppState)，确保退出主界面时子进程被正确清理。
    *   前端 Dashboard 顶部完善“全局状态条”，每 5 秒轮询一次健康状态，显示绿灯（运行中）或红灯（已停止）。

### Phase 3: 杀手锏 —— 视控舱与 VNC 占位 (Overwatch Console)
*完成 PRD 中最具视觉冲击力的双屏监控面板。*
*   **功能描述**：
    *   在 `Overwatch.vue` 中实现双面板：左侧 `ActionLog`（动作流），右侧 `VisualStream`（视觉流）。
    *   **暂不强行集成真实 VNC**，第一版先写一套精美的 **Mock (伪造) 数据引擎**。让前端循环播放几张屏幕截图，并通过 CSS 绝对定位+过渡动画 (`transform: translate`) 在截图上模拟一个“鼠标准星（Crosshair）”的移动，配合左侧滚动的 JSON 日志，制造出极强的科技感。
    *   实现 `HITL (Human-in-the-loop)` 拦截弹窗的 UI 组件（当 Mock 数据抛出 `status: waiting_approval` 时触发）。

### Phase 4: 成本监控与闭环 (Cost Monitor & Settings)
*   **功能描述**：
    *   开发 `CostMonitor.vue`，实现“预算油表”。提供 Input Token 和 Output Token 的环形图拆解。
    *   在 `Settings.vue` 中打通对 `~/.openclaw/openclaw.json` (或 YAML) 的读写，实现 API Key 的保存和底层重启。

---

## 🤖 第二部分：AI 编程助手专属提示词 (Prompts)

> **使用方法**：打开 Cursor (或 GitHub Copilot Chat)，先发送 **[全局系统提示词]** 设定上下文，然后针对具体任务按顺序发送 **[任务提示词]**。

### 0. 设定 AI 全局角色 (System Prompt)
```text
你现在是 ClawStudio 开源项目的资深架构师和全栈工程师。
我们的代码库基于 Tauri 2.0 (Rust) + Vue 3 (Composition API, script setup) + TypeScript + Tailwind CSS。
设计语言是 "Mission Control Dark" (暗黑极客风，主色调为深蓝背景配青色/荧光绿点缀)。
我们的产品目标是：将基于 CLI 的 OpenClaw Agent 框架包装成一个类似 Docker Desktop 的一键式桌面客户端。

接下来的开发要求：
1. Rust 代码必须充分处理错误 (Result)，禁止直接 unwrap() 导致应用崩溃。跨线程状态共享请使用 Mutex 和 Tauri AppState。
2. Vue 前端尽量复用现有的 Tailwind 工具类，注重科幻感和微交互动画。
3. 必须考虑跨平台兼容性 (Windows 的 .cmd 和 macOS/Linux 的 sh)。

请回复“已了解代码库上下文，准备开始开发”，我将发送第一个模块的任务。
```
1. 开发 Setup Wizard (Rust 环境检测篇)

```text
【当前任务：实现底层环境检测】
请在 `src-tauri/src` 目录下新建 `setup.rs`。
需求：
1. 编写一个 Tauri Command `check_environment() -> Result<EnvStatus, String>`。
2. `EnvStatus` 包含: `has_node` (bool), `node_version` (String), `has_openclaw` (bool), `openclaw_version` (String)。
3. 使用 `std::process::Command` 执行 `node -v` 和 `openclaw --version`。
4. 细节处理：在 Windows 下要兼容执行 `node.exe` 和 `openclaw.cmd`。如果命令未找到（如 ErrorKind::NotFound），不要报错，而是优雅地返回 `has_node: false`。
5. 请给出 `setup.rs` 的完整代码，并告诉我需要在 `main.rs` 中添加什么代码来注册这个模块和 command。
```
2. 开发 Gateway 守护进程管理 (Rust 后端控制篇)

```text
【当前任务：接管 OpenClaw 底层服务】
我们需要在 Rust 端控制 `openclaw gateway start` 进程。请新建 `src-tauri/src/gateway.rs`。
需求：
1. 定义一个全局的 Tauri State (如 `GatewayState { process: Mutex<Option<Child>> }`) 用于持有子进程的句柄。
2. 编写 command `start_gateway(port: u16)`：使用 `std::process::Command` 以子进程形式拉起 `openclaw gateway start --port <port>`。将拉起的 Child 进程存入 State 中。
3. 编写 command `stop_gateway()`：从 State 中获取 Child 并调用 `kill()` 终止进程。
4. 编写 command `check_health(port: u16) -> bool`：发送一个轻量级的 HTTP GET 请求到 `http://127.0.0.1:<port>/healthz` (可使用 reqwest 库)，返回 true/false。
5. 代码必须确保在 Tauri 应用退出时，能自动 kill 掉这个子进程，不留孤儿进程。
请给出健壮的 Rust 代码及依赖项说明。
```
3. 开发视控舱 Overwatch (Vue UI 核心面板篇)

```text
【当前任务：实现“视控舱”双面板与 Mock 动画】
请在 `src/views/Overwatch.vue` (或相应的组件) 中实现界面布局。
需求：
1. 界面分为左右结构：左侧 40% 宽度为 `ActionLog` (思维流)，右侧 60% 宽度为 `VisualStream` (视觉屏幕)。
2. **ActionLog**：写一个可以自动滚动到底部的日志列表。请用 TypeScript 模拟一个定时器，每 2 秒向日志数组 push 一条假数据（例如：`{ type: 'think', text: '正在识别页面元素...' }`, `{ type: 'action', text: '点击了 登录 按钮' }`），并用不同的文字颜色区分。
3. **VisualStream**：右侧居中放置一个相对定位 (`relative`) 的区域。背景放一张假的桌面截图。
4. **准星动画 (Crosshair)**：在这个截图区域内，放置一个绝对定位的 `div`（设计成一个科技感的红色准星 🎯）。通过 Vue 的响应式变量 `cursorX` 和 `cursorY` 绑定它的 `left` 和 `top`。写一个定时器，随机改变这两个值，并加上 CSS `transition: all 0.5s ease-in-out`，让用户感觉 AI 正在移动鼠标！
请用最酷炫的 Tailwind CSS 样式实现这个组件代码。
```
4. 开发 HITL (Human-in-the-loop) 拦截弹窗

```text
【当前任务：实现高危操作的人工审批拦截 UI】
请创建一个全局可复用的 Vue 组件 `components/HitlModal.vue`。
需求：
1. 这是一个全屏覆盖的遮罩层 (backdrop-blur-sm)，整体带有红色/黄色的警告氛围警示线设计 (`border-red-500`)。
2. 标题居中：“⚠️ 敏感操作拦截”。
3. 接收 props：`agentName` (AI名称), `actionType` (如: 支付 / 删除文件), `reason` (AI为什么要执行此操作)。
4. 底部动作区要有三个醒目的按钮：
   - 🔴 [拒绝并挂起进程] (红色描边)
   - 🟡 [接管并人工输入] (黄色描边)
   - 🟢 [允许执行] (绿色实心，带发光特效)
5. 弹窗出现时需要有明显的入场动画（如缩放或从上方掉落）。
请提供该组件的完整 Vue 3 代码。
```
5. 开发 Cost Monitor (预算油表与 Token 消耗)

```text
【当前任务：实现 Token 燃烧可视化】
我们需要在页面中加入一个展示大模型 API 成本的看板。
需求：
1. 在顶部或侧边栏设计一个“燃烧器/油表” UI。
2. 显示字段：当前任务已消耗金额（如 $1.25），预算上限（如 $5.00）。
3. 设计一个进度条：用渐变色表示（0-50% 绿色，50-80% 橙色，80% 以上红色带呼吸动画闪烁）。
4. 下方用极简的排版列出：`Input Tokens: 45,000` (主要因为 Computer Use 的截图传输), `Output Tokens: 2,300`。
请用纯 Tailwind CSS (不依赖重型图表库) 实现这个进度条和数据展示面板。
```

```text
### 💡 针对现有代码库的实操建议：

1. **善用 Cursor 的 @ 功能**：在执行上述 Prompt 时，如果涉及修改现有页面，请务必在 Cursor 的聊天框中输入 `@src/views/Overwatch.vue` 或 `@src-tauri/src/main.rs`，让 AI 先阅读你现有的代码上下文，然后再粘贴 Prompt。
2. **先做 Mock，再做底层**：强烈建议您**先执行 Prompt 3 和 4（纯前端的视控舱和弹窗）**。因为这个界面的视觉冲击力最强。一旦你在现有的暗黑主题里跑通了“自动乱跑的红框准星”和“拦截弹窗”，项目的逼格瞬间拉满，截几张图发到 GitHub README 上，很容易就能吸引到懂 Rust 的开发者来帮你一起写底层代码。
3. **Rust 跨平台坑点预警**：在执行 Prompt 1 和 2 时，`std::process::Command` 在 Windows 下直接调用 `openclaw` 会报错，必须让 AI 加上 `.cmd` 后缀或者使用 `cmd /c openclaw`。AI 如果写错了，直接把终端报错丢给它，它会自动修正。

按照这份结合了现有代码库现状的增量计划推进，ClawStudio 绝对能以极快的速度完成 v2.0 MVP 的交付！
```