import type { TicketCategory } from "./types";

export const WARNING_THRESHOLD = 600;
export const ASAP_THRESHOLD = 900;

export function classifyTicket(elapsed: number): TicketCategory {
  if (elapsed >= ASAP_THRESHOLD) return "asap";
  if (elapsed >= WARNING_THRESHOLD) return "warning";
  return "new";
}

export function formatElapsed(elapsed: number): string {
  if (elapsed < 60) return `${elapsed}s`;
  const m = Math.floor(elapsed / 60);
  const s = elapsed % 60;
  if (m < 60) return s > 0 ? `${m}m ${s}s` : `${m}m`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  return rm > 0 ? `${h}h ${rm}m` : `${h}h`;
}

export function stripHtml(s: string): string {
  return s.replace(/<[^>]*>/g, "");
}
