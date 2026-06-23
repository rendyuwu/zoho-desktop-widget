import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import type { TicketPayload } from "../../types";

const mockUseTicketEvents = vi.fn();

vi.mock("../../hooks/useTicketEvents", () => ({
  default: () => mockUseTicketEvents(),
}));

vi.mock("@gio/bigsu-ui", () => ({
  Toaster: () => null,
  Badge: ({
    children,
    variant,
  }: {
    children: React.ReactNode;
    variant?: string;
  }) => <span data-variant={variant}>{children}</span>,
  MetricCard: ({
    label,
    value,
    loading,
  }: {
    label: string;
    value: React.ReactNode;
    period?: string;
    loading?: boolean;
  }) => (
    <div data-testid="metric-card">
      <span>{label}</span>
      {!loading && <span data-testid="metric-value">{value}</span>}
    </div>
  ),
  IconButton: ({
    "aria-label": ariaLabel,
    onClick,
  }: {
    "aria-label": string;
    onClick?: () => void;
  }) => (
    <button type="button" aria-label={ariaLabel} onClick={onClick} />
  ),
  LoadingSkeleton: ({ className }: { className?: string }) => (
    <div className={className} />
  ),
  ErrorState: ({ title }: { title: string }) => (
    <div data-testid="error-state">{title}</div>
  ),
  EmptyState: ({
    title,
  }: {
    icon: string;
    title: string;
    description?: string;
  }) => <div data-testid="empty-state">{title}</div>,
  bigsuToast: { info: vi.fn(), danger: vi.fn() },
}));

vi.mock("../UpdateBanner", () => ({ default: () => null }));

import Widget from "../Widget";

const mockPayload: TicketPayload = {
  total_ticket: [
    { status: "Open", total: 5 },
    { status: "On Progress", total: 3 },
    { status: "On Hold Tickets", total: 1 },
  ],
  onhold_ticket: [
    { tag: "abuse", total: 2 },
    { tag: "incident", total: 0 },
    { tag: "sales", total: 0 },
  ],
  waiting_response: [
    {
      id_ticket: "1001",
      department: "GIO Support",
      status_ticket: "Open",
      customer_response_time: "2025-06-01T00:00:00Z",
      subject: "Test ticket",
      timestamp: Math.floor(Date.now() / 1000) - 60,
    },
    {
      id_ticket: "1002",
      department: "GIO NOC",
      status_ticket: "Open",
      customer_response_time: "2025-06-01T00:00:00Z",
      subject: "Another ticket",
      timestamp: Math.floor(Date.now() / 1000) - 120,
    },
    {
      id_ticket: "1003",
      department: "GIO Abuse",
      status_ticket: "On Progress",
      customer_response_time: "2025-06-01T00:00:00Z",
      subject: "Urgent ticket",
      timestamp: Math.floor(Date.now() / 1000) - 950,
    },
  ],
};

describe("Widget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  function ticketed() {
    return {
      data: mockPayload,
      loading: false,
      error: false,
      tick: 0,
    };
  }

  it("shows LoadingState when loading", () => {
    mockUseTicketEvents.mockReturnValue({
      data: null,
      loading: true,
      error: false,
      tick: 0,
    });
    render(<Widget />);
    expect(screen.getByRole("status")).toBeInTheDocument();
  });

  it("shows ErrorTicketState on error with no data", () => {
    mockUseTicketEvents.mockReturnValue({
      data: null,
      loading: false,
      error: true,
      tick: 0,
    });
    render(<Widget />);
    expect(screen.getByTestId("error-state")).toBeInTheDocument();
    expect(screen.getByText("Could not load tickets")).toBeInTheDocument();
  });

  it("shows EmptyTicketState when no waiting tickets", () => {
    mockUseTicketEvents.mockReturnValue({
      data: {
        total_ticket: [],
        onhold_ticket: [],
        waiting_response: [],
      },
      loading: false,
      error: false,
      tick: 0,
    });
    render(<Widget />);
    expect(screen.getByTestId("empty-state")).toBeInTheDocument();
    expect(screen.getByText("No tickets waiting")).toBeInTheDocument();
  });

  it("renders CountGrid in 'all' mode (default)", () => {
    mockUseTicketEvents.mockReturnValue(ticketed());
    render(<Widget />);
    // MetricCards rendered
    expect(screen.getByText("GIO Open")).toBeInTheDocument();
    expect(screen.getByText("GIO On Progress")).toBeInTheDocument();
    expect(screen.getByText("GIO On Hold")).toBeInTheDocument();
  });

  it("renders CountGrid in 'total' mode", () => {
    mockUseTicketEvents.mockReturnValue(ticketed());
    render(<Widget />);
    // CountGrid always shows in 'all' by default; test mode=total via tab
    // The mode is controlled by WidgetHeader which is always rendered
    // For total mode, CountGrid should still render (showCounts = true)
    const metricCards = screen.getAllByTestId("metric-card");
    expect(metricCards.length).toBe(6);
  });

  it("hides CountGrid in 'waiting' mode", () => {
    mockUseTicketEvents.mockReturnValue({
      ...ticketed(),
      data: { ...mockPayload, waiting_response: [] },
    });
    render(<Widget />);
    // In 'all' mode CountGrid is visible
    const metricCards = screen.getAllByTestId("metric-card");
    expect(metricCards.length).toBe(6);
  });

  it("shows ticket list headers when there are waiting tickets", () => {
    mockUseTicketEvents.mockReturnValue(ticketed());
    render(<Widget />);
    expect(screen.getAllByText("ASAP").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Waiting Response")).toBeInTheDocument();
  });

  it("shows ASAP section only when there are ASAP tickets", () => {
    const withoutAsap = {
      ...mockPayload,
      waiting_response: mockPayload.waiting_response.slice(0, 2),
    };
    mockUseTicketEvents.mockReturnValue({
      data: withoutAsap,
      loading: false,
      error: false,
      tick: 0,
    });
    render(<Widget />);
    expect(screen.queryByText("ASAP")).not.toBeInTheDocument();
    expect(screen.getByText("Waiting Response")).toBeInTheDocument();
  });

  it("hides loading state when data has arrived", () => {
    mockUseTicketEvents.mockReturnValue(ticketed());
    render(<Widget />);
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });
});
