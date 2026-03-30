# ClawStudio v2.0 - 增量开发计划与 AI 编程提示词
> **文档说明**：本计划基于 `Eplayed/ClawStudio` 现有的 Tauri + Vue 3 骨架制定，并**深度对齐 PRD 2.0 的商业化与生态防御战略（Open-Core / 审计留痕 / 模板裂变）**。目标不仅是打通底层，更是构建起防抄袭的代码护城河。

## 📅 第一部分：任务拆解与开发计划 (Phases)

### Phase 0: 战略防御基建与开源协议 (Strategic Infrastructure)
*兵马未动，协议先行。确立 Open-Core 架构，为未来的企业版 SaaS 变现留出接口。*
*   **功能描述**：
    *   在仓库根目录添加 `AGPL-3.0` License 文件。
    *   在 `src-tauri/src` 下建立 `enterprise` 目录（预留为空或定义未实现的 trait），确立核心业务逻辑与商业化逻辑的物理隔离。

### Phase 1: 核心地基 —— 环境向导与依赖检测 (Setup Engine)
*实现零门槛“一键安装”，让文科生也能在 5 分钟内部署 AI 助手。*
*   **功能描述**：
    *   新增 `setup.rs`，利用 Rust 的 `which` 或 `std::process::Command` 检测宿主机 Node.js 和 OpenClaw 环境。
    *   在前端完善 `SetupWizard.vue`，实现环境检测状态机（Loading -> 引导安装）。
    *   实现 Tauri Event 推送，将底层的 `npm install` 进度（stdout 日志流）实时打印在前端 UI。

### Phase 2: 核心引擎 —— Gateway 守护进程控制 (Daemon Manager)
*接管 OpenClaw 生命周期，杜绝野进程，确保系统安全。*
*   **功能描述**：
    *   新增 `gateway.rs`。实现 `start_gateway`、`stop_gateway` 和 `health_check`。
    *   使用 Rust `std::process::Child` 将 OpenClaw 作为后台子进程拉起，将 Handle 存入 Tauri AppState 的 Mutex 中，退出应用时自动 Kill。
    *   Dashboard 顶部实现“全局状态条”，每 5 秒轮询健康状态。

### Phase 3: 杀手锏 —— 视控舱与 HITL 拦截 (Overwatch & HITL)
*完成最具视觉冲击力、且最具安全价值的双屏监控面板。*
*   **功能描述**：
    *   在 `Overwatch.vue` 中实现双面板：左侧 `ActionLog`（思维流），右侧 `VisualStream`（视觉流/VNC占位区）。
    *   实现极其醒目的 `HITL (Human-in-the-loop)` 人工拦截弹窗组件（支持“拒绝”、“纠正”和“允许”）。

### Phase 4: 商业化基石 —— 成本监控与合规审计 (Cost & Audit Logs)
*落实 PRD 2.0 中针对 B 端企业用户的杀手级功能：留痕与算账。*
*   **功能描述**：
    *   开发 `CostMonitor.vue`，实现“预算油表”，加入 API 费用到达阈值自动挂起的熔断机制。
    *   **【新增】** 在 `src-tauri` 中引入 SQLite (使用 `rusqlite` 或 `sqlx`)，建立 `audit_logs` 表。将 AI 所有的动作记录、Token 消耗、甚至屏幕关键帧路径存入本地数据库，实现只读的 `AuditTraces.vue` 回放面板。

### Phase 5: 生态护城河 —— 模板分享与裂变 (Ecosystem & Viral Growth)
*通过“一键导入/导出”打造 Agent 技能市场雏形。*
*   **功能描述**：
    *   在 `Agents` 页面增加“导出为模板”功能，将当前 Agent 的 System Prompt、环境参数打包为 Base64 或专有后缀文件（`.claw-template`）。
    *   支持用户将 `.claw-template` 文件拖拽进 ClawStudio 界面直接完成 Agent 创建。

---

## 🤖 第二部分：AI 编程助手专属提示词 (Prompts)

> **使用方法**：打开 Cursor (或 GitHub Copilot/Windsurf)，先发送 **[全局系统提示词]**，然后针对具体任务按顺序发送对应的 **[任务提示词]**。执行跨文件修改时，记得用 `@` 引用相关文件给 AI 看。

### 0. 设定 AI 全局角色 (System Prompt)
```text
你现在是 ClawStudio 开源项目的资深架构师和全栈工程师。
我们的代码库基于 Tauri 2.0 (Rust) + Vue 3 (Composition API) + TypeScript + Tailwind CSS。
设计语言是 "Mission Control Dark" (极客暗黑风)。
核心商业战略：采用 Open-Core 模式，本地社区版完全开源 (AGPL-3.0)，为未来的云端 SaaS 企业版预留清晰的接口隔离。

开发红线：
1. Rust 代码必须充分处理 Result，禁止直接 unwrap()。跨线程状态使用 Mutex 和 Tauri AppState。
2. 架构上需具备“防御性编程”思维，任何对外部命令的调用都必须有超时和错误回退机制。
3. 界面交互必须考虑非技术小白用户的体验，文案通俗易懂。

请回复“已了解商业战略与代码库上下文，准备开始”，我将发送具体的模块开发需求。
```
1. 开发 Setup Wizard (Rust 环境检测与日志流推送)
```text
【任务：实现底层环境检测与安装进度推送】
请在 `src-tauri/src/setup.rs` 中编写 Tauri Command。
需求：
1. 编写 `check_environment() -> Result<EnvStatus, String>`，检测 node 和 openclaw 是否安装。
2. 编写 `install_openclaw(window: tauri::Window)`：通过 `std::process::Command` 执行 `npm install -g openclaw@latest`。
3. **关键**：必须将子进程的 stdout/stderr 逐行捕获，并通过 `window.emit("install-progress", payload)` 实时推送到前端。
4. 请处理好 Windows 下执行 `.cmd` 和 macOS 执行 `sh` 的跨平台差异。
提供 Rust 代码及 `main.rs` 中的注册方法。
```
2. 开发 Gateway 守护进程管理 (Rust 后端控制篇)
```text
【任务：接管 OpenClaw 底层服务，确保不留僵尸进程】
请新建 `src-tauri/src/gateway.rs`。
需求：
1. 定义全局 State `GatewayState { process: Mutex<Option<Child>> }`。
2. `start_gateway(port: u16)`：拉起 `openclaw gateway start --port <port>`，将 Child 存入 State。
3. `stop_gateway()`：获取 Child 并调用 `kill()` 终止进程。
4. `check_health(port: u16) -> bool`：向 `http://127.0.0.1:<port>/healthz` 发送轻量级 HTTP GET。
5. **安全要求**：在 Tauri 的 `RunEvent::Exit` 或类似的退出生命周期中，必须确保调用 kill 彻底清理这个后台进程。
请给出健壮的 Rust 代码。
```
3. 开发视控舱 Overwatch 与 HITL 拦截 (Vue UI 核心)
```text
【任务：实现“视控舱”双面板与高危操作审批拦截 UI】
在 `src/views/Overwatch.vue` 中实现界面：
1. 左侧 40% `ActionLog` (思维流)：自动滚动到底部的 JSON/文本日志流，用 Tailwind 颜色区分 (think 灰色, action 青色, error 红色)。
2. 右侧 60% `VisualStream` (视觉屏幕)：相对定位区域，放一张假桌面截图。使用 Vue 响应式变量绑定一个绝对定位的“红色准星 (Crosshair)”，加上 `transition` 动画，用定时器随机移动它，模拟 AI 动鼠标。
3. **编写 HITL 拦截组件**：在屏幕正中央绝对定位一个醒目的 Modal (带红色/橙色发光边框，`backdrop-blur`)。标题“⚠️ 敏感操作拦截”。包含操作描述和三个大按钮：[🔴 拒绝并挂起], [🟡 接管输入],[🟢 允许执行]。
请用最具科幻控制台风格的 Tailwind CSS 编写这套 UI。
```
4. 开发合规审计与成本中心 (Audit Logs & Cost)
```text
【任务：落实 B 端核心商业功能——审计留痕】
在 `src-tauri/src/audit.rs` 中引入 SQLite 支持 (建议使用 rusqlite)。
需求：
1. 编写建表逻辑：`CREATE TABLE IF NOT EXISTS audit_logs (id INTEGER PRIMARY KEY, agent_id TEXT, action_type TEXT, description TEXT, token_cost REAL, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)`。
2. 提供 `insert_audit_log` 和 `get_recent_logs` 的 Tauri Command。
3. 在前端新建 `src/views/AuditTraces.vue`，用企业级后台数据表格的形式展示这些日志。
4. 在页面顶部增加“成本中心”油表：统计表内今天的 `token_cost` 总计，配合进度条展示（例如预算 $5.00，当达到 $4.50 时进度条变红闪烁）。
请提供 Rust 数据库逻辑和 Vue 表格展示代码。
```
5. 开发生态裂变机制 (模板导入导出)
```text
【任务：实现 Agent 模板的分享与生态裂变】
在前端 Agent 管理页面实现模板打包逻辑。
需求：
1. 编写一个 TypeScript 工具函数 `exportTemplate(agentConfig)`。将包含大模型偏好、System Prompt、运行沙盒参数的 JSON 对象，序列化并转化为 Base64 字符串，拼接上前缀 `claw://template/`。
2. 生成一个可下载的 `.claw-template` 文本文件。
3. 在界面上实现一个**拖拽上传区域**：用户将 `.claw-template` 文件拖入应用内时，解析 Base64 并反序列化为 JSON，弹出“是否导入此 Agent 模板？”的确认框。
请给出优雅的前端实现方案。
```