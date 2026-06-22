use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::TicketCache;

const WS_URL: &str = "wss://your-domain.com/zoho/wss";
const BACKOFF_SEQUENCE: &[u64] = &[1, 2, 5, 10, 30];

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

    loop {
        eprintln!("WS connecting to {}", WS_URL);

        match connect_async(WS_URL).await {
            Ok((ws_stream, _response)) => {
                eprintln!("WS connected");
                backoff_index = 0;

                let (mut write, mut read) = ws_stream.split();

                let _ = write.send(Message::Text("GET".into())).await;

                while let Some(msg_result) = read.next().await {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            handle_message(&app, &text);
                        }
                        Ok(Message::Binary(data)) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                handle_message(&app, &text);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            eprintln!("WS closed by server");
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("WS error: {}", e);
                            break;
                        }
                    }
                }

                eprintln!("WS disconnected. reconnecting...");
            }
            Err(e) => {
                eprintln!("WS connect failed: {}", e);
            }
        }

        let delay = next_backoff(backoff_index);
        eprintln!("reconnect backoff: {:?}", delay);
        tokio::time::sleep(delay).await;

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
            eprintln!("JSON parse error: {} | raw: {}...", e, &text[..text.len().min(100)]);
        }
    }
}

pub fn next_backoff(attempt: usize) -> Duration {
    let idx = attempt.min(BACKOFF_SEQUENCE.len() - 1);
    Duration::from_secs(BACKOFF_SEQUENCE[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
