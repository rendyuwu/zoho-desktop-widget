import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

const { mockBigsuToast } = vi.hoisted(() => ({
  mockBigsuToast: {
    info: vi.fn(),
    danger: vi.fn(),
    success: vi.fn(),
    warning: vi.fn(),
    dismiss: vi.fn(),
  },
}));

vi.mock("@gio/bigsu-ui", () => ({
  Button: ({ children, onClick, loading, disabled, ...props }: {
    children: React.ReactNode;
    onClick?: () => void;
    loading?: boolean;
    disabled?: boolean;
    variant?: string;
    size?: string;
  }) => (
    <button onClick={onClick} disabled={loading || disabled} {...props}>
      {children}
    </button>
  ),
  Toaster: () => null,
  bigsuToast: mockBigsuToast,
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

  it("fires bigsuToast.info when update-available event fires (V14 — user must see toast)", async () => {
    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0", body: "Bug fixes" } });

    await waitFor(() => {
      expect(mockBigsuToast.info).toHaveBeenCalledWith(
        "Update available — v0.2.0",
        { description: "Bug fixes" },
      );
    });
  });

  it("fires bigsuToast.danger on install failure (V14 — error feedback)", async () => {
    mockInvoke.mockResolvedValue({ success: false, error: "Network error" });

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(mockBigsuToast.danger).toHaveBeenCalledWith(
        "Could not install update",
        { description: "Network error" },
      );
    });
  });

  it("fires bigsuToast.danger on invoke rejection (V15)", async () => {
    mockInvoke.mockRejectedValue(new Error("invoke failed"));

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(mockBigsuToast.danger).toHaveBeenCalledWith(
        "Could not install update",
        { description: "invoke failed" },
      );
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

  it("re-shows banner when update-available fires after dismissal", async () => {
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

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText(/Update available/i)).toBeInTheDocument();
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

  it("resets installing to false on install failure (V15)", async () => {
    mockInvoke.mockResolvedValue({ success: false, error: "test" });

    render(<UpdateBanner />);
    await waitFor(() => expect(mockListen).toHaveBeenCalled());

    listenCallback?.({ payload: { version: "0.2.0" } });

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Update & Restart"));

    await waitFor(() => {
      expect(screen.getByText("Update & Restart")).toBeInTheDocument();
      expect(screen.queryByText("Installing…")).not.toBeInTheDocument();
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

  it("cleans up listener even if listen() resolves after unmount (StrictMode)", async () => {
    let resolveListen: (fn: () => void) => void = () => {};
    mockListen.mockImplementation(
      async (_event: string, _cb: (e: { payload: { version: string } }) => void) => {
        return new Promise<() => void>((resolve) => {
          resolveListen = resolve;
        });
      },
    );

    const { unmount } = render(<UpdateBanner />);

    unmount();

    const capturedUnlisten = vi.fn();
    resolveListen(capturedUnlisten);

    await waitFor(() => {
      expect(capturedUnlisten).toHaveBeenCalled();
    });
  });
});
