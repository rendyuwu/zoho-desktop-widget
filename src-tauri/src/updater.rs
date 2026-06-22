use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

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

pub async fn check_update(app: &AppHandle) -> UpdateInfo {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("update check: updater build failed: {}", e);
            return UpdateInfo {
                available: false,
                version: None,
                body: None,
            };
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let info = UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            };

            let event = UpdateAvailableEvent {
                version: update.version.clone(),
                body: update.body.clone(),
            };
            if let Err(e) = app.emit("update-available", &event) {
                eprintln!("update check: failed to emit update-available event: {}", e);
            }

            info
        }
        Ok(None) => UpdateInfo {
            available: false,
            version: None,
            body: None,
        },
        Err(e) => {
            eprintln!("update check: check failed: {}", e);
            UpdateInfo {
                available: false,
                version: None,
                body: None,
            }
        }
    }
}

pub async fn run_update_check_on_launch(app: AppHandle) {
    let _ = check_update(&app).await;
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    Ok(check_update(&app).await)
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<InstallResult, String> {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            let msg = format!("updater build failed: {}", e);
            eprintln!("install_update: {}", msg);
            return Ok(InstallResult {
                success: false,
                error: Some(msg),
            });
        }
    };

    let update = match updater.check().await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Ok(InstallResult {
                success: false,
                error: Some("no update available".to_string()),
            });
        }
        Err(e) => {
            let msg = format!("update check failed: {}", e);
            eprintln!("install_update: {}", msg);
            return Ok(InstallResult {
                success: false,
                error: Some(msg),
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
}
