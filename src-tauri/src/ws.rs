use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::TicketCache;

const WS_URL: &str = env!("ZOHO_WS_URL");
const BACKOFF_SEQUENCE: &[u64] = &[1, 2, 5, 10, 30];

pub struct ReconnectSignal(pub Notify);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalTicket {
    pub status: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnholdTicket {
    pub tag: String,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingResponse {
    pub id_ticket: String,
    pub department: String,
    pub status_ticket: String,
    pub customer_response_time: String,
    pub subject: String,
    pub timestamp: i64,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt: Option<Vec<T>> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketPayload {
    pub total_ticket: Vec<TotalTicket>,
    pub onhold_ticket: Vec<OnholdTicket>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub waiting_response: Vec<WaitingResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub data: TicketPayload,
}

pub async fn run_ws_client(app: AppHandle) {
    let mut backoff_index: usize = 0;
    let reconnect_notify = app.state::<ReconnectSignal>();

    loop {
        eprintln!("WS connecting");

        match connect_async(WS_URL).await {
            Ok((ws_stream, _response)) => {
                eprintln!("WS connected");
                backoff_index = 0;

                let (mut write, mut read) = ws_stream.split();

                let _ = write.send(Message::Text("GET".into())).await;

                let mut force_reconnect = false;

                loop {
                    tokio::select! {
                        msg_result = read.next() => {
                            match msg_result {
                                Some(Ok(Message::Text(text))) => {
                                    handle_message(&app, &text);
                                }
                                Some(Ok(Message::Binary(data))) => {
                                    if let Ok(text) = String::from_utf8(data.to_vec()) {
                                        handle_message(&app, &text);
                                    }
                                }
                                Some(Ok(Message::Close(_))) => {
                                    eprintln!("WS closed by server");
                                    break;
                                }
                                Some(Ok(_)) => {}
                                Some(Err(e)) => {
                                    eprintln!("WS error: {}", e);
                                    break;
                                }
                                None => {
                                    eprintln!("WS stream ended");
                                    break;
                                }
                            }
                        }
                        _ = reconnect_notify.0.notified() => {
                            eprintln!("WS reconnect requested");
                            force_reconnect = true;
                            break;
                        }
                    }
                }

                if force_reconnect {
                    let _ = write.send(Message::Close(None)).await;
                    backoff_index = 0;
                    continue;
                }

                eprintln!("WS disconnected. reconnecting...");
            }
            Err(e) => {
                eprintln!("WS connect failed: {}", e);
            }
        }

        let delay = next_backoff(backoff_index);
        eprintln!("reconnect backoff: {:?}", delay);

        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = reconnect_notify.0.notified() => {
                eprintln!("WS reconnect requested during backoff");
                backoff_index = 0;
                continue;
            }
        }

        if backoff_index < BACKOFF_SEQUENCE.len() - 1 {
            backoff_index += 1;
        }
    }
}

fn handle_message(app: &AppHandle, text: &str) {
    match serde_json::from_str::<WsMessage>(text) {
        Ok(msg) => {
            let cache = app.state::<TicketCache>();
            {
                let mut guard = cache.0.lock().unwrap();
                *guard = Some(msg.data.clone());
            }
            let _ = app.emit("ticket-data", &msg.data);
        }
        Err(e) => {
            eprintln!("JSON parse error: {} | raw: {}...", e, text.chars().take(100).collect::<String>());
        }
    }
}

pub fn next_backoff(attempt: usize) -> Duration {
    let idx = attempt.min(BACKOFF_SEQUENCE.len() - 1);
    Duration::from_secs(BACKOFF_SEQUENCE[idx])
}

#[tauri::command]
pub fn get_current_tickets(cache: tauri::State<'_, TicketCache>) -> Option<TicketPayload> {
    let guard = cache.0.lock().unwrap();
    guard.clone()
}

#[tauri::command]
pub fn reconnect_ws(app: AppHandle) -> Result<(), String> {
    let notify = app.state::<ReconnectSignal>();
    notify.0.notify_one();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_backoff_sequence() {
        assert_eq!(next_backoff(0), Duration::from_secs(1));
        assert_eq!(next_backoff(1), Duration::from_secs(2));
        assert_eq!(next_backoff(2), Duration::from_secs(5));
        assert_eq!(next_backoff(3), Duration::from_secs(10));
        assert_eq!(next_backoff(4), Duration::from_secs(30));
    }

    #[test]
    fn test_backoff_cap() {
        assert_eq!(next_backoff(5), Duration::from_secs(30));
        assert_eq!(next_backoff(10), Duration::from_secs(30));
        assert_eq!(next_backoff(100), Duration::from_secs(30));
    }

    #[test]
    fn test_parse_valid_json() {
        let raw = r#"{
            "data": {
                "total_ticket": [
                    {"status": "Open", "total": 5},
                    {"status": "On Progress", "total": 3}
                ],
                "onhold_ticket": [
                    {"tag": "abuse", "total": 2}
                ],
                "waiting_response": [
                    {
                        "id_ticket": "T001",
                        "department": "Support",
                        "status_ticket": "Open",
                        "customer_response_time": "2024-01-01 10:00",
                        "subject": "Login issue",
                        "timestamp": 1704067200
                    }
                ]
            }
        }"#;

        let msg: WsMessage = serde_json::from_str(raw).unwrap();
        assert_eq!(msg.data.total_ticket.len(), 2);
        assert_eq!(msg.data.total_ticket[0].status, "Open");
        assert_eq!(msg.data.total_ticket[0].total, 5);
        assert_eq!(msg.data.onhold_ticket.len(), 1);
        assert_eq!(msg.data.onhold_ticket[0].tag, "abuse");
        assert_eq!(msg.data.waiting_response.len(), 1);
        assert_eq!(msg.data.waiting_response[0].id_ticket, "T001");
        assert_eq!(msg.data.waiting_response[0].timestamp, 1704067200);
    }

    #[test]
    fn test_parse_empty_arrays() {
        let raw = r#"{
            "data": {
                "total_ticket": [],
                "onhold_ticket": [],
                "waiting_response": []
            }
        }"#;

        let msg: WsMessage = serde_json::from_str(raw).unwrap();
        assert!(msg.data.total_ticket.is_empty());
        assert!(msg.data.onhold_ticket.is_empty());
        assert!(msg.data.waiting_response.is_empty());
    }

    #[test]
    fn test_parse_null_waiting_response() {
        let raw = r#"{
            "data": {
                "total_ticket": [{"status": "Open", "total": 1}],
                "onhold_ticket": [],
                "waiting_response": null
            }
        }"#;

        let msg: WsMessage = serde_json::from_str(raw).unwrap();
        assert!(msg.data.waiting_response.is_empty());
        assert_eq!(msg.data.total_ticket[0].status, "Open");
    }

    #[test]
    fn test_cache_returns_none_when_empty() {
        let cache = TicketCache(Mutex::new(None));
        let guard = cache.0.lock().unwrap();
        assert!(guard.is_none());
    }

    #[test]
    fn test_cache_returns_data_when_populated() {
        let payload = TicketPayload {
            total_ticket: vec![TotalTicket { status: "Open".to_string(), total: 5 }],
            onhold_ticket: vec![],
            waiting_response: vec![],
        };
        let cache = TicketCache(Mutex::new(Some(payload.clone())));
        let guard = cache.0.lock().unwrap();
        let data = guard.as_ref().unwrap();
        assert_eq!(data.total_ticket.len(), 1);
        assert_eq!(data.total_ticket[0].status, "Open");
        assert_eq!(data.total_ticket[0].total, 5);
    }

    #[tokio::test]
    async fn test_reconnect_signal_fires() {
        let notify = std::sync::Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let notified = tokio::spawn(async move {
            notify_clone.notified().await;
        });

        notify.notify_one();

        tokio::time::timeout(Duration::from_secs(1), notified)
            .await
            .expect("notify signal did not fire in time")
            .expect("spawned task panicked");
    }

    #[tokio::test]
    async fn test_reconnect_signal_no_panic_without_listener() {
        let notify = Notify::new();
        notify.notify_one();
    }

    #[test]
    fn test_backoff_sequence_matches_spec() {
        assert_eq!(BACKOFF_SEQUENCE, &[1, 2, 5, 10, 30]);
    }

    #[test]
    fn test_backoff_monotonic_non_decreasing() {
        let mut prev = Duration::from_secs(0);
        for i in 0..BACKOFF_SEQUENCE.len() {
            let d = next_backoff(i);
            assert!(d >= prev, "backoff at step {} decreased: {:?} < {:?}", i, d, prev);
            prev = d;
        }
    }

    #[test]
    fn test_backoff_caps_at_30s_beyond_sequence() {
        for i in BACKOFF_SEQUENCE.len()..100 {
            assert_eq!(next_backoff(i), Duration::from_secs(30));
        }
    }

    #[test]
    fn test_ws_url_resolves() {
        // ZOHO_WS_URL env var baked at compile time via env!. No fallback.
        assert!(
            WS_URL.starts_with("wss://") || WS_URL.starts_with("ws://"),
            "WS_URL must be valid ws/wss URL, got: {}",
            WS_URL
        );
        assert!(!WS_URL.is_empty(), "WS_URL must not be empty");
    }
}
