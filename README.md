# ClawStudio Nova

<div align="center">
  <img src="src-tauri/icons/icon.svg" alt="ClawStudio Logo" width="128" height="128">
  
  <h3>AI Agent Visual Control Center</h3>
  
  <p>A professional desktop application for monitoring and controlling AI agents with real-time visualization, VNC streaming, and cost tracking.</p>

  <p>
    <img src="https://img.shields.io/badge/version-0.1.0-blue.svg" alt="Version">
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg" alt="Platform">
    <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
    <img src="https://img.shields.io/badge/Tauri-v2-blueviolet.svg" alt="Tauri">
  </p>
</div>

---

## ✨ Features

### 🤖 Agents Dashboard
- **Kanban Board**: 4-column task board (Queued / Running / Paused / Completed)
- **Channel Aggregator**: Multi-platform integration (WeChat / Telegram / Discord)
- **Agent Creation Wizard**: Full-featured modal with Computer Use and sandbox binding

### 📺 Overwatch Console
- **Thought Log**: Real-time AI reasoning stream with syntax highlighting
- **Visual Stream**: Simulated desktop with AI cursor tracking
- **HITL Bar**: Human-in-the-Loop approval system with timeout countdown
- **VNC Streaming**: Live desktop view for Computer Use agents

### 📦 Sandboxes Manager
- **Docker Integration**: Create, start, stop, destroy containers
- **Resource Monitoring**: CPU/Memory usage bars
- **VNC Connection**: One-click desktop access
- **Installation Guide**: Platform-specific Docker setup

### ⏱ Traces & Replay
- **Session Player**: Step-by-step replay with speed control
- **History Table**: Searchable task history with filters
- **Export**: Download trace logs for analysis

### 💰 Cost Tracking
- **Real-time Calculator**: Accurate API cost estimation
- **Budget Alerts**: Automatic pause when limit exceeded
- **Multi-model Support**: Claude, GPT-4o pricing tables

---

## 🖥 Screenshots

| Dashboard | Overwatch |
|:---------:|:---------:|
| <img src="docs/screenshots/dashboard.png" width="400"> | <img src="docs/screenshots/overwatch.png" width="400"> |

| Agents | Sandboxes |
|:------:|:---------:|
| <img src="docs/screenshots/agents.png" width="400"> | <img src="docs/screenshots/sandboxes.png" width="400"> |

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** 18+ 
- **pnpm** 9+
- **Rust** 1.70+ (for Tauri backend)
- **Docker** (optional, for sandbox features)

### Installation

```bash
# Clone the repository
git clone https://github.com/clawstudio/nova.git
cd nova

# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Development

```bash
# Frontend only (fast iteration)
pnpm dev

# Full Tauri development
pnpm tauri dev

# Type checking
pnpm typecheck

# Lint
pnpm lint
```

---

## 🏗 Architecture

```
clawstudio/
├── src/                    # Vue 3 frontend
│   ├── components/         # Reusable UI components
│   │   ├── KanbanBoard.vue
│   │   ├── ThoughtLog.vue
│   │   ├── VisualStream.vue
│   │   ├── HITLBar.vue
│   │   └── ...
│   ├── views/              # Page components
│   │   ├── Dashboard.vue
│   │   ├── Agents.vue
│   │   ├── Overwatch.vue
│   │   ├── Sandboxes.vue
│   │   └── Traces.vue
│   ├── stores/             # Pinia state management
│   │   ├── agents.ts
│   │   └── settings.ts
│   ├── utils/              # Helper functions
│   │   ├── eventParser.ts
│   │   └── costCalculator.ts
│   └── styles/
│       └── variables.css   # Design tokens
│
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri entry point
│   │   ├── keychain.rs     # Secure key storage
│   │   ├── openclaw.rs     # API client
│   │   ├── docker.rs       # Container management
│   │   └── db.rs           # SQLite operations
│   └── tauri.conf.json     # Tauri configuration
│
└── .github/
    └── workflows/
        └── release.yml     # CI/CD pipeline
```

---

## 🔧 Configuration

### API Keys

ClawStudio stores API keys securely in your OS keychain:
- **macOS**: Keychain Access
- **Windows**: Credential Manager
- **Linux**: Secret Service (GNOME Keyring / KWallet)

### Settings

Access via **Settings** page:
- Default model selection
- Computer Use model
- Temperature
- Permission levels
- Budget limits

---

## 🎨 Design System

ClawStudio uses a **Mission Control Dark** theme:

| Token | Value | Usage |
|-------|-------|-------|
| `--bg-deep` | `#060a14` | Deepest background |
| `--bg-base` | `#0a0e1a` | Primary background |
| `--bg-card` | `#0f1629` | Card surfaces |
| `--cyan` | `#06d6d6` | Primary accent |
| `--amber` | `#f0a030` | Warning / active |
| `--green` | `#22c55e` | Success |
| `--red` | `#ef4444` | Error / danger |

Font stack:
- **UI**: Outfit, Noto Sans SC
- **Monospace**: JetBrains Mono

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [Vue 3](https://vuejs.org/) - Progressive JavaScript framework
- [noVNC](https://novnc.com/) - HTML5 VNC client
- [Anthropic Claude](https://www.anthropic.com/) - AI model provider

---

<div align="center">
  <p>Made with ❤️ by the ClawStudio Team</p>
  <p>
    <a href="https://github.com/clawstudio/nova/issues">Report Bug</a> ·
    <a href="https://github.com/clawstudio/nova/issues">Request Feature</a>
  </p>
</div>
