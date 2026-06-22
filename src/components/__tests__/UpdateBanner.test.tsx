import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import UpdateBanner from "../UpdateBanner";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;
const mockListen = listen as ReturnType<typeof vi.fn>;

describe("UpdateBanner", () => {
  let unlistenFn: ReturnType<typeof vi.fn>;
  let listenCallback: ((event: { payload: { version: string; body?: string } }) => void) | null;

  beforeEach(() => {
    vi.clearAllMocks();
    unlistenFn = vi.fn();
    listenCallback = null;
    mockListen.mockImplementation(async (event: string, cb: (e: { payload: { version: string; body?: string } }) => void) => {
      if (event === "update-available") {
        listenCallback = cb;
      }
      return unlistenFn;
    });
    mockInvoke.mockResolvedValue({ success: true });
  });

  it("renders nothing initially — no update event received (V14)", () => {
    render(<UpdateBanner />);
    expect(screen.queryByText(/Update available/i)).not.toBeInTheDocument();
  });

  it("renders inline banner with version when update-available event fires (V14)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText(/Update available — v0\.2\.0/i)).toBeInTheDocument();
    });
  });

  it("shows body text when event includes body (V14)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0", body: "Bug fixes and improvements" } });

    await waitFor(() => {
      expect(screen.getByText("Bug fixes and improvements")).toBeInTheDocument();
    });
  });

  it("Later button dismisses banner (V14 — can defer)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Later")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Later"));

    await waitFor(() => {
      expect(screen.queryByText(/Update available/i)).not.toBeInTheDocument();
    });
  });

  it("Update & Restart button calls install_update (V14)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("install_update");
    });
  });

  it("shows loading state on install button (V14)", async () => {
    let resolveInstall: (value: { success: boolean }) => void = () => {};
    mockInvoke.mockReturnValue(
      new Promise((resolve) => {
        resolveInstall = resolve;
      }),
    );

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(screen.getByText("Installing…")).toBeInTheDocument();
    });

    resolveInstall({ success: true });
  });

  it("does not crash on install failure — shows error state, no throw (V15)", async () => {
    mockInvoke.mockResolvedValue({ success: false, error: "Network error" });

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });
  });

  it("does not crash on invoke rejection (V15)", async () => {
    mockInvoke.mockRejectedValue(new Error("invoke failed"));

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });
  });

  it("Later button disabled while installing (V14)", async () => {
    let resolveInstall: (value: { success: boolean; error?: string }) => void = () => {};
    mockInvoke.mockReturnValue(
      new Promise<{ success: boolean; error?: string }>((resolve) => {
        resolveInstall = resolve;
      }),
    );

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(screen.getByText("Installing…")).toBeInTheDocument();
    });

    expect(screen.getByText("Later")).toBeDisabled();

    resolveInstall({ success: false, error: "test" });
  });

  it("banner has role=status and aria-live=polite (V14 — user must see)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      const banner = screen.getByRole("status");
      expect(banner).toHaveAttribute("aria-live", "polite");
    });
  });

  it("cleans up listener on unmount", async () => {
    const { unmount } = render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    unmount();

    expect(unlistenFn).toHaveBeenCalled();
  });
});
