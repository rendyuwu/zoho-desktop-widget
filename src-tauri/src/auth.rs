//! Direct-bind LDAP authentication for the desktop widget.
//!
//! This binary is published on a public GitHub release, so — unlike the
//! inlethq server — it must NOT carry a service-account password. We bind
//! directly as the user against a DN/UPN built from a compile-time template,
//! baked from build secrets exactly like `ZOHO_WS_URL` (see `ws.rs`).
//!
//! Security model:
//! - No service credential is embedded; `strings <binary>` leaks only the
//!   server host and the DN/UPN shape, both useless without VPN + valid creds.
//! - The LDAP server is reachable only on the corporate VPN, so a public
//!   download cannot authenticate off-VPN.
//! - "Remember me" stores the password in the OS keychain, never plaintext.
//!
//! REMAINING GAP: the directory currently listens on plain `ldap://` (port
//! 389), so the bind sends the user's password in cleartext on the wire. This
//! mirrors the inlethq backend (`LDAP_ALLOW_INSECURE_TRANSPORT=true`) and is
//! gated behind the `LDAP_ALLOW_INSECURE` build flag below. Moving the
//! directory to `ldaps://` (636) and dropping that flag closes this gap.

use std::time::Duration;

use ldap3::{LdapConnAsync, LdapConnSettings};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// LDAP server URI, baked at compile time (e.g. "ldap://10.20.206.11:389").
const LDAP_SERVER_URI: &str = env!("LDAP_SERVER_URI");

/// Bind DN/UPN template, baked at compile time. `{user}` is replaced with the
/// DN-escaped username, e.g. "{user}@biznetgio.com" (UPN) or
/// "uid={user},ou=people,dc=corp,dc=com" (DN).
const LDAP_BIND_TEMPLATE: &str = env!("LDAP_BIND_TEMPLATE");

/// When "true", allow binding over plain `ldap://` (cleartext password on the
/// wire). Optional — defaults to disabled so `ldaps://` is required.
const LDAP_ALLOW_INSECURE: Option<&str> = option_env!("LDAP_ALLOW_INSECURE");

const LDAP_TIMEOUT: Duration = Duration::from_secs(10);

const KEYRING_SERVICE: &str = "zoho-desktop-widget";
const KEYRING_ACCOUNT: &str = "ldap-credential";

/// Authentication failures, mapped to safe, generic user-facing messages.
/// We never echo raw LDAP/server detail to the UI (no user enumeration, no
/// internal topology leak).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthError {
    /// Wrong username or password.
    InvalidCredentials,
    /// Could not reach / talk to the directory (likely VPN down).
    Connection,
    /// Misconfigured build (e.g. insecure transport not allowed).
    Config,
}

impl AuthError {
    fn message(self) -> String {
        match self {
            AuthError::InvalidCredentials => "Invalid username or password.".to_string(),
            AuthError::Connection => {
                "Cannot reach the authentication server. Check your VPN connection.".to_string()
            }
            AuthError::Config => {
                "Authentication is misconfigured in this build. Contact the maintainer.".to_string()
            }
        }
    }
}

fn insecure_allowed() -> bool {
    matches!(LDAP_ALLOW_INSECURE, Some(v) if v.eq_ignore_ascii_case("true"))
}

/// Build the bind identity from the template, DN-escaping the username so it
/// cannot break out of an RDN (defense even though a bind DN is not a filter).
fn build_bind_dn(username: &str) -> String {
    let escaped = ldap3::dn_escape(username);
    LDAP_BIND_TEMPLATE.replace("{user}", &escaped)
}

/// Verify a username/password against LDAP by binding directly as the user.
async fn bind(username: &str, password: &str) -> Result<(), AuthError> {
    // Empty fields can produce an "unauthenticated bind" that some servers
    // accept as anonymous — treat them as invalid up front.
    if username.trim().is_empty() || password.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    // Refuse cleartext transport unless explicitly opted in at build time.
    if !LDAP_SERVER_URI.starts_with("ldaps://") && !insecure_allowed() {
        eprintln!("auth: refusing insecure LDAP transport (set LDAP_ALLOW_INSECURE=true to allow)");
        return Err(AuthError::Config);
    }

    let settings = LdapConnSettings::new().set_conn_timeout(LDAP_TIMEOUT);

    let (conn, mut ldap) = LdapConnAsync::with_settings(settings, LDAP_SERVER_URI)
        .await
        .map_err(|e| {
            eprintln!("auth: LDAP connect failed: {e}");
            AuthError::Connection
        })?;

    ldap3::drive!(conn);

    let bind_dn = build_bind_dn(username);
    let result = ldap.simple_bind(&bind_dn, password).await.map_err(|e| {
        // A protocol-level error here is a transport/connection problem; a
        // wrong password comes back as a non-zero result code below.
        eprintln!("auth: bind transport error: {e}");
        AuthError::Connection
    })?;

    let _ = ldap.unbind().await;

    if result.rc != 0 {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(())
}

// ---- OS keychain (remember me) ------------------------------------------

#[derive(Serialize, Deserialize)]
struct StoredCredential {
    username: String,
    password: String,
}

fn keyring_entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
}

fn save_credential(username: &str, password: &str) {
    let blob = match serde_json::to_string(&StoredCredential {
        username: username.to_string(),
        password: password.to_string(),
    }) {
        Ok(b) => b,
        Err(_) => return,
    };
    match keyring_entry().and_then(|e| e.set_password(&blob)) {
        Ok(()) => {}
        Err(e) => eprintln!("auth: failed to save credential to keychain: {e}"),
    }
}

fn load_credential() -> Option<StoredCredential> {
    let entry = keyring_entry().ok()?;
    let blob = entry.get_password().ok()?;
    serde_json::from_str(&blob).ok()
}

fn delete_credential() {
    if let Ok(entry) = keyring_entry() {
        // Missing entry is fine; only log unexpected failures.
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => eprintln!("auth: failed to delete credential: {e}"),
        }
    }
}

// ---- Tauri commands ------------------------------------------------------

/// Result of the silent startup auto-login attempt.
#[derive(Serialize)]
pub struct AutoLoginResult {
    /// True when saved credentials bound successfully and the session started.
    authenticated: bool,
    /// Saved username, for prefilling the login form when not authenticated.
    username: Option<String>,
    /// Non-fatal message to surface (e.g. server unreachable, password stale).
    error: Option<String>,
}

/// Interactive login from the BIGSU form.
#[tauri::command]
pub async fn ldap_login(
    app: AppHandle,
    username: String,
    password: String,
    remember: bool,
) -> Result<(), String> {
    bind(&username, &password)
        .await
        .map_err(AuthError::message)?;

    if remember {
        save_credential(&username, &password);
    } else {
        delete_credential();
    }

    crate::start_session(&app);
    Ok(())
}

/// Silent login at startup using keychain-saved credentials.
#[tauri::command]
pub async fn auto_login(app: AppHandle) -> AutoLoginResult {
    let Some(cred) = load_credential() else {
        return AutoLoginResult {
            authenticated: false,
            username: None,
            error: None,
        };
    };

    match bind(&cred.username, &cred.password).await {
        Ok(()) => {
            crate::start_session(&app);
            AutoLoginResult {
                authenticated: true,
                username: Some(cred.username),
                error: None,
            }
        }
        // Saved password no longer valid (e.g. rotated) — forget it but keep
        // the username so the form can prefill.
        Err(AuthError::InvalidCredentials) => {
            delete_credential();
            AutoLoginResult {
                authenticated: false,
                username: Some(cred.username),
                error: Some("Saved password is no longer valid. Please sign in again.".to_string()),
            }
        }
        // Server unreachable (VPN down) — keep creds, let the user retry.
        Err(e) => AutoLoginResult {
            authenticated: false,
            username: Some(cred.username),
            error: Some(e.message()),
        },
    }
}

/// Forget saved credentials and return to the login screen.
#[tauri::command]
pub fn logout() -> Result<(), String> {
    delete_credential();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_bind_dn_replaces_user_placeholder() {
        // Template is baked at compile time; just verify the substitution shape
        // against whatever the build provided.
        let dn = build_bind_dn("alice");
        assert!(
            dn.contains("alice"),
            "expected username in bind DN, got: {dn}"
        );
        assert!(!dn.contains("{user}"), "placeholder must be replaced: {dn}");
    }

    #[test]
    fn build_bind_dn_escapes_dn_special_chars() {
        // DN-special characters must be escaped so a username cannot break the
        // RDN. ldap3::dn_escape uses RFC 4514 escaping (hex `\2c` or `\,`).
        let dn = build_bind_dn("a,b");
        assert!(!dn.contains("a,b"), "raw comma must not survive: {dn}");
        assert!(
            dn.contains("\\2c") || dn.contains("\\,"),
            "comma must be escaped: {dn}"
        );
    }

    #[tokio::test]
    async fn bind_rejects_empty_username() {
        assert_eq!(
            bind("", "secret").await,
            Err(AuthError::InvalidCredentials)
        );
    }

    #[tokio::test]
    async fn bind_rejects_empty_password() {
        assert_eq!(
            bind("alice", "").await,
            Err(AuthError::InvalidCredentials)
        );
    }

    #[test]
    fn error_messages_are_generic() {
        // Must not leak server/topology detail or distinguish "no such user".
        let invalid = AuthError::InvalidCredentials.message();
        assert!(invalid.to_lowercase().contains("invalid"));
        assert!(!invalid.contains(LDAP_SERVER_URI));
        assert!(!AuthError::Connection.message().contains(LDAP_SERVER_URI));
    }

    #[test]
    fn stored_credential_roundtrips() {
        let json = serde_json::to_string(&StoredCredential {
            username: "alice".into(),
            password: "p@ss".into(),
        })
        .unwrap();
        let back: StoredCredential = serde_json::from_str(&json).unwrap();
        assert_eq!(back.username, "alice");
        assert_eq!(back.password, "p@ss");
    }
}
