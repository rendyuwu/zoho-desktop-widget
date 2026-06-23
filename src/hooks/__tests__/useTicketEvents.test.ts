import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";

const { mockInvoke, mockListen } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
  mockListen: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
}));

import useTicketEvents from "../useTicketEvents";
import type { TicketPayload } from "../../types";

const mockPayload: TicketPayload = {
  total_ticket: [{ status: "Open", total: 5 }],
  onhold_ticket: [{ tag: "abuse", total: 2 }],
  waiting_response: [],
};

describe("useTicketEvents", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts in loading state", () => {
    mockInvoke.mockResolvedValue(null);
    mockListen.mockResolvedValue(vi.fn());

    const { result } = renderHook(() => useTicketEvents());
    expect(result.current.loading).toBe(true);
    expect(result.current.data).toBeNull();
    expect(result.current.error).toBe(false);
  });

  it("fetches cached data on mount (get_current_tickets)", async () => {
    mockInvoke.mockResolvedValue(mockPayload);
    mockListen.mockResolvedValue(vi.fn());

    const { result } = renderHook(() => useTicketEvents());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
    expect(result.current.data).toEqual(mockPayload);
    expect(result.current.error).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith("get_current_tickets");
  });

  it("handles invoke rejection gracefully — clears loading", async () => {
    mockInvoke.mockRejectedValue(new Error("failed"));
    mockListen.mockResolvedValue(vi.fn());

    const { result } = renderHook(() => useTicketEvents());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
    expect(result.current.error).toBe(false);
  });

  it("listens for ticket-data events", async () => {
    mockInvoke.mockResolvedValue(null);
    let capturedHandler: (event: { payload: TicketPayload }) => void = () => {};

    mockListen.mockImplementation(
      (
        eventName: string,
        handler: (event: { payload: TicketPayload }) => void,
      ) => {
        if (eventName === "ticket-data") {
          capturedHandler = handler;
          return Promise.resolve(vi.fn());
        }
        return Promise.resolve(vi.fn());
      },
    );

    const { result } = renderHook(() => useTicketEvents());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    act(() => {
      capturedHandler({ payload: mockPayload });
    });

    expect(result.current.data).toEqual(mockPayload);
    expect(result.current.error).toBe(false);
  });

  it("sets error after 15s timeout with no data", () => {
    vi.useFakeTimers();
    mockInvoke.mockResolvedValue(null);
    mockListen.mockResolvedValue(vi.fn());

    const { result } = renderHook(() => useTicketEvents());

    expect(result.current.loading).toBe(true);
    expect(result.current.error).toBe(false);

    act(() => {
      vi.advanceTimersByTime(15000);
    });

    expect(result.current.error).toBe(true);
    expect(result.current.loading).toBe(false);

    vi.useRealTimers();
  });

  it("tick increments every 3s", () => {
    vi.useFakeTimers();
    mockInvoke.mockResolvedValue(null);
    mockListen.mockResolvedValue(vi.fn());

    const { result } = renderHook(() => useTicketEvents());

    expect(result.current.tick).toBe(0);

    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current.tick).toBe(1);

    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current.tick).toBe(2);

    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current.tick).toBe(3);

    vi.useRealTimers();
  });

  it("cleans up on unmount", () => {
    mockInvoke.mockResolvedValue(null);
    const mockUnlisten = vi.fn();
    mockListen.mockResolvedValue(mockUnlisten);

    const { unmount } = renderHook(() => useTicketEvents());
    expect(() => unmount()).not.toThrow();
  });
});
