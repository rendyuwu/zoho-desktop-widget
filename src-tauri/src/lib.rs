use std::sync::Mutex;

use tauri::{Manager, WindowEvent};

mod timer;
mod tray;
mod window_state;
mod ws;

use ws::TicketPayload;

pub struct TicketCache(pub Mutex<Option<TicketPayload>>);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn load_tauri_conf() -> Value {
        let raw = include_str!("../tauri.conf.json");
        serde_json::from_str(raw).expect("tauri.conf.json must parse")
    }

    fn main_window() -> Value {
        let conf = load_tauri_conf();
        conf["app"]["windows"]
            .as_array()
            .expect("windows array")
            .iter()
            .find(|w| w["label"] == "main")
            .expect("main window config")
            .clone()
    }

    #[test]
    fn test_window_always_on_top_true() {
        let win = main_window();
        assert_eq!(
            win["alwaysOnTop"].as_bool(),
            Some(true),
            "V5: alwaysOnTop must be true"
        );
    }

    #[test]
    fn test_window_decorations_false() {
        let win = main_window();
        assert_eq!(
            win["decorations"].as_bool(),
            Some(false),
            "frameless: decorations must be false"
        );
    }

    #[test]
    fn test_window_skip_taskbar_true() {
        let win = main_window();
        assert_eq!(
            win["skipTaskbar"].as_bool(),
            Some(true),
            "skipTaskbar must be true"
        );
    }

    #[test]
    fn test_window_width_360() {
        let win = main_window();
        assert_eq!(win["width"].as_u64(), Some(360));
        assert_eq!(win["minWidth"].as_u64(), Some(360));
        assert_eq!(win["maxWidth"].as_u64(), Some(360));
    }

    #[test]
    fn test_window_height_640() {
        let win = main_window();
        assert_eq!(win["height"].as_u64(), Some(640));
    }

    #[test]
    fn test_window_resizable_height_only() {
        let win = main_window();
        assert_eq!(win["resizable"].as_bool(), Some(true));
        assert!(win["minHeight"].as_u64().unwrap_or(0) < 640);
    }

    #[test]
    fn test_window_label_main() {
        let win = main_window();
        assert_eq!(win["label"].as_str(), Some("main"));
    }

    #[test]
    fn test_updater_endpoints_configured() {
        let conf = load_tauri_conf();
        let endpoints = conf["plugins"]["updater"]["endpoints"]
            .as_array()
            .expect("updater endpoints must be array");
        assert!(!endpoints.is_empty(), "V12: at least one endpoint required");
        let url = endpoints[0].as_str().expect("endpoint must be string");
        assert!(
            url.starts_with("https://"),
            "V13: updater endpoint must be HTTPS — unsigned/MITM risk"
        );
        assert!(
            url.contains("github.com/simondayce/zoho-desktop-widget/releases"),
            "I.updater: endpoint must point to GitHub releases"
        );
        assert!(
            url.ends_with("/latest.json"),
            "I.updater: endpoint must serve latest.json"
        );
    }

    #[test]
    fn test_create_updater_artifacts_true() {
        let conf = load_tauri_conf();
        assert_eq!(
            conf["bundle"]["createUpdaterArtifacts"].as_bool(),
            Some(true),
            "V13: createUpdaterArtifacts must be true for signed update artifacts"
        );
    }

    #[test]
    fn test_updater_pubkey_exists() {
        let conf = load_tauri_conf();
        let pubkey = conf["plugins"]["updater"]["pubkey"]
            .as_str()
            .expect("pubkey must exist");
        assert!(!pubkey.is_empty(), "V13: pubkey must not be empty — unsigned updates forbidden");
    }

    #[test]
    fn test_updater_pubkey_is_valid_minisign_format() {
        let conf = load_tauri_conf();
        let pubkey = conf["plugins"]["updater"]["pubkey"]
            .as_str()
            .expect("pubkey must exist");
        let decoded = base64_decode(pubkey);
        let decoded_str = String::from_utf8(decoded).unwrap_or_default();
        assert!(
            decoded_str.starts_with("untrusted comment:"),
            "V13: pubkey must be valid minisign public key (base64 of 'untrusted comment: ...')"
        );
    }

    #[test]
    fn test_private_key_not_tracked_in_git() {
        let key_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("keys")
            .join("update.key");
        if !key_path.exists() {
            return;
        }
        let output = std::process::Command::new("git")
            .args(["check-ignore", key_path.to_str().unwrap()])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output();
        let out = output.expect("git command failed — cannot verify private key is gitignored");
        assert!(
            out.status.success(),
            "V13: private key must be gitignored — found tracked at {:?}",
            key_path
        );
    }

    #[test]
    fn test_pubkey_matches_key_file() {
        let conf = load_tauri_conf();
        let conf_pubkey = conf["plugins"]["updater"]["pubkey"]
            .as_str()
            .expect("pubkey must exist in tauri.conf.json");
        let pub_key_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("keys")
            .join("update.key.pub");
        if !pub_key_path.exists() {
            return;
        }
        let file_pubkey = std::fs::read_to_string(&pub_key_path)
            .expect("failed to read update.key.pub")
            .trim()
            .to_string();
        assert_eq!(
            conf_pubkey, file_pubkey,
            "V13: pubkey in tauri.conf.json must match update.key.pub — mismatch means config is stale"
        );
    }

    fn base64_decode(s: &str) -> Vec<u8> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let mut result = Vec::new();
        let mut buffer = 0u32;
        let mut bits = 0;
        for c in s.bytes() {
            if c == b'=' {
                continue;
            }
            if let Some(pos) = CHARS.iter().position(|&x| x == c) {
                buffer = (buffer << 6) | pos as u32;
                bits += 6;
                if bits >= 8 {
                    bits -= 8;
                    result.push((buffer >> bits) as u8);
                    buffer &= (1 << bits) - 1;
                }
            }
        }
        result
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(TicketCache(Mutex::new(None)))
        .manage(tray::TrayState {
            aot_item: Mutex::new(None),
        })
        .manage(ws::ReconnectSignal(tokio::sync::Notify::new()))
        .invoke_handler(tauri::generate_handler![
            ws::get_current_tickets,
            ws::reconnect_ws,
        ])
        .on_window_event(|window, event| {
            match event {
                WindowEvent::Moved(_) => {
                    window_state::cache_window_position(window);
                }
                WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
                    window_state::cache_window_position(window);
                    window_state::flush_window_position(window);
                }
                _ => {}
            }
        })
        .setup(|app| {
            let (_tray, aot_item) = tray::build_tray(app.handle())?;
            if let Some(state) = app.try_state::<tray::TrayState>() {
                *state.aot_item.lock().unwrap() = Some(aot_item);
            }

            if let Some(window) = app.get_webview_window("main") {
                window_state::restore_window_position(&window);
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                ws::run_ws_client(handle).await;
            });
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                timer::run_timer(handle2).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
