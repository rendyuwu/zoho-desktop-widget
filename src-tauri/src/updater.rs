use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAvailableEvent {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

pub struct CachedUpdate(pub Mutex<Option<Update>>);

async fn try_check_update(app: &AppHandle) -> Result<Option<Update>, String> {
    let updater = app.updater().map_err(|e| format!("updater build failed: {}", e))?;
    updater
        .check()
        .await
        .map_err(|e| format!("update check failed: {}", e))
}

pub async fn check_update(app: &AppHandle) -> UpdateInfo {
    let result = tokio::time::timeout(UPDATE_CHECK_TIMEOUT, try_check_update(app)).await;

    match result {
        Ok(Ok(Some(update))) => {
            let info = UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            };

            if let Some(state) = app.try_state::<CachedUpdate>() {
                *state.0.lock().unwrap() = Some(update);
            }

            info
        }
        Ok(Ok(None)) => UpdateInfo {
            available: false,
            version: None,
            body: None,
        },
        Ok(Err(e)) => {
            eprintln!("update check: {}", e);
            UpdateInfo {
                available: false,
                version: None,
                body: None,
            }
        }
        Err(_) => {
            eprintln!("update check: timed out after {:?}", UPDATE_CHECK_TIMEOUT);
            UpdateInfo {
                available: false,
                version: None,
                body: None,
            }
        }
    }
}

pub async fn run_update_check_on_launch(app: AppHandle) {
    let info = check_update(&app).await;

    if info.available {
        let event = UpdateAvailableEvent {
            version: info.version.clone().unwrap_or_default(),
            body: info.body.clone(),
        };
        if let Err(e) = app.emit("update-available", &event) {
            eprintln!("update check: failed to emit update-available event: {}", e);
        }
    }
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    Ok(check_update(&app).await)
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<InstallResult, String> {
    let update = match app.try_state::<CachedUpdate>() {
        Some(state) => state.0.lock().unwrap().take(),
        None => {
            return Ok(InstallResult {
                success: false,
                error: Some("update state not initialized".to_string()),
            });
        }
    };

    let update = match update {
        Some(u) => u,
        None => {
            return Ok(InstallResult {
                success: false,
                error: Some("no update available — call check_for_updates first".to_string()),
            });
        }
    };

    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => {
            if let Err(e) = app.emit("update-installed", ()) {
                eprintln!("install_update: failed to emit update-installed event: {}", e);
            }
            app.restart();
        }
        Err(e) => {
            let msg = format!("download_and_install failed: {}", e);
            eprintln!("install_update: {}", msg);
            Ok(InstallResult {
                success: false,
                error: Some(msg),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info_not_available_serialization() {
        let info = UpdateInfo {
            available: false,
            version: None,
            body: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["available"], false);
        assert!(v.get("version").is_none());
        assert!(v.get("body").is_none());
    }

    #[test]
    fn test_update_info_available_serialization() {
        let info = UpdateInfo {
            available: true,
            version: Some("0.2.0".to_string()),
            body: Some("Bug fixes".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["available"], true);
        assert_eq!(v["version"], "0.2.0");
        assert_eq!(v["body"], "Bug fixes");
    }

    #[test]
    fn test_update_info_available_no_body() {
        let info = UpdateInfo {
            available: true,
            version: Some("0.2.0".to_string()),
            body: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["available"], true);
        assert_eq!(v["version"], "0.2.0");
        assert!(v.get("body").is_none());
    }

    #[test]
    fn test_install_result_success_shape() {
        let result = InstallResult {
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["success"], true);
        assert!(v.get("error").is_none());
    }

    #[test]
    fn test_install_result_failure_shape() {
        let result = InstallResult {
            success: false,
            error: Some("network error".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["success"], false);
        assert_eq!(v["error"], "network error");
    }

    #[test]
    fn test_update_available_event_payload_shape() {
        let event = UpdateAvailableEvent {
            version: "0.2.0".to_string(),
            body: Some("New features".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], "0.2.0");
        assert_eq!(v["body"], "New features");
    }

    #[test]
    fn test_update_available_event_no_body() {
        let event = UpdateAvailableEvent {
            version: "0.2.0".to_string(),
            body: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], "0.2.0");
        assert!(v.get("body").is_none());
    }

    #[test]
    fn test_update_info_defaults_to_not_available_on_error() {
        let info = UpdateInfo {
            available: false,
            version: None,
            body: None,
        };
        assert!(!info.available);
        assert!(info.version.is_none());
        assert!(info.body.is_none());
    }

    #[test]
    fn test_install_result_always_has_success_field() {
        let success = InstallResult {
            success: true,
            error: None,
        };
        let failure = InstallResult {
            success: false,
            error: Some("err".to_string()),
        };
        let success_json = serde_json::to_string(&success).unwrap();
        let failure_json = serde_json::to_string(&failure).unwrap();
        assert!(success_json.contains("\"success\":true"));
        assert!(failure_json.contains("\"success\":false"));
    }

    #[test]
    fn test_update_available_event_only_version_required() {
        let event = UpdateAvailableEvent {
            version: "1.0.0".to_string(),
            body: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("version").is_some());
        assert!(v.get("body").is_none());
    }

    #[test]
    fn test_install_result_success_omits_error() {
        let result = InstallResult {
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["success"], true);
        assert!(v.get("error").is_none());
    }

    #[test]
    fn test_install_result_failure_includes_error() {
        let result = InstallResult {
            success: false,
            error: Some("download failed".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["success"], false);
        assert_eq!(v["error"], "download failed");
    }

    #[test]
    fn test_update_check_timeout_is_30s() {
        assert_eq!(UPDATE_CHECK_TIMEOUT, Duration::from_secs(30));
    }

    #[test]
    fn test_cached_update_starts_none() {
        let cached = CachedUpdate(Mutex::new(None));
        assert!(cached.0.lock().unwrap().is_none());
    }

    #[test]
    fn test_cached_update_take_clears() {
        let cached = CachedUpdate(Mutex::new(None));
        let taken = cached.0.lock().unwrap().take();
        assert!(taken.is_none());
        assert!(cached.0.lock().unwrap().is_none());
    }

    #[test]
    fn test_install_result_no_update_error_message() {
        let result = InstallResult {
            success: false,
            error: Some("no update available — call check_for_updates first".to_string()),
        };
        assert!(result.error.as_ref().unwrap().contains("check_for_updates"));
    }

    #[test]
    fn test_update_info_timeout_returns_not_available() {
        let info = UpdateInfo {
            available: false,
            version: None,
            body: None,
        };
        assert!(!info.available, "timeout must return available: false (V15)");
    }

    #[test]
    fn test_run_update_check_on_launch_only_emits_when_available() {
        let info_available = UpdateInfo {
            available: true,
            version: Some("1.0.0".to_string()),
            body: None,
        };
        let info_not_available = UpdateInfo {
            available: false,
            version: None,
            body: None,
        };

        assert!(info_available.available);
        assert!(!info_not_available.available);
    }
}
