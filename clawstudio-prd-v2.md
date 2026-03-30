# ClawStudio Nova — PRD v2.0

> 一键安装、可视化监控、零门槛使用 OpenClaw 的桌面工作站

---

## 第一部分：头脑风暴 — 可行性分析

### 1.1 想法概述

将 OpenClaw 的安装、配置、运行、监控、卸载全流程集成到 ClawStudio GUI 中，让非技术用户也能一键部署私人 AI 助手，并通过可视化界面监控所有 Agent 行为。

### 1.2 技术可行性评估

| 环节 | 技术方案 | 可行性 | 风险点 |
|------|----------|--------|--------|
| **检测 Node.js** | Tauri shell 执行 `node --version` | ✅ 简单 | 无 |
| **安装 Node.js** | 下载官方安装包（.msi/.pkg/.deb）并执行 | ✅ 可行 | 需要管理员权限；中国网络可能需要镜像 |
| **安装 OpenClaw** | `npm install -g openclaw@latest` | ✅ 可行 | npm 全局安装在 Linux/macOS 可能需 sudo；中国需 npmmirror |
| **自动配置** | `openclaw onboard --non-interactive` + 各 flag | ✅ 可行 | OpenClaw 支持完整的非交互模式，所有参数可通过 flag 传入 |
| **启动服务** | `openclaw gateway start` 或 `--install-daemon` | ✅ 可行 | 需要对接三平台的服务管理器（launchd/systemd/Scheduled Tasks） |
| **健康检查** | HTTP GET `localhost:18789/healthz` | ✅ 简单 | 无 |
| **实时监控** | WebSocket 连接 `ws://127.0.0.1:18789` | ✅ 可行 | 需要解析 OpenClaw 的事件协议 |
| **卸载** | `openclaw uninstall --all --yes` | ✅ 可行 | OpenClaw 已提供完整卸载命令 |
| **版本升级** | `npm update -g openclaw` | ✅ 可行 | 需要处理大版本 breaking changes |

### 1.3 核心优势判断

**结论：完全可行，且是一个极有价值的产品方向。**

原因：
1. OpenClaw 的 CLI 已提供 `--non-interactive` 模式，所有配置可程序化传入
2. `openclaw uninstall --all --yes` 提供了干净的卸载路径
3. Gateway 暴露标准 WebSocket + HTTP 健康检查，天然适合 GUI 对接
4. Tauri 的 `tauri-plugin-shell` 可以执行任意系统命令并捕获 stdout/stderr
5. 市场上没有同类产品 — 目前 OpenClaw 只有 CLI 使用方式

### 1.4 潜在风险及应对

| 风险 | 影响 | 应对方案 |
|------|------|----------|
| 中国大陆 npm 速度慢 | 安装超时 | 内置 npmmirror 镜像切换选项；支持离线安装包 |
| Node.js 安装需管理员权限 | 用户困惑 | 提供权限提升引导弹窗；macOS 支持 brew 备选 |
| OpenClaw 大版本升级不兼容 | 功能失效 | 锁定兼容版本范围；升级前备份 `openclaw backup create` |
| 用户 API Key 安全 | 泄露风险 | 使用 OpenClaw 的 `ref` 模式存环境变量；配合 OS Keychain |
| 安装过程中断电/崩溃 | 半安装状态 | 实现安装状态机 + 回滚能力 |

---

## 第二部分：PRD v2.0 正文

### 2.1 产品定位

**PRD 1.0 定位：** AI Agent 可视化监控工作站（面向开发者）
**PRD 2.0 定位：** 一键 AI 助手平台 — 安装、配置、监控、管理全流程 GUI（面向所有人）

一句话描述：**ClawStudio 是 OpenClaw 的「Docker Desktop」**— 把命令行工具变成人人可用的桌面应用。

### 2.2 目标用户

| 用户类型 | PRD 1.0 | PRD 2.0 |
|----------|---------|---------|
| 开发者 / 技术人员 | ✅ 核心用户 | ✅ 保留 |
| 运营 / 产品经理 | ❌ 不覆盖 | ✅ 新增 — 通过 GUI 配置 Agent |
| 个人用户 / 小白 | ❌ 不覆盖 | ✅ 核心目标 — 一键安装，零 CLI |
| 小团队 / 工作室 | ❌ 不覆盖 | ✅ 新增 — 多 Agent 管理 + 成本控制 |

### 2.3 核心用户旅程

```
用户下载 ClawStudio 安装包（双击 .dmg / .msi / .AppImage）
        ↓
首次启动 → 进入 Setup Wizard（安装向导）
        ↓
Step 1: 检测环境（Node.js 是否存在）
        ↓ 不存在
Step 2: 一键安装 Node.js（带进度条）
        ↓
Step 3: 一键安装 OpenClaw（npm install -g，带进度条）
        ↓
Step 4: 配置 API Key（输入框 + 测试按钮）
        ↓
Step 5: 选择模型（下拉框 + 推荐标签）
        ↓
Step 6: 选择通道（可选：Telegram / WeChat / Discord...）
        ↓
Step 7: 启动 Gateway（一键按钮，显示启动日志）
        ↓
安装完成 → 自动跳转 Dashboard → 看到监控面板
        ↓
日常使用：Dashboard 查看状态 / Overwatch 监控行为 / Settings 调整配置
        ↓
不想用了 → Settings → 一键卸载（确认弹窗 → 清理一切）
```

### 2.4 信息架构（页面结构）

```
ClawStudio Nova v2.0
├── 🚀 Setup Wizard（首次安装向导）    ← 新增
│   ├── 环境检测
│   ├── Node.js 安装
│   ├── OpenClaw 安装
│   ├── API Key 配置
│   ├── 模型选择
│   ├── 通道配置（可选）
│   └── 启动确认
│
├── 📊 Dashboard（仪表盘）             ← 保留并增强
│   ├── OpenClaw 运行状态卡片          ← 新增
│   ├── 活跃 Agent 统计
│   ├── 今日费用 + 7 天趋势图
│   ├── 最近动态时间线
│   └── Agent 卡片网格
│
├── 🤖 Agents（特工列队）              ← 保留
│   ├── 看板视图
│   ├── 通道聚合器
│   └── 创建 Agent Modal
│
├── 📺 Overwatch（监控舱）             ← 保留
│   ├── 思维流
│   ├── 视觉流（模拟 / VNC）
│   └── HITL 审批栏
│
├── 💰 Cost Monitor（烧钱计算器）      ← 新增页面
│   ├── 预算油表
│   ├── Token 分解
│   ├── Agent 费用明细
│   └── 自动熔断设置
│
├── 📦 Environment（运行环境）         ← 重构
│   ├── OpenClaw 状态面板              ← 新增
│   │   ├── 版本 / Gateway 端口 / 运行时长
│   │   ├── 一键启动 / 停止 / 重启
│   │   └── 一键升级
│   ├── 运行模式切换（本机 / Docker）
│   ├── Docker 沙盒管理
│   ├── CPU / Memory 监控
│   └── 快捷配置
│
├── ⏱ Traces（历史回放）               ← 保留
│   ├── 回放播放器
│   └── 历史任务表格
│
├── ⚙ Settings（系统设置）             ← 增强
│   ├── API Key 管理
│   ├── 模型偏好
│   ├── HITL 安全设置
│   ├── 通道管理                       ← 新增
│   │   ├── 已连接通道列表
│   │   ├── 添加新通道（引导式）
│   │   └── 断开 / 重连
│   ├── 网络与代理
│   ├── OpenClaw 管理                  ← 新增
│   │   ├── 当前版本 + 检查更新
│   │   ├── 查看日志
│   │   └── 一键卸载
│   └── 关于
│
└── 🧙 首次引导覆盖层                  ← 新增
    └── 安装完成后的交互式教程
```

### 2.5 新增模块详细设计

#### 2.5.1 Setup Wizard（安装向导）— 核心新增

**入口条件：** ClawStudio 启动时检测 `openclaw --version`，若失败则强制进入向导。

**Step 1: 环境检测**

| 检测项 | 命令 | 通过条件 | 失败处理 |
|--------|------|----------|----------|
| Node.js | `node --version` | >= 22.14.0 | 跳转 Step 2 安装 |
| npm | `npm --version` | 存在即可 | 随 Node.js 安装 |
| OpenClaw | `openclaw --version` | 存在即可 | 跳转 Step 3 安装 |
| OpenClaw Gateway | `openclaw gateway status` | running | 跳转 Step 7 启动 |

UI: 四行检测项，每行左侧图标 + 名称 + 版本号/状态，右侧 ✅ / ❌ / ⏳ 动画。检测完自动决定从哪一步开始。

**Step 2: 安装 Node.js**

三平台方案：

| 平台 | 安装方式 | 命令 |
|------|----------|------|
| macOS | 官方 .pkg 或 brew | `brew install node@22` 或下载 .pkg |
| Windows | 官方 .msi | 下载并静默安装 `msiexec /i node-v22.x.msi /quiet` |
| Linux | NodeSource 或 nvm | `curl -fsSL https://deb.nodesource.com/setup_22.x \| sudo bash && sudo apt install -y nodejs` |

UI: 大按钮「安装 Node.js v22」+ 进度条 + 实时日志输出框（折叠式）。安装完后自动重新检测。

**备选方案开关：** 用户可选择「我已自行安装 Node.js」→ 重新检测 → 通过则跳过。

**Step 3: 安装 OpenClaw**

```bash
npm install -g openclaw@latest
```

UI: 大按钮「安装 OpenClaw」+ 进度条 + 日志。

**中国镜像选项：** 顶部 toggle「使用国内镜像加速」→ 切换为：
```bash
npm install -g openclaw@latest --registry=https://registry.npmmirror.com
```

**Step 4: 配置 API Key**

UI: 与现有 Settings 的密钥面板相同，但增加：
- 提供商 Tab 切换：Anthropic / OpenAI / 其他
- 输入框 + 「测试连接」按钮
- 测试通过显示绿色 ✅ + 可用模型列表
- 支持 「跳过，稍后配置」

底层调用：
```bash
openclaw onboard --non-interactive \
  --auth-choice anthropic \
  --secret-input-mode plaintext
```
然后将 key 写入 `~/.openclaw/openclaw.json` 的 agents.defaults 配置。

**Step 5: 选择模型**

UI: 卡片式选择器（非下拉框），每张卡片显示：
- 模型名 + 图标
- 价格标签（$3/M input, $15/M output）
- 推荐标签（适合大多数任务 / 高级推理 / 经济实惠）
- 选中后高亮

推荐排序：
1. Claude 3.5 Sonnet ← 默认推荐
2. GPT-4o
3. Claude 3 Opus
4. DeepSeek Chat ← 预算友好

**Step 6: 通道配置（可选）**

UI: 通道卡片网格，每张显示平台 logo + 名称 + 「连接」按钮。
- Telegram: 输入 Bot Token → 调用 `openclaw channels login telegram`
- Discord: 输入 Bot Token
- WeChat: 扫码（如支持）
- 其他通道灰显 + 「即将支持」

底部「跳过，直接开始」按钮（通道非必需）。

**Step 7: 启动 Gateway**

UI: 大型启动按钮（类似火箭发射），点击后：
1. 执行 `openclaw onboard --install-daemon --non-interactive`
2. 等待 `openclaw gateway status --require-rpc` 返回成功
3. 显示启动成功动画 + 「进入控制台」按钮

```
底层命令序列：
openclaw gateway start --port 18789
openclaw gateway health --url ws://127.0.0.1:18789
```

#### 2.5.2 OpenClaw 状态面板（Dashboard + Environment 联动）

**Dashboard 顶部新增状态条：**

```
┌─────────────────────────────────────────────────────┐
│ 🦞 OpenClaw v1.2.3  │  Gateway: ●运行中  │  Port: 18789  │  ↑ 2h 34m  │  [重启] [停止]  │
└─────────────────────────────────────────────────────┘
```

- 每 10 秒轮询 `localhost:18789/healthz`
- Gateway 停止时变红 + 显示「启动」按钮
- 右侧操作按钮：重启 / 停止 / 查看日志

**Environment 页面 OpenClaw 区块：**

| 信息 | 来源 |
|------|------|
| 版本号 | `openclaw --version` |
| Gateway 状态 | `openclaw gateway status` |
| 监听端口 | 配置文件 `gateway.port` |
| 运行时长 | 服务管理器 uptime |
| 已连接通道 | `openclaw channels list` |
| 日志路径 | `~/.openclaw/logs/` |

操作按钮：
- 一键启动 / 停止 / 重启 Gateway
- 检查更新 → `npm view openclaw version` 对比本地
- 一键升级 → `npm update -g openclaw`
- 查看日志 → 弹出 modal 显示最近 200 行

#### 2.5.3 通道管理（Settings 子面板）

| 功能 | 实现 |
|------|------|
| 查看已连接通道 | 解析 `~/.openclaw/openclaw.json` 的 channels 段 |
| 添加 Telegram | 引导输入 Bot Token → 写入配置 → 重启 gateway |
| 添加 Discord | 引导输入 Bot Token + App ID |
| 断开通道 | 从配置中移除对应段 → 重启 gateway |
| 测试通道 | `openclaw message send --to <channel> --message "ping"` |

#### 2.5.4 一键卸载（Settings）

UI: Settings 页面底部红色危险区域：

```
═══════════════════════════════════════════
⚠️ 危险操作 Danger Zone
═══════════════════════════════════════════
卸载 OpenClaw：彻底移除 OpenClaw 及所有数据

[一键卸载 OpenClaw]  ← 红色按钮
```

点击后弹出确认弹窗：
- 第一步：选择卸载范围
  - ☑ 停止并移除 Gateway 服务
  - ☑ 删除 OpenClaw CLI (`npm uninstall -g openclaw`)
  - ☑ 删除配置和数据 (`~/.openclaw/`)
  - ☐ 同时移除 Node.js（默认不勾选）
- 第二步：输入确认文字 "UNINSTALL" 防误操作
- 第三步：执行并显示进度

底层命令序列：
```bash
openclaw backup create                     # 先备份
openclaw gateway stop                      # 停止服务
openclaw gateway uninstall                 # 卸载服务
openclaw uninstall --all --yes             # 清理数据
npm uninstall -g openclaw                  # 移除 CLI
# 如勾选：也移除 Node.js
```

### 2.6 Rust 后端新增模块

#### 新增模块：`src-tauri/src/setup.rs`

```rust
// 环境检测与安装管理
#[tauri::command] fn detect_node() -> Result<NodeInfo>
#[tauri::command] fn detect_openclaw() -> Result<OpenClawInfo>
#[tauri::command] fn detect_gateway_status() -> Result<GatewayStatus>
#[tauri::command] fn install_node(platform: String) -> Result<()>         // 流式进度
#[tauri::command] fn install_openclaw(use_mirror: bool) -> Result<()>     // 流式进度
#[tauri::command] fn configure_openclaw(config: SetupConfig) -> Result<()>
#[tauri::command] fn start_gateway() -> Result<()>
#[tauri::command] fn stop_gateway() -> Result<()>
#[tauri::command] fn restart_gateway() -> Result<()>
#[tauri::command] fn check_gateway_health() -> Result<HealthStatus>
#[tauri::command] fn get_openclaw_version() -> Result<String>
#[tauri::command] fn check_openclaw_update() -> Result<UpdateInfo>
#[tauri::command] fn upgrade_openclaw() -> Result<()>                     // 流式进度
#[tauri::command] fn uninstall_openclaw(scope: UninstallScope) -> Result<()>
#[tauri::command] fn get_openclaw_logs(lines: usize) -> Result<Vec<String>>
#[tauri::command] fn list_channels() -> Result<Vec<ChannelInfo>>
#[tauri::command] fn add_channel(channel: ChannelConfig) -> Result<()>
#[tauri::command] fn remove_channel(channel_id: String) -> Result<()>
```

**关键设计：流式进度反馈**

安装 Node.js 和 OpenClaw 过程耗时较长，需通过 Tauri Event 实时推送进度：
```rust
window.emit("setup-progress", json!({
    "step": "install_openclaw",
    "percent": 45,
    "message": "Downloading packages...",
    "log_line": "npm WARN deprecated ..."
}));
```

前端监听 `setup-progress` 事件，更新进度条和日志面板。

#### 新增模块：`src-tauri/src/gateway.rs`

```rust
// Gateway 生命周期管理（独立于 openclaw.rs 的 Agent 进程管理）
#[tauri::command] fn gateway_start(port: u16) -> Result<()>
#[tauri::command] fn gateway_stop() -> Result<()>
#[tauri::command] fn gateway_health() -> Result<GatewayHealth>
#[tauri::command] fn gateway_logs(tail: usize) -> Result<String>
```

健康检查实现：
```rust
async fn check_health(port: u16) -> Result<GatewayHealth> {
    let url = format!("http://127.0.0.1:{}/healthz", port);
    let resp = reqwest::get(&url).await?;
    // 解析响应...
}
```

### 2.7 Vue 前端新增

#### 新增文件清单

| 文件 | 说明 |
|------|------|
| `src/views/SetupWizard.vue` | 安装向导主页面 |
| `src/components/setup/EnvDetector.vue` | 环境检测步骤 |
| `src/components/setup/NodeInstaller.vue` | Node.js 安装步骤 |
| `src/components/setup/ClawInstaller.vue` | OpenClaw 安装步骤 |
| `src/components/setup/ApiKeySetup.vue` | API Key 配置步骤 |
| `src/components/setup/ModelSelector.vue` | 模型选择步骤 |
| `src/components/setup/ChannelSetup.vue` | 通道配置步骤 |
| `src/components/setup/LaunchGateway.vue` | 启动确认步骤 |
| `src/components/GatewayStatusBar.vue` | Dashboard 顶部状态条 |
| `src/components/UninstallDialog.vue` | 卸载确认弹窗 |
| `src/components/ChannelManager.vue` | Settings 通道管理面板 |
| `src/stores/setup.ts` | 安装状态 Pinia store |

#### 路由变更

```typescript
// 新增
{ path: '/setup', name: 'setup', component: SetupWizard, meta: { fullscreen: true } }

// App.vue 中增加守卫逻辑：
// 如果 OpenClaw 未安装 → 重定向到 /setup
```

### 2.8 非功能性需求

| 项目 | 要求 |
|------|------|
| 安装总耗时 | < 5 分钟（100Mbps 网络） |
| 支持离线安装 | 提供 offline bundle（含 Node.js + OpenClaw .tgz） |
| 安装失败恢复 | 每步可重试，支持从断点继续 |
| 中国网络优化 | npmmirror 镜像、Node.js 镜像、检测网络环境自动切换 |
| 安装包体积 | ClawStudio 本体 < 30MB（不含 Node.js / OpenClaw） |
| 卸载干净度 | 卸载后不残留任何文件和服务 |
| 安全性 | API Key 通过 OS Keychain 或 OpenClaw ref 模式存储，不明文写日志 |

---

## 第三部分：PRD 1.0 vs 2.0 差异对比

### 3.1 定位变化

| 维度 | PRD 1.0 | PRD 2.0 |
|------|---------|---------|
| 产品定位 | AI Agent 可视化监控工具 | 一键 AI 助手平台（安装+监控一体） |
| 核心价值 | 看得见 Agent 在干什么 | **从零开始到用上 AI 助手只需 5 分钟** |
| 用户门槛 | 需会 CLI，已装好 OpenClaw | 零 CLI 经验，什么都不需要预装 |
| 产品比喻 | Agent 的监控面板 | OpenClaw 的 Docker Desktop |

### 3.2 页面级差异

| 页面/模块 | PRD 1.0 | PRD 2.0 | 变化类型 |
|-----------|---------|---------|----------|
| Setup Wizard | ❌ 不存在 | ✅ 7 步安装向导 | **全新** |
| Dashboard | 统计 + Agent 网格 | + OpenClaw 运行状态条 | 增强 |
| Agents | 看板 + 通道聚合 + 创建 | 不变 | 保持 |
| Overwatch | 思维流 + 视觉流 + HITL | 不变 | 保持 |
| Cost Monitor | ❌ 不存在（PRD 1.0 有但未实现） | ✅ 油表 + Token 分解 + 明细 | **全新** |
| Environment | Docker 沙盒管理 | + OpenClaw 管理面板 + 启停控制 | 重大增强 |
| Traces | 回放播放器 + 历史表 | 不变 | 保持 |
| Settings | 密钥 + 模型 + HITL + 网络 | + 通道管理 + OpenClaw 升级/卸载 | 增强 |

### 3.3 后端差异

| 模块 | PRD 1.0 | PRD 2.0 |
|------|---------|---------|
| `keychain.rs` | ✅ | ✅ 保持 |
| `openclaw.rs` | Agent 进程管理 | 保持 |
| `docker.rs` | 沙盒管理 | 保持 |
| `db.rs` | SQLite 持久化 | 保持 |
| `setup.rs` | ❌ | ✅ **全新** — 环境检测、安装、卸载 |
| `gateway.rs` | ❌ | ✅ **全新** — Gateway 生命周期管理 |

### 3.4 新增依赖

| 依赖 | 用途 | 位置 |
|------|------|------|
| `reqwest` | HTTP 健康检查 `/healthz` | Cargo.toml |
| `which` | 检测 node/npm/openclaw 路径 | Cargo.toml |
| `dirs` | 获取跨平台 home 目录 | Cargo.toml |
| `serde_json5` | 解析 OpenClaw 的 JSON5 配置 | Cargo.toml |

---

## 第四部分：商业模式与核心竞争力

### 4.1 市场定位

```
                高门槛（CLI/API）
                    │
                    │  OpenClaw CLI ←── 当前状态
                    │
    开发者工具 ─────┼───── 大众消费品
                    │
                    │  ClawStudio ←── 目标位置
                    │
                低门槛（GUI）
```

ClawStudio 要做的事：**把 OpenClaw 从左上象限推到右下象限**。

### 4.2 核心竞争力分析

#### 竞争力 1：唯一的 OpenClaw GUI 前端

- OpenClaw 目前有 34.1 万 GitHub Stars，社区活跃
- 官方只提供 CLI 和 Web Chat UI
- **没有任何桌面 GUI 管理工具**
- ClawStudio 可以成为事实标准的 OpenClaw 桌面管理器

#### 竞争力 2：零门槛安装体验

| 对比 | OpenClaw 原生 | ClawStudio |
|------|--------------|------------|
| 安装 Node.js | 用户自行处理 | 自动检测+一键安装 |
| 安装 OpenClaw | `npm install -g` | 一个按钮 |
| 配置 API Key | 编辑 JSON 文件 | 输入框 + 测试按钮 |
| 配置模型 | CLI flag 或 JSON | 卡片选择器 |
| 启动服务 | `openclaw gateway start` | 一个按钮 |
| 连接 Telegram | 多步 CLI 操作 | 引导式表单 |
| 监控运行状态 | 看日志 | 实时可视化面板 |
| 卸载 | 多步 CLI 操作 | 一个按钮 + 确认 |

**从 20+ 个命令行步骤 → 7 次鼠标点击**

#### 竞争力 3：Computer Use 可视化监控

这是 PRD 1.0 的核心优势，在 v2.0 中保留并强化：
- 实时准星 + 包围框 — 看见 AI 在屏幕上做什么
- HITL 审批系统 — 危险操作人工拦截
- 回放录像 — 复盘 Agent 每一步
- **市场上没有任何工具提供这种可视化能力**

#### 竞争力 4：成本控制系统

- 实时费用燃烧器（精确到 $0.001）
- 预算熔断机制（超限自动停止）
- 截图 Token 占比分析（Computer Use 场景的痛点）
- 对于企业用户：**可控的 AI 支出 = 可预测的 ROI**

### 4.3 商业模式选项

#### 模式 A：开源免费 + 增值服务（推荐）

| 层级 | 价格 | 内容 |
|------|------|------|
| Community | 免费开源 | 完整功能，本地运行 |
| Pro | $9.9/月 | 云端备份、多设备同步、优先更新 |
| Team | $29.9/月/人 | 团队共享 Agent、集中费用管理、审计日志 |

核心逻辑：**工具免费，服务收费**。类比 Docker Desktop 的免费个人 + 付费企业模式。

#### 模式 B：平台抽成

- ClawStudio 内置 API Key 代理（用户不需要自己注册 Anthropic/OpenAI 账号）
- 按 Token 用量加成 10-20% 服务费
- 极大降低使用门槛（连注册 API Key 都不用）
- 风险：依赖上游定价，利润空间薄

#### 模式 C：应用商店

- 构建 Agent 模板市场（类似 GPTs Store）
- 用户上传/下载预配置的 Agent 模板
- 模板作者获得收益分成
- 长期形成生态壁垒

**建议策略：以模式 A 为主，逐步叠加模式 C 的应用商店。**

### 4.4 竞品对比

| 产品 | 类型 | 安装门槛 | GUI 监控 | Computer Use | 成本控制 | 本地运行 |
|------|------|----------|----------|--------------|----------|----------|
| OpenClaw CLI | CLI 工具 | 高 | ❌ | ❌ 可视化 | ❌ | ✅ |
| Claude Desktop | 桌面 App | 低 | ❌ | ❌ 可视化 | ❌ | ✅ |
| ChatGPT Desktop | 桌面 App | 低 | ❌ | ❌ | ❌ | ❌ 云端 |
| Dify | Web 平台 | 中 | 部分 | ❌ | 部分 | ❌ 云端 |
| **ClawStudio** | **桌面 App** | **低** | **✅ 全面** | **✅ 准星+回放** | **✅ 油表+熔断** | **✅** |

ClawStudio 的独特组合：**低门槛 + 全面可视化 + 本地隐私 + 成本控制** — 没有直接竞品。

### 4.5 一句话总结

> **ClawStudio 的核心竞争力是：让 OpenClaw 从「极客玩具」变成「人人可用的 AI 助手管理器」，同时提供市场上唯一的 Computer Use 可视化监控能力。**

---

## 第五部分：开发路线图

### Phase 1 — MVP（当前 → 补全 PRD 1.0 遗留）
- 补全 D1-D7（Dashboard 图表、Cost Monitor、Sandboxes 增强）
- 确保 PRD 1.0 功能 100% 覆盖

### Phase 2 — 安装集成（核心新增）
- 实现 Setup Wizard（7 步向导）
- 实现 `setup.rs` + `gateway.rs` 后端
- Dashboard 集成 OpenClaw 状态条
- Settings 集成卸载功能

### Phase 3 — 通道与生态
- 通道管理面板（Telegram / Discord / WeChat 引导式连接）
- Agent 模板导入导出
- 一键升级 OpenClaw

### Phase 4 — 商业化
- 云同步 + 多设备
- Agent 模板市场
- 团队协作功能
