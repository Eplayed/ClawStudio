# ClawStudio Nova

<div align="center">
  <img src="src-tauri/icons/icon.svg" alt="ClawStudio Logo" width="128" height="128">
  
  <h3>🦞 OpenClaw 可视化管理工作站</h3>
  
  <p>一键安装、可视化监控、零门槛使用 OpenClaw 的桌面应用。<br>
  <strong>The "Docker Desktop" for OpenClaw.</strong></p>

  <p>
    <img src="https://img.shields.io/badge/version-0.1.0-blue.svg" alt="Version">
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg" alt="Platform">
    <img src="https://img.shields.io/badge/license-AGPL--3.0-green.svg" alt="License">
    <img src="https://img.shields.io/badge/Tauri-v2-blueviolet.svg" alt="Tauri">
  </p>
</div>

---

## 📦 下载安装

### Windows

| 版本 | 下载 | 说明 |
|------|------|------|
| v0.1.0 | [ClawStudio_0.1.0_x64_portable.zip](https://github.com/Eplayed/ClawStudio/releases/download/v0.1.0/ClawStudio_0.1.0_x64_portable.zip) | 便携版，解压即运行 |

### macOS / Linux

暂无预编译版本，请从源码编译。

---

## ✨ 核心功能

### 🚀 Setup Wizard（安装向导）
- **一键安装 Node.js** - 自动检测并安装
- **一键安装 OpenClaw** - 支持 npm 镜像加速
- **API Key 配置** - 安全存储于系统 Keychain
- **模型选择** - Claude / GPT-4o / DeepSeek
- **通道配置** - Telegram / Discord / WeChat

### 📊 Dashboard（仪表盘）
- OpenClaw 运行状态实时监控
- Gateway 启动/停止/重启
- 活跃 Agent 统计
- 今日费用 + 7 天趋势

### 🤖 Agents（特工列队）
- 看板视图管理多个 Agent
- 通道聚合器（多平台消息）
- 创建 Agent 向导

### 📺 Overwatch（监控舱）
- 思维流实时显示
- 视觉流（VNC 桌面）
- HITL 审批系统

### 💰 Cost Monitor（烧钱计算器）
- 预算油表
- Token 分解
- 自动熔断

### ⏱ Audit & Traces（合规审计）
- 操作回放
- 7 天本地存储
- CSV/JSON 导出

### 🧙 Template Market（模板市场）
- Agent 技能模板导入/导出
- `.claw-template` 格式
- 深链接分享 `claw://template/xxx`

---

## 🚀 快速开始

### 方式一：下载安装包（推荐）

1. 访问 [Releases](https://github.com/Eplayed/ClawStudio/releases) 页面
2. 下载对应平台的安装包
3. 双击安装，首次启动进入 Setup Wizard

### 方式二：从源码编译

#### 前置要求

| 工具 | 版本 | 说明 |
|------|------|------|
| Node.js | ≥ 22.0 | JavaScript 运行时 |
| pnpm | ≥ 9.0 | 包管理器 |
| Rust | ≥ 1.70 | 后端编译 |
| VS Build Tools | 2022 | Windows 编译必需 |

#### 编译步骤

```bash
# 1. 克隆仓库
git clone https://github.com/Eplayed/ClawStudio.git
cd ClawStudio

# 2. 安装依赖
pnpm install

# 3. 开发模式运行
pnpm tauri dev

# 4. 构建生产版本
pnpm tauri build
```

#### Windows 额外步骤

```powershell
# 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

---

## 🧪 运行测试

```bash
# Rust 后端测试 (28 tests)
cd src-tauri
cargo test --lib

# Vue 前端测试 (18 tests)
pnpm test
```

---

## 🏗 项目结构

```
ClawStudio/
├── src/                    # Vue 3 前端
│   ├── views/              # 页面组件
│   │   ├── SetupWizard.vue # 安装向导
│   │   ├── Dashboard.vue   # 仪表盘
│   │   ├── Agents.vue      # 特工列队
│   │   ├── Overwatch.vue   # 监控舱
│   │   └── Settings.vue    # 系统设置
│   ├── components/         # UI 组件
│   ├── stores/             # Pinia 状态
│   └── __tests__/          # 前端测试
│
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── main.rs         # 入口
│       ├── setup.rs        # 环境检测+安装
│       ├── gateway.rs      # Gateway 管理
│       ├── audit.rs        # 审计日志
│       ├── template.rs     # 模板系统
│       └── tests/          # 后端测试
│
├── package.json            # 前端依赖
├── Cargo.toml              # Rust 依赖
└── LICENSE                 # AGPL-3.0
```

---

## 🔐 安全性

- **API Key** 通过操作系统原生 Keychain 加密存储
- **审计日志** 本地 7 天滚动存储
- **HITL 拦截** 危险操作需人工确认

---

## 📜 开源协议

ClawStudio 采用 **AGPL-3.0** 协议：

- ✅ 个人使用、学习、修改
- ✅ 开源项目集成（需保持 AGPL-3.0）
- ❌ 闭源商业使用（需购买商业授权）

详见 [LICENSE](LICENSE)。

---

## 🤝 贡献指南

欢迎贡献代码、报告 Bug、提出建议！

1. Fork 本仓库
2. 创建分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面框架
- [Vue 3](https://vuejs.org/) - 前端框架
- [OpenClaw](https://github.com/openclaw/openclaw) - AI Agent 框架
- [Anthropic Claude](https://www.anthropic.com/) - AI 模型

---

<div align="center">
  <p>Made with ❤️ by ClawStudio Team</p>
  <p>
    <a href="https://github.com/Eplayed/ClawStudio/issues">报告 Bug</a> ·
    <a href="https://github.com/Eplayed/ClawStudio/issues">功能建议</a>
  </p>
</div>
