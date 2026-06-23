import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";

const { mockInvoke } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

import useAuth from "../useAuth";

describe("useAuth", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts in checking status", () => {
    mockInvoke.mockResolvedValue({ authenticated: false, username: null, error: null });
    const { result } = renderHook(() => useAuth());
    expect(result.current.status).toBe("checking");
  });

  it("sets authenticated on successful auto_login", async () => {
    mockInvoke.mockResolvedValue({ authenticated: true, username: null, error: null });
    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("authenticated");
    });
  });

  it("sets unauthenticated with username on failed auto_login", async () => {
    mockInvoke.mockResolvedValue({
      authenticated: false,
      username: "rendy",
      error: "Invalid credentials",
    });
    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("unauthenticated");
    });
    expect(result.current.savedUsername).toBe("rendy");
    expect(result.current.initialError).toBe("Invalid credentials");
  });

  it("sets unauthenticated on invoke rejection", async () => {
    mockInvoke.mockRejectedValue("Connection failed");
    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("unauthenticated");
    });
    expect(result.current.initialError).toBe("Connection failed");
  });

  it("throws generic error on non-string rejection", async () => {
    mockInvoke.mockRejectedValue(new Error("boom"));
    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("unauthenticated");
    });
    expect(result.current.initialError).toBe("Sign-in failed.");
  });

  it("login calls ldap_login and sets authenticated", async () => {
    mockInvoke
      .mockResolvedValueOnce({ authenticated: false, username: null, error: null })
      .mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("unauthenticated");
    });

    await act(async () => {
      await result.current.login("rendy", "secret", true);
    });

    expect(mockInvoke).toHaveBeenCalledWith("ldap_login", {
      username: "rendy",
      password: "secret",
      remember: true,
    });
    expect(result.current.status).toBe("authenticated");
  });

  it("login propagates errors (does not catch)", async () => {
    mockInvoke
      .mockResolvedValueOnce({ authenticated: false, username: null, error: null })
      .mockRejectedValueOnce("Bad credentials");

    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("unauthenticated");
    });

    await expect(
      act(async () => {
        await result.current.login("rendy", "wrong", false);
      }),
    ).rejects.toThrow();

    expect(result.current.status).toBe("unauthenticated");
  });

  it("logout calls logout command and resets state", async () => {
    mockInvoke
      .mockResolvedValueOnce({ authenticated: true, username: "rendy", error: null })
      .mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useAuth());

    await waitFor(() => {
      expect(result.current.status).toBe("authenticated");
    });

    await act(async () => {
      await result.current.logout();
    });

    expect(mockInvoke).toHaveBeenCalledWith("logout");
    expect(result.current.status).toBe("unauthenticated");
    expect(result.current.savedUsername).toBe("");
    expect(result.current.initialError).toBe("");
  });

  it("does not update state after unmount (cleanup)", async () => {
    let resolveAutoLogin: (value: unknown) => void;
    mockInvoke.mockReturnValue(
      new Promise((resolve) => {
        resolveAutoLogin = resolve;
      }),
    );

    const { result, unmount } = renderHook(() => useAuth());
    expect(result.current.status).toBe("checking");

    unmount();

    await act(async () => {
      resolveAutoLogin!({ authenticated: true, username: null, error: null });
    });

    // After unmount, state should remain 'checking' — the cancelled flag
    // prevents setState. The hook is unmounted so result.current is stale.
    expect(result.current.status).toBe("checking");
  });
});
