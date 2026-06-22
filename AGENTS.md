# AGENTS.md

## Communication

- Caveman mode ALWAYS ON. All prose, comments, commits. Drop articles, filler, hedging. Fragments OK. Code/identifiers verbatim.
- `stop caveman` or `normal mode` → revert temporarily.

## Project

Tauri v2 always-on-top desktop widget. Streams Zoho ticket data via WebSocket. BIGSU UI components. Rust backend owns WS + timer logic.

- SPEC.md = source of truth. Read before any work. §T tasks drive build order.
- FORMAT.md = spec encoding rules. Caveman symbols (`!` must, `⊥` never, `∀` every, etc.).
- No code yet. Scaffold from §T.T1.

## Reference impl

`/home/ubuntu/simondayce/zoho-frontend/resources/` — existing jQuery app. Reference for:
- WS endpoint + JSON data shape: `asset/js/websocket.js`
- 3s timer + threshold logic (600s/900s): `asset/js/interval.js`, `asset/js/websocket.js`
- Count color thresholds (>9 danger, 6-9 warning): `websocket.js:ticket()`

Ref ! binding. ∃ better Rust-native approach → use it. Match behavior, not implementation.

## Stack (planned)

- Tauri v2 (Rust backend + React frontend)
- Vite + React + TypeScript
- BIGSU: `@gio/bigsu-ui`, `@gio/bigsu-icons`
- No AppShell/Sidebar/TopCommandBar. Widget = custom compact header only.

## Env

- `ZOHO_WS_URL` — WS endpoint, baked at compile time via `env!`. GitHub secret in CI. No fallback — build fails if unset.

## Key invariants (from §V)

- Rust owns WS. Frontend never connects directly.
- WS auto-reconnect. Backoff: 1s → 2s → 5s → 10s → 30s cap.
- ∀ ticket → ASAP (900s) → native notification.
- Timer re-evaluate every 3s.
- Color ≠ only indicator. Badge tone + text label always paired.
