use std::sync::Mutex;

use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow, Wry,
};
use tauri::tray::TrayIcon;

use crate::timer::TicketCategory;
use crate::TicketCache;

const TRAY_ID: &str = "main-tray";

const SHOW_LABEL: &str = "show_hide";
const AOT_LABEL: &str = "always_on_top";
const QUIT_LABEL: &str = "quit";

const DEFAULT_TOOLTIP: &str = "Zoho Widget";

pub struct TrayState {
    pub aot_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<(TrayIcon<Wry>, CheckMenuItem<Wry>)> {
    let show_hide = MenuItem::with_id(app, SHOW_LABEL, "Show/Hide", true, None::<&str>)?;
    let always_on_top =
        CheckMenuItem::with_id(app, AOT_LABEL, "Always on Top", true, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_LABEL, "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &always_on_top, &quit])?;

    let icon_bytes = include_bytes!("../icons/icon.png");
    let icon = Image::from_bytes(icon_bytes)?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip(DEFAULT_TOOLTIP)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_LABEL => {
                if let Some(window) = app.get_webview_window("main") {
                    toggle_window_visibility(&window);
                }
            }
            AOT_LABEL => {
                if let Some(window) = app.get_webview_window("main") {
                    toggle_always_on_top(app, &window);
                }
            }
            QUIT_LABEL => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    toggle_window_visibility(&window);
                }
            }
        })
        .build(app)?;

    Ok((tray, always_on_top))
}

fn toggle_window_visibility(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_always_on_top(app: &AppHandle, window: &WebviewWindow) {
    let current = window.is_always_on_top().unwrap_or(true);
    let new_state = !current;
    let _ = window.set_always_on_top(new_state);

    if let Some(state) = app.try_state::<TrayState>() {
        let guard = state.aot_item.lock().unwrap();
        if let Some(item) = guard.as_ref() {
            let _ = item.set_checked(new_state);
        }
    }
}

pub fn update_tray_badge(app: &AppHandle, asap_count: usize) {
    if let Some(tray) = app.tray_by_id(&TRAY_ID.to_string()) {
        let tooltip = if asap_count > 0 {
            format!("Zoho Widget — {} ASAP", asap_count)
        } else {
            DEFAULT_TOOLTIP.to_string()
        };
        let _ = tray.set_tooltip(Some(tooltip));
        let title = if asap_count > 0 {
            Some(format!("{}", asap_count))
        } else {
            None
        };
        let _ = tray.set_title(title);
    }
}

pub fn count_asap_tickets(cache: &TicketCache) -> usize {
    let guard = cache.0.lock().unwrap();
    match &*guard {
        Some(payload) => {
            let now = chrono::Utc::now().timestamp();
            payload
                .waiting_response
                .iter()
                .filter(|t| {
                    let elapsed = now - t.timestamp;
                    crate::timer::classify(elapsed) == TicketCategory::Asap
                })
                .count()
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::{TicketPayload, WaitingResponse};
    use std::sync::Mutex;

    fn make_ticket(id: &str, ts: i64) -> WaitingResponse {
        WaitingResponse {
            id_ticket: id.to_string(),
            department: "Support".to_string(),
            status_ticket: "Open".to_string(),
            customer_response_time: "2024-01-01 10:00".to_string(),
            subject: "Test".to_string(),
            timestamp: ts,
        }
    }

    #[test]
    fn test_count_asap_empty_cache() {
        let cache = TicketCache(Mutex::new(None));
        assert_eq!(count_asap_tickets(&cache), 0);
    }

    #[test]
    fn test_count_asap_no_tickets() {
        let payload = TicketPayload {
            total_ticket: vec![],
            onhold_ticket: vec![],
            waiting_response: vec![],
        };
        let cache = TicketCache(Mutex::new(Some(payload)));
        assert_eq!(count_asap_tickets(&cache), 0);
    }

    #[test]
    fn test_count_asap_with_asap_ticket() {
        let now = chrono::Utc::now().timestamp();
        let payload = TicketPayload {
            total_ticket: vec![],
            onhold_ticket: vec![],
            waiting_response: vec![make_ticket("T001", now - 1000)],
        };
        let cache = TicketCache(Mutex::new(Some(payload)));
        assert_eq!(count_asap_tickets(&cache), 1);
    }

    #[test]
    fn test_count_asap_mixed() {
        let now = chrono::Utc::now().timestamp();
        let payload = TicketPayload {
            total_ticket: vec![],
            onhold_ticket: vec![],
            waiting_response: vec![
                make_ticket("T001", now - 1000),
                make_ticket("T002", now - 100),
            ],
        };
        let cache = TicketCache(Mutex::new(Some(payload)));
        assert_eq!(count_asap_tickets(&cache), 1);
    }

    #[test]
    fn test_count_asap_all_asap() {
        let now = chrono::Utc::now().timestamp();
        let payload = TicketPayload {
            total_ticket: vec![],
            onhold_ticket: vec![],
            waiting_response: vec![
                make_ticket("T001", now - 1000),
                make_ticket("T002", now - 2000),
                make_ticket("T003", now - 3000),
            ],
        };
        let cache = TicketCache(Mutex::new(Some(payload)));
        assert_eq!(count_asap_tickets(&cache), 3);
    }
}
