# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- WS endpoint now sourced from `ZOHO_WS_URL` env var (compile-time via `env!`). No fallback — build fails if unset. Set as GitHub secret in release CI to avoid exposing domain in source.

## [0.1.0] - 2026-06-22

### Added

- Tauri v2 always-on-top desktop widget (360x640, frameless, skip taskbar)
- Rust WebSocket client streaming Zoho ticket data (endpoint via `ZOHO_WS_URL` env var, baked at compile time)
- Auto-reconnect with backoff: 1s, 2s, 5s, 10s, 30s cap
- 3s timer evaluating waiting ticket elapsed time against thresholds (600s warning, 900s ASAP)
- Native OS notification when ticket crosses to ASAP (>= 900s)
- `ticket-move` Tauri event emission on category change
- System tray: toggle window visibility, always-on-top toggle, ASAP count badge
- Window position persistence via tauri-plugin-store
- BIGSU UI components: WidgetHeader, CountGrid (MetricCards), TicketCard, AsapList, WaitingList
- Urgency badges: danger (ASAP, >= 900s), warning (600-899s), info (< 600s)
- Empty state, loading skeleton, error state
- `get_current_tickets` and `reconnect_ws` Tauri commands
- Cross-platform: Linux, Windows, macOS

[Unreleased]: https://github.com/rendyuwu/zoho-desktop-widget/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/rendyuwu/zoho-desktop-widget/releases/tag/v0.1.0
