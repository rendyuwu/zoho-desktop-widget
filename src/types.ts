export interface TotalTicket {
  status: string;
  total: number;
}

export interface OnholdTicket {
  tag: string;
  total: number;
}

export interface WaitingResponse {
  id_ticket: string;
  department: string;
  status_ticket: string;
  customer_response_time: string;
  subject: string;
  timestamp: number;
}

export interface TicketPayload {
  total_ticket: TotalTicket[];
  onhold_ticket: OnholdTicket[];
  waiting_response: WaitingResponse[];
}

export type TicketCategory = "new" | "warning" | "asap";

export interface TicketMoveEvent {
  id_ticket: string;
  from: TicketCategory;
  to: TicketCategory;
}
