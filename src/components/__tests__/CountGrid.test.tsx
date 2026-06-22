import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import CountGrid from "../CountGrid";
import type { TicketPayload } from "../../types";

const mockData: TicketPayload = {
  total_ticket: [
    { status: "Open", total: 12 },
    { status: "On Progress", total: 7 },
    { status: "On Hold Tickets", total: 3 },
  ],
  onhold_ticket: [
    { tag: "abuse", total: 10 },
    { tag: "incident", total: 5 },
    { tag: "sales", total: 0 },
  ],
  waiting_response: [],
};

describe("CountGrid", () => {
  it("renders all 6 MetricCards with label and period (V8)", () => {
    render(<CountGrid data={mockData} loading={false} />);

    const labels = [
      "GIO Open",
      "GIO On Progress",
      "GIO On Hold",
      "OnHold Abuse",
      "OnHold Incident",
      "OnHold Sales",
    ];

    for (const label of labels) {
      expect(screen.getByText(label)).toBeInTheDocument();
      expect(screen.getAllByText("Current").length).toBeGreaterThanOrEqual(6);
    }
  });

  it("maps total_ticket data correctly", () => {
    render(<CountGrid data={mockData} loading={false} />);

    expect(screen.getByText("12")).toBeInTheDocument();
    expect(screen.getByText("7")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("maps onhold_ticket data correctly", () => {
    render(<CountGrid data={mockData} loading={false} />);

    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("shows 0 for missing data", () => {
    const emptyData: TicketPayload = {
      total_ticket: [],
      onhold_ticket: [],
      waiting_response: [],
    };

    render(<CountGrid data={emptyData} loading={false} />);

    const zeros = screen.getAllByText("0");
    expect(zeros.length).toBe(6);
  });

  it("shows 0 for null data", () => {
    render(<CountGrid data={null} loading={false} />);

    const zeros = screen.getAllByText("0");
    expect(zeros.length).toBe(6);
  });

  it("shows danger Badge with text label for count > 9 (V10)", () => {
    render(<CountGrid data={mockData} loading={false} />);

    const highBadges = screen.getAllByText("High");
    expect(highBadges.length).toBe(2);
  });

  it("shows warning Badge with text label for count 6-9 (V10)", () => {
    render(<CountGrid data={mockData} loading={false} />);

    expect(screen.getByText("Watch")).toBeInTheDocument();
  });

  it("does not show Badge for count < 6", () => {
    render(<CountGrid data={mockData} loading={false} />);

    const badgedCards = screen.getAllByText(/High|Watch/);
    expect(badgedCards.length).toBe(3);
  });

  it("renders in loading state", () => {
    render(<CountGrid data={null} loading={true} />);

    const labels = [
      "GIO Open",
      "GIO On Progress",
      "GIO On Hold",
      "OnHold Abuse",
      "OnHold Incident",
      "OnHold Sales",
    ];

    for (const label of labels) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
  });
});
