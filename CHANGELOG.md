# Changelog

All notable changes to ClawStudio Nova will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-03-29

### Added
- **Agents Page**: Kanban board with 4 columns (Queued/Running/Paused/Completed), channel aggregator, create agent modal
- **Overwatch Page**: Real-time monitoring with thought log, visual stream, and HITL (Human-in-the-Loop) approval bar
- **Sandboxes Page**: Docker container management with VNC connection support
- **Traces Page**: Session replay player with history table
- **noVNC Integration**: Real VNC streaming for Computer Use agents
- **Event Parser**: OpenClaw event stream parsing and UI binding
- **Cost Calculator**: Accurate API cost tracking with budget alerts
- **Pinia Stores**: Reactive state management for agents and settings
- **Rust Backend**: Keychain, OpenClaw API, Docker, and SQLite modules

### Technical
- Vue 3 + TypeScript + Vite frontend
- Tauri v2 desktop framework
- Pinia state management
- CSS custom properties for theming
- GitHub Actions CI/CD for releases

[0.1.0]: https://github.com/clawstudio/nova/releases/tag/v0.1.0
