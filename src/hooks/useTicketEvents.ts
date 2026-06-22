import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { TicketPayload, TicketMoveEvent } from "../types";

export interface TicketMoveRecord {
  id_ticket: string;
  from: string;
  to: string;
  timestamp: number;
}

interface UseTicketEventsResult {
  data: TicketPayload | null;
  loading: boolean;
  error: boolean;
  moves: TicketMoveRecord[];
  tick: number;
}

function useTicketEvents(): UseTicketEventsResult {
  const [data, setData] = useState<TicketPayload | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [moves, setMoves] = useState<TicketMoveRecord[]>([]);
  const [tick, setTick] = useState(0);
  const hasData = useRef(false);

  useEffect(() => {
    let cancelled = false;
    const unlisteners: (() => void)[] = [];

    const timeout = setTimeout(() => {
      if (cancelled) return;
      if (!hasData.current) {
        setError(true);
        setLoading(false);
      }
    }, 15000);

    (async () => {
      try {
        const cached = await invoke<TicketPayload | null>("get_current_tickets");
        if (cancelled) return;
        if (cached) {
          setData(cached);
          hasData.current = true;
        }
        setLoading(false);
      } catch {
        if (!cancelled) setLoading(false);
      }

      const unlistenData = await listen<TicketPayload>("ticket-data", (event) => {
        if (cancelled) return;
        setData(event.payload);
        hasData.current = true;
        setError(false);
        setLoading(false);
      });
      if (cancelled) {
        unlistenData();
        return;
      }
      unlisteners.push(unlistenData);

      const unlistenMove = await listen<TicketMoveEvent>("ticket-move", (event) => {
        if (cancelled) return;
        setMoves((prev) => [
          ...prev.slice(-49),
          {
            id_ticket: event.payload.id_ticket,
            from: event.payload.from,
            to: event.payload.to,
            timestamp: Date.now(),
          },
        ]);
      });
      if (cancelled) {
        unlistenMove();
        return;
      }
      unlisteners.push(unlistenMove);
    })();

    return () => {
      cancelled = true;
      clearTimeout(timeout);
      for (const fn of unlisteners) fn();
    };
  }, []);

  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 3000);
    return () => clearInterval(interval);
  }, []);

  return { data, loading, error, moves, tick };
}

export default useTicketEvents;
