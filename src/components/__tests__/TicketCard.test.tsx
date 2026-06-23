import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";

vi.mock("@gio/bigsu-ui", () => ({
  Badge: ({
    children,
    variant,
  }: {
    children: React.ReactNode;
    variant: string;
  }) => <span data-variant={variant}>{children}</span>,
}));

import TicketCard from "../TicketCard";
import type { WaitingResponse } from "../../types";

const FIXED_MS = 1719600000000;
const FIXED_S = 1719600000;

const baseTicket: WaitingResponse = {
  id_ticket: "42001",
  department: "GIO Support",
  status_ticket: "Open",
  customer_response_time: "2025-01-01T00:00:00Z",
  subject: "Server latency <b>spike</b> in EU region",
  timestamp: FIXED_S - 420,
};

describe("TicketCard", () => {
  beforeEach(() => {
    vi.spyOn(Date, "now").mockReturnValue(FIXED_MS);
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders department badge and urgency badge (V9, V10)", () => {
    render(<TicketCard ticket={baseTicket} category="warning" />);

    const dept = screen.getByText("GIO Support");
    expect(dept).toBeInTheDocument();
    expect(dept.closest("span")?.dataset.variant).toBe("neutral");

    const urgency = screen.getByText("Warning");
    expect(urgency).toBeInTheDocument();
    expect(urgency.closest("span")?.dataset.variant).toBe("warning");
  });

  it("shows ticket ID with # prefix in mono font", () => {
    render(<TicketCard ticket={baseTicket} category="new" />);
    expect(screen.getByText("#42001")).toBeInTheDocument();
  });

  it("shows formatted elapsed time", () => {
    render(<TicketCard ticket={baseTicket} category="new" />);
    expect(screen.getByText("7m")).toBeInTheDocument();
  });

  it("strips HTML from subject", () => {
    render(<TicketCard ticket={baseTicket} category="asap" />);
    expect(screen.getByText("Server latency spike in EU region")).toBeInTheDocument();
    expect(screen.queryByText(/<b>/)).not.toBeInTheDocument();
  });

  it("renders danger badge for asap tickets", () => {
    render(<TicketCard ticket={baseTicket} category="asap" />);
    const badge = screen.getByText("ASAP");
    expect(badge.closest("span")?.dataset.variant).toBe("danger");
  });

  it("renders info badge for new tickets", () => {
    render(<TicketCard ticket={baseTicket} category="new" />);
    const badge = screen.getByText("New");
    expect(badge.closest("span")?.dataset.variant).toBe("info");
  });

  it("handles plain-text subjects without HTML", () => {
    const ticket = { ...baseTicket, subject: "Plain text subject" };
    render(<TicketCard ticket={ticket} category="new" />);
    expect(screen.getByText("Plain text subject")).toBeInTheDocument();
  });
});
