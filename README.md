# Zoho Desktop Widget

Always-on-top desktop widget streaming Zoho ticket data via WebSocket. Built with Tauri v2 (Rust backend + React frontend). BIGSU UI components.

<!-- TODO: Add screenshot after all features implemented (T25-T32) -->
<!-- ![Screenshot](docs/screenshot.png) -->

## Overview

Compact 360px sidebar widget that stays on top of your desktop. Streams live ticket counts and waiting response tickets from Zoho. Fires native OS notifications when tickets cross the ASAP threshold (900s elapsed). System tray icon with ASAP count badge.

**Key features:**

- Real-time WebSocket streaming (Rust-owned connection, auto-reconnect with backoff)
- Ticket count grid: Open / On Progress / On Hold + On Hold breakdown (Abuse / Incident / Sales)
- Waiting response list with urgency badges:
  - **ASAP** (danger) — elapsed >= 900s
  - **Warning** (warning) — elapsed 600-899s
  - **New** (info) — elapsed < 600s
- Native OS notifications when ticket crosses to ASAP
- System tray: click to toggle window visibility, right-click for menu (Show/Hide, Always on Top, Quit)
- Tray tooltip shows ASAP count
- Window position persistence (restored on next launch)
- Always-on-top toggle via tray menu
- Frameless, skip taskbar

## Install

### Download

> **Note:** Auto-update infrastructure (signing key, updater plugin) is configured. Update check logic (T26), install command (T27), release CI (T29), and UpdateBanner UI (T30) are pending. Build from source for now.

Pre-built binaries will be available from [GitHub Releases](https://github.com/simondayce/zoho-desktop-widget/releases) once release CI is set up.

- **Linux**: `.AppImage` or `.deb`
- **Windows**: `.msi` or `.exe` (NSIS)
- **macOS**: `.dmg`

### Build from source

See [Build](#build) below.

## Dev setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+ and npm
- Tauri v2 system dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
  - **Windows**: Microsoft Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools

### Steps

```bash
git clone https://github.com/simondayce/zoho-desktop-widget.git
cd zoho-desktop-widget
npm install
npm run tauri dev
```

Dev server runs at `http://localhost:1420`. Tauri window opens automatically.

## Build

```bash
npm run tauri build
```

Output in `src-tauri/target/release/bundle/`.

### Build prerequisites

Same as [Dev setup](#dev-setup) prerequisites. No additional env vars needed for local build.

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Release CI only | Tauri signing key for auto-update `.sig` files. Copy contents of `src-tauri/keys/update.key` into this GitHub secret. |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Not required | Key generated without password. Omit this env var in CI. |

**No env vars required for dev or local build.** WebSocket endpoint is hardcoded (`wss://your-domain.com/zoho/wss`), no auth token needed.

### GitHub secret setup (release CI)

1. Copy the contents of `src-tauri/keys/update.key` (private key, NOT the `.pub` file)
2. Go to repo Settings → Secrets and variables → Actions → New repository secret
3. Name: `TAURI_SIGNING_PRIVATE_KEY`, value: paste private key content
4. No password secret needed — key was generated without encryption

## Architecture

```
src-tauri/src/
  lib.rs          — app entry, plugin setup, window events
  ws.rs           — WebSocket client (connect, parse, auto-reconnect backoff)
  timer.rs        — 3s timer: classify tickets, emit ticket-move, fire notifications
  tray.rs         — system tray: toggle window, always-on-top, ASAP badge
  window_state.rs — window position persistence via tauri-plugin-store

src/
  App.tsx                  — root component, layout
  hooks/useTicketEvents.ts — listen to ticket-data + ticket-move Tauri events
  components/              — WidgetHeader, CountGrid, TicketCard, AsapList, WaitingList, etc.
  constants.ts             — threshold constants, classify/format helpers
  types.ts                 — TypeScript interfaces matching Rust structs
```

### Data flow

1. Rust WS client connects to `wss://your-domain.com/zoho/wss`
2. Server pushes JSON: `{ data: { total_ticket, onhold_ticket, waiting_response } }`
3. Rust parses, caches, emits `ticket-data` event to frontend
4. Rust 3s timer evaluates elapsed time per waiting ticket, emits `ticket-move` on category change
5. Ticket crossing to ASAP (>= 900s) triggers native notification
6. Frontend renders BIGSU components based on received data

### Reconnect backoff

WS auto-reconnects on disconnect. Backoff sequence: 1s, 2s, 5s, 10s, 30s (cap).

## Troubleshooting

### Widget not connecting / no data

- Check network connectivity to `wss://your-domain.com`
- Check stderr logs for WS errors (`WS connect failed`, `WS error`)
- Widget auto-reconnects with backoff — wait 30s max
- Use tray menu → Quit, then relaunch

### Notifications not appearing

- **Linux**: Install `libnotify` and ensure a notification daemon is running
- **Windows**: Check Focus Assist / notification settings
- **macOS**: System Settings → Notifications → ensure widget app is allowed

### Tray icon not showing

- **Linux**: Ensure `libayatana-appindicator3` is installed. Some desktop environments require `XAppStatusIcon` support
- **Windows**: Check system tray overflow area
- **macOS**: Check menu bar

### Window position not restored

- Position stored in platform config directory:
  - **Linux**: `~/.config/zoho-widget/store.json`
  - **macOS**: `~/Library/Application Support/com.simondayce.zoho-widget/store.json`
  - **Windows**: `%APPDATA%/com.simondayce.zoho-widget/store.json`
- Delete `store.json` to reset position to default

### Always-on-top not working

- Toggle via tray menu: right-click → "Always on Top"
- Some Linux window managers (tiling WMs) may not respect `alwaysOnTop`

### Widget too tall / short

- Height is resizable. Drag window edges to adjust.
- Width is fixed at 360px per spec.

## License

Private project.

## Tech stack

- [Tauri v2](https://v2.tauri.app/) — desktop framework
- [React 19](https://react.dev/) + [Vite](https://vitejs.dev/) + [TypeScript](https://www.typescriptlang.org/)
- [BIGSU UI](https://www.npmjs.com/package/@gio/bigsu-ui) — component library
- [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) — WebSocket client
- [tauri-plugin-notification](https://v2.tauri.app/plugin/notification/) — native notifications
- [tauri-plugin-store](https://v2.tauri.app/plugin/store/) — local persistence
