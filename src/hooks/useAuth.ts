import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export type AuthStatus = "checking" | "unauthenticated" | "authenticated";

interface AutoLoginResult {
  authenticated: boolean;
  username: string | null;
  error: string | null;
}

interface UseAuthResult {
  status: AuthStatus;
  savedUsername: string;
  initialError: string;
  login: (username: string, password: string, remember: boolean) => Promise<void>;
  logout: () => Promise<void>;
}

/**
 * Auth gate state machine. On mount it asks the Rust backend to silently
 * auto-login from keychain-saved credentials; until that resolves the app
 * shows a splash. The widget only mounts once `status === "authenticated"`.
 */
export default function useAuth(): UseAuthResult {
  const [status, setStatus] = useState<AuthStatus>("checking");
  const [savedUsername, setSavedUsername] = useState("");
  const [initialError, setInitialError] = useState("");

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await invoke<AutoLoginResult>("auto_login");
        if (cancelled) return;
        if (res.authenticated) {
          setStatus("authenticated");
          return;
        }
        if (res.username) setSavedUsername(res.username);
        if (res.error) setInitialError(res.error);
        setStatus("unauthenticated");
      } catch (e) {
        if (cancelled) return;
        setInitialError(typeof e === "string" ? e : "Sign-in failed.");
        setStatus("unauthenticated");
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const login = useCallback(
    async (username: string, password: string, remember: boolean) => {
      // Throws on failure with the backend's safe error string — LoginScreen
      // catches it and shows it inline.
      await invoke("ldap_login", { username, password, remember });
      setStatus("authenticated");
    },
    [],
  );

  const logout = useCallback(async () => {
    await invoke("logout");
    setSavedUsername("");
    setInitialError("");
    setStatus("unauthenticated");
  }, []);

  return { status, savedUsername, initialError, login, logout };
}
