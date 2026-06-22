use std::sync::Mutex;

use tauri::Manager;

mod timer;
mod tray;
mod ws;

use ws::TicketPayload;

pub struct TicketCache(pub Mutex<Option<TicketPayload>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(TicketCache(Mutex::new(None)))
        .manage(tray::TrayState {
            aot_item: Mutex::new(None),
        })
        .manage(ws::ReconnectSignal(tokio::sync::Notify::new()))
        .invoke_handler(tauri::generate_handler![
            ws::get_current_tickets,
            ws::reconnect_ws,
        ])
        .setup(|app| {
            let (_tray, aot_item) = tray::build_tray(app.handle())?;
            if let Some(state) = app.try_state::<tray::TrayState>() {
                *state.aot_item.lock().unwrap() = Some(aot_item);
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
