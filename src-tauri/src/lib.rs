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
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
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
