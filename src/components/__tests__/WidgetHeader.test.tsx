import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

const { mockInvoke, mockHide } = vi.hoisted(() => ({
  mockInvoke: vi.fn().mockResolvedValue(undefined),
  mockHide: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ hide: mockHide }),
}));

vi.mock("@gio/bigsu-ui", () => ({
  IconButton: ({
    "aria-label": ariaLabel,
    onClick,
  }: {
    "aria-label": string;
    onClick?: () => void;
    icon?: string;
    variant?: string;
    size?: string;
  }) => (
    <button type="button" aria-label={ariaLabel} onClick={onClick} />
  ),
  Badge: ({
    children,
    variant,
  }: {
    children: React.ReactNode;
    variant?: string;
  }) => <span data-variant={variant}>{children}</span>,
  bigsuToast: { info: vi.fn(), danger: vi.fn() },
}));

import WidgetHeader from "../WidgetHeader";

describe("WidgetHeader", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders title", () => {
    render(<WidgetHeader asapCount={0} mode="all" onModeChange={vi.fn()} />);
    expect(screen.getByText("Zoho Tickets")).toBeInTheDocument();
  });

  it("shows ASAP badge when asapCount > 0 (V10)", () => {
    render(<WidgetHeader asapCount={3} mode="all" onModeChange={vi.fn()} />);
    const badge = screen.getByText("3 ASAP");
    expect(badge).toBeInTheDocument();
    expect(badge.closest("span")?.dataset.variant).toBe("danger");
  });

  it("hides ASAP badge when asapCount is 0", () => {
    render(<WidgetHeader asapCount={0} mode="all" onModeChange={vi.fn()} />);
    expect(screen.queryByText(/ASAP/)).not.toBeInTheDocument();
  });

  it("renders three mode tabs with correct aria-selected", () => {
    render(<WidgetHeader asapCount={0} mode="waiting" onModeChange={vi.fn()} />);

    const allTab = screen.getByRole("tab", { name: "All" });
    const waitingTab = screen.getByRole("tab", { name: "Waiting" });
    const totalTab = screen.getByRole("tab", { name: "Total" });

    expect(allTab).toBeInTheDocument();
    expect(waitingTab).toBeInTheDocument();
    expect(totalTab).toBeInTheDocument();

    expect(allTab.getAttribute("aria-selected")).toBe("false");
    expect(waitingTab.getAttribute("aria-selected")).toBe("true");
    expect(totalTab.getAttribute("aria-selected")).toBe("false");
  });

  it("calls onModeChange when tab is clicked", () => {
    const onModeChange = vi.fn();
    render(<WidgetHeader asapCount={0} mode="all" onModeChange={onModeChange} />);

    fireEvent.click(screen.getByRole("tab", { name: "Waiting" }));
    expect(onModeChange).toHaveBeenCalledWith("waiting");

    fireEvent.click(screen.getByRole("tab", { name: "Total" }));
    expect(onModeChange).toHaveBeenCalledWith("total");
  });

  it("reconnect button invokes reconnect_ws", () => {
    render(<WidgetHeader asapCount={0} mode="all" onModeChange={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Reconnect WebSocket" }));
    expect(mockInvoke).toHaveBeenCalledWith("reconnect_ws");
  });

  it("close button hides the window", () => {
    render(<WidgetHeader asapCount={0} mode="all" onModeChange={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Close widget to tray" }));
    expect(mockHide).toHaveBeenCalled();
  });
});
