use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, Runtime, WebviewWindow, Window};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "store.json";
const WINDOW_POS_KEY: &str = "window-position";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

pub fn cache_window_position<R: Runtime>(window: &Window<R>) {
    let Ok(pos) = window.outer_position() else {
        return;
    };

    let wp = WindowPosition {
        x: pos.x,
        y: pos.y,
    };

    let Ok(store) = window.store(STORE_PATH) else {
        return;
    };

    let json = serde_json::to_value(&wp).unwrap_or(serde_json::Value::Null);
    store.set(WINDOW_POS_KEY, json);
}

pub fn flush_window_position<R: Runtime>(window: &Window<R>) {
    let Ok(store) = window.store(STORE_PATH) else {
        return;
    };
    let _ = store.save();
}

pub fn restore_window_position<R: Runtime>(window: &WebviewWindow<R>) {
    let Ok(store) = window.store(STORE_PATH) else {
        return;
    };

    if let Some(val) = store.get(WINDOW_POS_KEY) {
        if let Ok(wp) = serde_json::from_value::<WindowPosition>(val) {
            let _ = window.set_position(PhysicalPosition::new(wp.x, wp.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_position_serde_roundtrip() {
        let wp = WindowPosition { x: 100, y: 200 };
        let json = serde_json::to_string(&wp).unwrap();
        let deserialized: WindowPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(wp, deserialized);
    }

    #[test]
    fn test_window_position_negative_coords() {
        let wp = WindowPosition { x: -50, y: -100 };
        let json = serde_json::to_string(&wp).unwrap();
        let deserialized: WindowPosition = serde_json::from_str(&json).unwrap();
        assert_eq!(wp, deserialized);
    }

    #[test]
    fn test_window_position_json_shape() {
        let wp = WindowPosition { x: 360, y: 640 };
        let json = serde_json::to_string(&wp).unwrap();
        assert!(json.contains("\"x\":360"));
        assert!(json.contains("\"y\":640"));
    }

    #[test]
    fn test_window_position_from_json() {
        let json = r#"{"x":0,"y":0}"#;
        let wp: WindowPosition = serde_json::from_str(json).unwrap();
        assert_eq!(wp, WindowPosition { x: 0, y: 0 });
    }
}
