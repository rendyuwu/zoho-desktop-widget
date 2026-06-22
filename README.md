# Zoho Desktop Widget

Always-on-top desktop widget streaming Zoho ticket data via WebSocket. Built with Tauri v2 (Rust backend + React frontend). BIGSU UI components. LDAP authentication gate. Auto-update support.


## Overview

Compact 360px sidebar widget that stays on top of your desktop. Streams live ticket counts and waiting response tickets from Zoho. Fires native OS notifications when tickets cross the ASAP threshold (900s elapsed). System tray icon with ASAP count badge.

LDAP login required before widget or WS connection. Server on corp VPN — public download off-VPN cannot authenticate. Auto-update checks GitHub Releases on launch.

**Key features:**

- LDAP authentication gate (direct user bind, no service-account credential in binary)
- Remember-me via OS keychain (Windows Credential Manager / macOS Keychain / Linux Secret Service)
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
- Auto-update: checks GitHub Releases on launch, verifies Tauri signature before install, user can defer

## Install

### Download

Pre-built binaries available from [GitHub Releases](https://github.com/rendyuwu/zoho-desktop-widget/releases).

- **Linux**: `.AppImage` or `.deb`
- **Windows**: `.msi` or `.exe` (NSIS)
- **macOS**: `.dmg`

> LDAP server is on corp VPN. App cannot authenticate off-VPN.

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
git clone https://github.com/rendyuwu/zoho-desktop-widget.git
cd zoho-desktop-widget
npm install
```

Create `.env.local` in repo root with build secrets:

```bash
ZOHO_WS_URL="wss://your-domain.com/zoho/wss"
LDAP_SERVER_URI="ldaps://host:636"
LDAP_BIND_TEMPLATE="{user}@domain.com"
LDAP_ALLOW_INSECURE=false
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```

Then start dev server:

```bash
npm run tauri dev
```

> The Makefile auto-loads `.env.local` — `make build` exports vars for you. For `npm run tauri dev`, export vars manually or source `.env.local`.

Dev server runs at `http://localhost:1420`. Tauri window opens automatically.

### Environment variables

See [Environment variables](#environment-variables) below for details on each var.

## Build

### Via Makefile (recommended)

```bash
make build
```

Auto-loads `.env.local`, exports build secrets, runs `npm run tauri build`.

Output in `src-tauri/target/release/bundle/`.

### Via npm directly

```bash
npm run tauri build
```

Must export all required env vars manually (see [Environment variables](#environment-variables)).

### Build prerequisites

Same as [Dev setup](#dev-setup) prerequisites. All required env vars must be set: `ZOHO_WS_URL`, `LDAP_SERVER_URI`, `LDAP_BIND_TEMPLATE`.

## Environment variables

| Variable | Required | Description |
|---|---|---|
| `ZOHO_WS_URL` | Required | WebSocket endpoint URL. Baked into binary at compile time via Rust `env!`. No fallback — build fails if unset. |
| `LDAP_SERVER_URI` | Required | LDAP server URI, e.g. `ldaps://host:636` or `ldap://host:389`. Baked at compile time via `env!`. No fallback — build fails if unset. |
| `LDAP_BIND_TEMPLATE` | Required | Bind DN/UPN template with `{user}` placeholder, e.g. `{user}@domain.com`. Baked at compile time via `env!`. No fallback — build fails if unset. |
| `LDAP_ALLOW_INSECURE` | Optional | Set to `true` to allow plain `ldap://` cleartext bind. Defaults to disabled (`ldaps://` required). |
| `TAURI_SIGNING_PRIVATE_KEY` | Release build | Path to Tauri signing key for auto-update `.sig` files. Makefile defaults to `src-tauri/keys/update.key`. |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Release build | Password for signing key. Must be defined — use empty string for passwordless key. |

**Dev/local build:** Create `.env.local` (gitignored) with the required vars:

```bash
ZOHO_WS_URL="wss://your-domain.com/zoho/wss"
LDAP_SERVER_URI="ldaps://host:636"
LDAP_BIND_TEMPLATE="{user}@domain.com"
LDAP_ALLOW_INSECURE=false
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
```

### Release workflow (Makefile)

Tauri cannot cross-compile between OSes. Each platform must be built on its own machine:

1. On each machine (Linux / Windows / macOS), on the LAN:
   ```bash
   make release VERSION=v0.1.0
   ```
   Builds, signs, creates/updates a draft GitHub release, uploads platform bundle + `.sig`.

2. On any one machine, after all three have uploaded:
   ```bash
   make latest-json VERSION=v0.1.0   # assembles updater manifest
   make publish     VERSION=v0.1.0   # un-drafts → becomes "latest"
   ```

The updater endpoint points at `https://github.com/rendyuwu/zoho-desktop-widget/releases/latest/download/latest.json`.

## Architecture

```
src-tauri/src/
  lib.rs          — app entry, plugin setup, window events, session gate
  auth.rs         — LDAP direct-bind auth, keychain, ldap_login/auto_login/logout cmds
  ws.rs           — WebSocket client (connect, parse, auto-reconnect backoff)
  timer.rs        — 3s timer: classify tickets, emit ticket-move, fire notifications
  tray.rs         — system tray: toggle window, always-on-top, ASAP badge
  updater.rs      — auto-update: check_for_updates, install_update, update-available event
  window_state.rs — window position persistence via tauri-plugin-store

src/
  App.tsx                  — root component, auth router (login gate → widget)
  constants.ts             — threshold constants, classify/format helpers
  types.ts                 — TypeScript interfaces matching Rust structs
  components/
    Widget.tsx             — widget container, composes all sub-components
    WidgetHeader.tsx       — compact header with logout
    CountGrid.tsx          — MetricCards for ticket counts
    TicketCard.tsx         — id, dept Badge, subject, elapsed, urgency Badge
    AsapList.tsx           — danger Badge tickets, scrollable
    WaitingList.tsx        — warning + info Badge tickets, scrollable
    LoginScreen.tsx        — BIGSU login form (Card/FormField/Input/Checkbox/Button)
    UpdateBanner.tsx       — update toast + inline banner (Update & Restart / Later)
    LoadingState.tsx       — initial load skeleton
    EmptyTicketState.tsx   — "No tickets waiting"
    ErrorTicketState.tsx   — error state
  hooks/
    useAuth.ts             — auth state (ldap_login, auto_login, logout)
    useTicketEvents.ts     — listen to ticket-data + ticket-move Tauri events
```

### Data flow

1. App launches → attempts `auto_login` from OS keychain
2. If unauthenticated → login screen. User enters credentials → `ldap_login` binds to LDAP
3. Auth success → `start_session` spawns WS client + 3s timer (idempotent, spawn-once)
4. Rust WS client connects to endpoint from `ZOHO_WS_URL` (baked at compile time)
5. Server pushes JSON: `{ data: { total_ticket, onhold_ticket, waiting_response } }`
6. Rust parses, caches, emits `ticket-data` event to frontend
7. Rust 3s timer evaluates elapsed time per waiting ticket, emits `ticket-move` on category change
8. Ticket crossing to ASAP (>= 900s) triggers native notification
9. Frontend renders BIGSU components based on received data
10. On launch, Rust checks GitHub Releases for update → emits `update-available` if found

### Reconnect backoff

WS auto-reconnects on disconnect. Backoff sequence: 1s, 2s, 5s, 10s, 30s (cap).

## Troubleshooting

### Login fails / cannot authenticate

- Check VPN connectivity — LDAP server is on corp VPN
- Check stderr logs for `auth:` messages
- Wrong credentials → generic "Invalid username or password" (no user enumeration)
- Server unreachable → "Cannot reach the authentication server. Check your VPN connection."
- Saved password expired → app forgets creds, prefills username, prompts for re-login

### Remember-me not persisting

- **Linux**: ensure Secret Service / GNOME Keyring / KWallet is running and unlocked
- **Windows**: check Windows Credential Manager
- **macOS**: check Keychain Access

### Widget not connecting / no data

- Check network connectivity to your WS endpoint (configured via `ZOHO_WS_URL`)
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

### Auto-update not working

- Update check runs on launch only (no periodic re-check)
- Check stderr for `update check:` messages
- Update check timeout: 30s. Network issues → silent skip, app continues normally
- Update install failure → logged, app continues. No crash.
- Only signed updates accepted — unsigned `.sig` rejected.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

Private project.

## Tech stack

- [Tauri v2](https://v2.tauri.app/) — desktop framework
- [React 19](https://react.dev/) + [Vite](https://vitejs.dev/) + [TypeScript](https://www.typescriptlang.org/)
- [BIGSU UI](https://www.npmjs.com/package/@gio/bigsu-ui) — component library
- [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) — WebSocket client
- [ldap3](https://crates.io/crates/ldap3) — LDAP direct-bind auth
- [keyring](https://crates.io/crates/keyring) — OS keychain for remember-me
- [tauri-plugin-notification](https://v2.tauri.app/plugin/notification/) — native notifications
- [tauri-plugin-store](https://v2.tauri.app/plugin/store/) — local persistence
- [tauri-plugin-updater](https://v2.tauri.app/plugin/updater/) — auto-update
