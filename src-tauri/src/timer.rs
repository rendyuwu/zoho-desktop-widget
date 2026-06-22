use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::TicketCache;

const TICK_INTERVAL: Duration = Duration::from_secs(3);
const WARNING_THRESHOLD: i64 = 600;
const ASAP_THRESHOLD: i64 = 900;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TicketCategory {
    New,
    Warning,
    Asap,
}

#[derive(Debug, Clone, Serialize)]
pub struct TicketMoveEvent {
    pub id_ticket: String,
    pub from: TicketCategory,
    pub to: TicketCategory,
}

pub fn classify(elapsed: i64) -> TicketCategory {
    if elapsed >= ASAP_THRESHOLD {
        TicketCategory::Asap
    } else if elapsed >= WARNING_THRESHOLD {
        TicketCategory::Warning
    } else {
        TicketCategory::New
    }
}

pub fn should_notify_asap(from: TicketCategory, to: TicketCategory) -> bool {
    to == TicketCategory::Asap && from != TicketCategory::Asap
}

pub fn fire_asap_notification(app: &AppHandle, id_ticket: &str, department: &str, subject: &str) {
    let title = format!("Ticket #{} → ASAP", id_ticket);
    let body = format!("[{}] {}", department, truncate(subject, 60));
    let _ = app.notification().builder()
        .title(&title)
        .body(&body)
        .show();
}

pub fn fire_new_ticket_notification(app: &AppHandle, id_ticket: &str, department: &str, subject: &str) {
    let title = format!("New ticket #{}", id_ticket);
    let body = format!("[{}] {}", department, truncate(subject, 60));
    let _ = app.notification().builder()
        .title(&title)
        .body(&body)
        .show();
}

fn truncate(s: &str, max: usize) -> &str {
    if s.chars().count() <= max {
        return s;
    }
    let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
    &s[..end]
}

pub async fn run_timer(app: AppHandle) {
    let mut prev_categories: HashMap<String, TicketCategory> = HashMap::new();
    let mut prev_asap_count: usize = 0;
    let mut seeded = false;

    loop {
        tokio::time::sleep(TICK_INTERVAL).await;

        let now = chrono::Utc::now().timestamp();

        let waiting = {
            let cache = app.state::<TicketCache>();
            let guard = cache.0.lock().unwrap();
            match &*guard {
                Some(payload) => payload.waiting_response.clone(),
                None => Vec::new(),
            }
        };

        if waiting.is_empty() {
            if prev_asap_count != 0 {
                prev_asap_count = 0;
                crate::tray::update_tray_badge(&app, 0);
            }
            continue;
        }

        for ticket in &waiting {
            let elapsed = now - ticket.timestamp;
            let current = classify(elapsed);

            if let Some(&prev) = prev_categories.get(&ticket.id_ticket) {
                if prev != current {
                    let event = TicketMoveEvent {
                        id_ticket: ticket.id_ticket.clone(),
                        from: prev,
                        to: current,
                    };
                    let _ = app.emit("ticket-move", &event);

                    if should_notify_asap(prev, current) {
                        fire_asap_notification(
                            &app,
                            &ticket.id_ticket,
                            &ticket.department,
                            &ticket.subject,
                        );
                    }
                }
            } else if seeded {
                fire_new_ticket_notification(
                    &app,
                    &ticket.id_ticket,
                    &ticket.department,
                    &ticket.subject,
                );
            }

            prev_categories.insert(ticket.id_ticket.clone(), current);
        }

        let current_ids: std::collections::HashSet<&String> =
            waiting.iter().map(|t| &t.id_ticket).collect();
        prev_categories.retain(|id, _| current_ids.contains(id));

        if !seeded {
            seeded = true;
        }

        let asap_count = {
            let cache = app.state::<TicketCache>();
            crate::tray::count_asap_tickets(&*cache)
        };
        if asap_count != prev_asap_count {
            prev_asap_count = asap_count;
            crate::tray::update_tray_badge(&app, asap_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_new() {
        assert_eq!(classify(0), TicketCategory::New);
        assert_eq!(classify(300), TicketCategory::New);
        assert_eq!(classify(599), TicketCategory::New);
    }

    #[test]
    fn test_classify_warning() {
        assert_eq!(classify(600), TicketCategory::Warning);
        assert_eq!(classify(750), TicketCategory::Warning);
        assert_eq!(classify(899), TicketCategory::Warning);
    }

    #[test]
    fn test_classify_asap() {
        assert_eq!(classify(900), TicketCategory::Asap);
        assert_eq!(classify(1200), TicketCategory::Asap);
        assert_eq!(classify(9999), TicketCategory::Asap);
    }

    #[test]
    fn test_classify_boundary_600() {
        assert_eq!(classify(599), TicketCategory::New);
        assert_eq!(classify(600), TicketCategory::Warning);
        assert_ne!(classify(600), TicketCategory::New);
    }

    #[test]
    fn test_classify_boundary_900() {
        assert_eq!(classify(899), TicketCategory::Warning);
        assert_eq!(classify(900), TicketCategory::Asap);
        assert_ne!(classify(900), TicketCategory::Warning);
    }

    #[test]
    fn test_ticket_move_event_serialization() {
        let cases = vec![
            (TicketCategory::New, "new"),
            (TicketCategory::Warning, "warning"),
            (TicketCategory::Asap, "asap"),
        ];
        for (cat, expected) in cases {
            let event = TicketMoveEvent {
                id_ticket: "T001".to_string(),
                from: cat,
                to: cat,
            };
            let json = serde_json::to_string(&event).unwrap();
            assert!(json.contains("\"id_ticket\":\"T001\""));
            assert!(json.contains(&format!("\"from\":\"{}\"", expected)));
            assert!(json.contains(&format!("\"to\":\"{}\"", expected)));
        }
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 60), "hello");
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let long = "a".repeat(100);
        let result = truncate(&long, 60);
        assert_eq!(result.chars().count(), 60);
    }

    #[test]
    fn test_truncate_multibyte() {
        let s = "日本語テスト".repeat(20);
        let result = truncate(&s, 10);
        assert_eq!(result.chars().count(), 10);
    }

    #[test]
    fn test_tick_interval_is_3s() {
        assert_eq!(TICK_INTERVAL, Duration::from_secs(3));
    }

    #[test]
    fn test_warning_threshold_is_600s() {
        assert_eq!(WARNING_THRESHOLD, 600);
    }

    #[test]
    fn test_asap_threshold_is_900s() {
        assert_eq!(ASAP_THRESHOLD, 900);
    }

    #[test]
    fn test_ticket_move_new_to_warning_at_600s() {
        let prev = classify(599);
        let curr = classify(600);
        assert_ne!(prev, curr);
        assert_eq!(prev, TicketCategory::New);
        assert_eq!(curr, TicketCategory::Warning);
    }

    #[test]
    fn test_ticket_move_warning_to_asap_at_900s() {
        let prev = classify(899);
        let curr = classify(900);
        assert_ne!(prev, curr);
        assert_eq!(prev, TicketCategory::Warning);
        assert_eq!(curr, TicketCategory::Asap);
    }

    #[test]
    fn test_ticket_move_new_to_asap_skip_warning() {
        let prev = classify(599);
        let curr = classify(900);
        assert_eq!(prev, TicketCategory::New);
        assert_eq!(curr, TicketCategory::Asap);
        assert_ne!(prev, curr);
    }

    #[test]
    fn test_ticket_stays_in_category_no_move() {
        assert_eq!(classify(0), classify(300));
        assert_eq!(classify(600), classify(800));
        assert_eq!(classify(900), classify(5000));
    }

    #[test]
    fn test_ticket_move_asap_back_to_new() {
        let prev = classify(900);
        let curr = classify(100);
        assert_eq!(prev, TicketCategory::Asap);
        assert_eq!(curr, TicketCategory::New);
        assert_ne!(prev, curr);
    }

    #[test]
    fn test_notification_title_format() {
        let title = format!("Ticket #{} → ASAP", "T001");
        assert!(title.starts_with("Ticket #"));
        assert!(title.ends_with("→ ASAP"));
        assert!(title.contains("T001"));
    }

    #[test]
    fn test_notification_body_format() {
        let body = format!("[{}] {}", "Support", truncate("Login issue", 60));
        assert!(body.starts_with("[Support] "));
        assert!(body.contains("Login issue"));
    }

    #[test]
    fn test_notification_body_truncates_subject() {
        let long_subject = "a".repeat(100);
        let body = format!("[{}] {}", "Dept", truncate(&long_subject, 60));
        let body_after_bracket = body.strip_prefix("[Dept] ").unwrap();
        assert_eq!(body_after_bracket.chars().count(), 60);
    }

    #[test]
    fn test_notification_fires_exactly_when_crossing_to_asap() {
        let all_categories = vec![
            TicketCategory::New,
            TicketCategory::Warning,
            TicketCategory::Asap,
        ];
        for from in &all_categories {
            for to in &all_categories {
                let fires = should_notify_asap(*from, *to);
                let should_fire = *to == TicketCategory::Asap && *from != TicketCategory::Asap;
                assert_eq!(fires, should_fire, "from={:?} to={:?}", from, to);
            }
        }
    }

    #[test]
    fn test_new_ticket_notification_title_format() {
        let title = format!("New ticket #{}", "T001");
        assert!(title.starts_with("New ticket #"));
        assert!(title.contains("T001"));
    }
}
