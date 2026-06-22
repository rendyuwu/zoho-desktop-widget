use std::sync::Mutex;

use tauri::{Manager, WindowEvent};

mod timer;
mod tray;
mod window_state;
mod ws;

use ws::TicketPayload;

pub struct TicketCache(pub Mutex<Option<TicketPayload>>);

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
