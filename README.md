# SILO

**SILO** is a privacy-first desktop productivity and digital wellness platform built with **Tauri, Svelte, Rust, and SQLite**.

SILO helps users monitor screen time, control distractions, analyze internet usage, and understand where their bandwidth is being consumed through real-time analytics and intelligent productivity tools.

---

# Features

## Screen Time Tracking
- Real-time application usage monitoring
- Active window tracking
- Daily, weekly, and monthly statistics
- Productivity insights and trends

## Focus Mode
- App and website usage limits
- Scheduled restrictions
- Soft warning enforcement
- Hard blocking enforcement
- Focus session management

## Live Network Monitoring
- Real-time upload speed display
- Real-time download speed display
- Network activity dashboard
- Low-resource background monitoring

## Data Usage Analytics
- Per-application bandwidth tracking
- Per-website bandwidth tracking
- Daily usage records
- Overall upload/download summaries
- Historical analytics retention

## Data Monetization Awareness
- Identify data-heavy applications
- Discover bandwidth-intensive websites
- Optimize mobile hotspot and capped plans
- Improve internet cost management

## Backup & Retention
- 90-day analytics retention
- End-of-month backup reminders
- Exportable reports
- Automatic cleanup of expired records

---

# Technology Stack

## Frontend
- Svelte
- Vite
- Tauri Frontend APIs

## Backend
- Rust
- Tokio
- Tauri

## Database
- SQLite

## Windows Integration
- Win32 APIs
- ETW (Event Tracing for Windows)
- Windows Network APIs

---

# Architecture

User Interface (Svelte + Tauri)
↓
IPC Communication
↓
Rust Backend
├── Monitoring Engine
├── Network Analytics Engine
├── Rules Engine
├── Focus Engine
├── Storage Layer
├── Tray Services
└── Optional Elevated Agent

---

# Dashboard

The dashboard provides:

- Active application monitoring
- Live upload speed
- Live download speed
- Focus mode status
- Daily screen time summaries
- Per-app data usage analytics
- Per-website data usage analytics
- Historical productivity insights

---

# Privacy

- Local-first architecture
- User-owned analytics
- No mandatory cloud services
- Optional backups
- No selling of personal usage data

---

# Development

## Requirements

- Rust (Stable)
- Node.js 18+
- pnpm
- Tauri prerequisites
- WebView2 Runtime

## Development

```bash
pnpm install
pnpm tauri dev
```

## Production Build

```bash
pnpm tauri build
```

---

# Roadmap

- AI-powered productivity insights
- Encrypted cloud backups
- Cross-device synchronization
- Team productivity analytics
- Mobile companion application
- ISP quota tracking
- Advanced bandwidth forecasting

---

# License

Private project.
