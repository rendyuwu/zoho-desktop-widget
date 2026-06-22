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

impl TicketCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketCategory::New => "new",
            TicketCategory::Warning => "warning",
            TicketCategory::Asap => "asap",
        }
    }
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

fn truncate(s: &str, max: usize) -> &str {
    if s.chars().count() <= max {
        return s;
    }
    let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
    &s[..end]
}

pub async fn run_timer(app: AppHandle) {
    let mut prev_categories: HashMap<String, TicketCategory> = HashMap::new();

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
            }

            prev_categories.insert(ticket.id_ticket.clone(), current);
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
    fn test_category_as_str() {
        assert_eq!(TicketCategory::New.as_str(), "new");
        assert_eq!(TicketCategory::Warning.as_str(), "warning");
        assert_eq!(TicketCategory::Asap.as_str(), "asap");
    }

    #[test]
    fn test_ticket_move_event_serialization() {
        let event = TicketMoveEvent {
            id_ticket: "T001".to_string(),
            from: TicketCategory::New,
            to: TicketCategory::Asap,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"id_ticket\":\"T001\""));
        assert!(json.contains("\"from\":\"new\""));
        assert!(json.contains("\"to\":\"asap\""));
    }

    #[test]
    fn test_ticket_move_all_transitions() {
        let transitions = vec![
            (TicketCategory::New, TicketCategory::Warning),
            (TicketCategory::Warning, TicketCategory::Asap),
            (TicketCategory::New, TicketCategory::Asap),
            (TicketCategory::Asap, TicketCategory::New),
        ];
        for (from, to) in transitions {
            assert_ne!(from, to);
        }
    }

    #[test]
    fn test_should_notify_asap_from_warning() {
        assert!(should_notify_asap(TicketCategory::Warning, TicketCategory::Asap));
    }

    #[test]
    fn test_should_notify_asap_from_new() {
        assert!(should_notify_asap(TicketCategory::New, TicketCategory::Asap));
    }

    #[test]
    fn test_should_not_notify_on_non_asap_transition() {
        assert!(!should_notify_asap(TicketCategory::New, TicketCategory::Warning));
        assert!(!should_notify_asap(TicketCategory::Warning, TicketCategory::New));
    }

    #[test]
    fn test_should_not_notify_staying_asap() {
        assert!(!should_notify_asap(TicketCategory::Asap, TicketCategory::Asap));
    }

    #[test]
    fn test_should_not_notify_leaving_asap() {
        assert!(!should_notify_asap(TicketCategory::Asap, TicketCategory::New));
        assert!(!should_notify_asap(TicketCategory::Asap, TicketCategory::Warning));
    }

    #[test]
    fn test_should_not_notify_staying_new_or_warning() {
        assert!(!should_notify_asap(TicketCategory::New, TicketCategory::New));
        assert!(!should_notify_asap(TicketCategory::Warning, TicketCategory::Warning));
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
}
