# SPEC

## §G GOAL

Tauri v2 always-on-top desktop widget. Stream Zoho ticket counts + ASAP/Waiting Response tickets. Native notifications on threshold cross. BIGSU UI. Non-distracting sidebar widget.

## §C CONSTRAINTS

- Tauri v2. Rust backend owns WS connection + 3s timer logic.
- React + Vite frontend. BIGSU components (@gio/bigsu-ui).
- Frameless window. `alwaysOnTop: true`. `skipTaskbar: true`. `decorations: false`.
- Window ~360px wide, ~640px tall. Height resizable.
- Cross-platform: Linux, Windows, macOS.
- WS endpoint: `wss://your-domain.com/zoho/wss`. No auth required.
- Rust backend ! maintain WS connection even when window hidden/minimized to tray.
- No AppShell/Sidebar/TopCommandBar. Widget too small for full shell.
- No click-to-open ticket URLs (deferred).
- No auth/login UI.
- Ref impl: `/home/ubuntu/simondayce/zoho-frontend/resources/` (jQuery app. WS protocol, data shape, timer logic, threshold values reference).
- Ref ! binding. ∃ better Rust-native approach (e.g. timer logic, data caching) → use it. Match behavior, not implementation.
- Auto-update via `tauri-plugin-updater`. Tauri signing key only. OS code signing deferred.
- Release: GitHub Releases via `tauri-action`. Draft → manual publish. Stable channel only.
- Update check: launch only. No periodic re-check.

## §I INTERFACES

- ws: `wss://your-domain.com/zoho/wss` → JSON `{ data: { total_ticket: [{status, total}], onhold_ticket: [{tag, total}], waiting_response: [{id_ticket, department, status_ticket, customer_response_time, subject, timestamp}] } }`
- tauri-event: `ticket-data` → frontend. Payload: parsed counts + waiting list.
- tauri-event: `ticket-move` → frontend. Payload: `{ id_ticket, from: "new"\|"warning"\|"asap", to: "new"\|"warning"\|"asap" }`.
- tauri-cmd: `get_current_tickets()` → returns last cached ticket data.
- tauri-cmd: `reconnect_ws()` → force WS reconnect.
- tray: click → toggle window visibility. Icon shows ASAP count badge.
- notify: native OS notification when ticket crosses to ASAP (≥900s).
- file: `~/.config/zoho-widget/store.json` (window position, notification prefs).
- updater: `tauri-plugin-updater` → check GitHub releases `latest.json`. Endpoint: `https://github.com/simondayce/zoho-desktop-widget/releases/latest/download/latest.json`.
- tauri-cmd: `check_for_updates()` → check updater endpoint. Returns `{ available: bool, version?: string, body?: string }`.
- tauri-cmd: `install_update()` → download + verify sig + install. Returns `{ success: bool, error?: string }`.
- tauri-event: `update-available` → frontend. Payload: `{ version, body }`.
- ci: `.github/workflows/release.yml` → tag `v*` triggers cross-platform build. Draft release. Uploads `.sig` + `latest.json` manifest.
- env: `TAURI_SIGNING_PRIVATE_KEY` required for release CI. Generates `.sig` files for updater verification.
- env: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` required if key encrypted.
- docs: `README.md` → overview, screenshots, install, dev setup, build, env vars, troubleshooting.
- docs: `CHANGELOG.md` → version history. One entry per release.

## §V INVARIANTS

V1: Rust backend ! own WS connection. Frontend never connects directly.
V2: WS connection ! auto-reconnect on disconnect. Backoff: 1s → 2s → 5s → 10s → 30s cap.
V3: ∀ ticket move to ASAP → native notification fired.
V4: Timer re-evaluate every 3s. ∀ waiting ticket → check elapsed time vs thresholds (600s, 900s).
V5: Window `alwaysOnTop` ! `true` at all times. User ! can toggle via tray menu.
V6: WS endpoint hardcoded `wss://your-domain.com/zoho/wss`. No auth/token needed.
V7: Frontend ! render BIGSU components only. ⊥ raw HTML/jQuery.
V8: ∀ MetricCard ! show label + value + period.
V9: Ticket card ! show: id_ticket, department (Badge), subject, elapsed time, urgency Badge (danger/warning/info).
V10: Color ≠ only indicator. Badge tone + text label always paired.
V11: ⊥ AppShell/Sidebar/TopCommandBar in widget. Custom compact header only.
V12: App ! check for updates on launch. ∃ update → fire `update-available` event.
V13: Update download ! verify Tauri signature before install. ⊥ unsigned updates.
V14: ⊥ silent update. User ! see toast + inline banner. Can defer ("Later").
V15: Update check/install failure ! not crash app. Log error, continue normal operation.
V16: Release CI ! produce draft release. Auto-update manifest (`latest.json`) ! published when draft → published.
V17: ∀ release tag ! match `v*` semver pattern. ⊥ non-semver tags trigger CI.

## §T TASKS

id|status|task|cites
T1|x|scaffold Tauri v2 project + Vite + React + TS|-
T2|x|install BIGSU packages (@gio/bigsu-ui, @gio/bigsu-icons)|V7
T3|x|config tauri.conf.json: frameless, alwaysOnTop, skipTaskbar, 360x640|V5,V11
T4|x|impl Rust WS client (connect, parse JSON, auto-reconnect backoff)|V1,V2,I.ws
T5|x|impl Rust 3s timer: re-evaluate elapsed time, emit ticket-move events|V4,I.tauri-event
T6|x|impl Rust notification: fire on ticket → ASAP threshold cross|V3,I.notify
T7|x|impl system tray: toggle window, ASAP count badge|V5,I.tray
T8|x|impl tauri commands: get_current_tickets, reconnect_ws|I.tauri-cmd
T9|x|WS endpoint hardcoded. No token needed.|V6
T10|x|build WidgetHeader component (compact, custom, no AppShell)|V11
T11|x|build CountGrid: MetricCards for GIO Open/OnProgress/OnHold + OnHold Abuse/Incident/Sales|V8,I.ws
T12|x|build TicketCard: id, dept Badge, subject, elapsed, urgency Badge|V9,V10
T13|x|build AsapList: danger Badge tickets, scrollable|V9,V10
T14|x|build WaitingList: warning Badge (10-15min) + info Badge (<10min), scrollable|V9,V10
T15|x|build useTicketEvents hook: listen ticket-data + ticket-move events|I.tauri-event
T16|x|impl EmptyState: "No tickets waiting" when list empty|-
T17|x|impl LoadingSkeleton: initial load state|-
T18|x|impl window position persistence via tauri-plugin-store|I.file
T19|x|test cross-platform: Linux, Windows, macOS window flags|V5
T20|x|test WS auto-reconnect: kill server → verify backoff reconnect|V2
T21|x|test timer threshold: simulate 600s/900s elapsed → verify ticket-move|V4
T22|x|test notification: ticket crosses 900s → verify native notify fired|V3
T23|x|write README.md: overview, screenshots, install, dev setup, build, env vars, troubleshooting|I.docs
T24|x|write CHANGELOG.md: initial v0.1.0 entry|I.docs
T25|x|add tauri-plugin-updater to Cargo.toml + configure updater endpoint in tauri.conf.json|V12,V13,I.updater
T26|x|impl Rust update check: check_for_updates() cmd, fire update-available event on launch|V12,V15,I.tauri-cmd,I.tauri-event
T27|x|impl Rust install_update() cmd: download, verify sig, install, relaunch|V13,V15,I.tauri-cmd
T28|x|generate Tauri signing key pair. Store private key as GitHub secret `TAURI_SIGNING_PRIVATE_KEY`|V13,I.env
T29|x|create .github/workflows/release.yml: tag v* → cross-platform build, draft release, upload .sig + latest.json|V16,V17,I.ci
T30|x|impl UpdateBanner component: toast + inline banner. "Update available — v{version}". Buttons: "Update & Restart" / "Later"|V14
T31|x|impl update error handling: check/install failure → log, continue. ⊥ crash|V15
T32|.|test update flow: publish draft release → verify app detects update → install → relaunch|V12,V13,V16

## §B BUGS

id|date|cause|fix
