import { Badge } from "@gio/bigsu-ui";
import TicketCard from "./TicketCard";
import type { WaitingResponse } from "../types";
import { classifyTicket } from "../constants";

interface WaitingListProps {
  tickets: WaitingResponse[];
}

function WaitingList({ tickets }: WaitingListProps) {
  const now = Math.floor(Date.now() / 1000);

  const warningTickets = tickets.filter(
    (t) => classifyTicket(now - t.timestamp) === "warning",
  );

  const newTickets = tickets.filter(
    (t) => classifyTicket(now - t.timestamp) === "new",
  );

  return (
    <section className="flex flex-col gap-2" aria-label="Waiting response tickets">
      <div className="flex items-center gap-2 px-1">
        <h2 className="text-sm font-semibold text-text-primary">
          Waiting Response
        </h2>
        <span className="text-xs text-text-muted">
          {warningTickets.length + newTickets.length}
        </span>
      </div>

      {warningTickets.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <div className="flex items-center gap-2 px-1">
            <Badge variant="warning">Warning</Badge>
            <span className="text-xs text-text-muted">10–15 min</span>
          </div>
          <ul className="flex flex-col gap-2 overflow-y-auto">
            {warningTickets.map((ticket) => (
              <li key={ticket.id_ticket}>
                <TicketCard ticket={ticket} category="warning" />
              </li>
            ))}
          </ul>
        </div>
      )}

      {newTickets.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <div className="flex items-center gap-2 px-1">
            <Badge variant="info">New</Badge>
            <span className="text-xs text-text-muted">&lt; 10 min</span>
          </div>
          <ul className="flex flex-col gap-2 overflow-y-auto">
            {newTickets.map((ticket) => (
              <li key={ticket.id_ticket}>
                <TicketCard ticket={ticket} category="new" />
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}

export default WaitingList;
