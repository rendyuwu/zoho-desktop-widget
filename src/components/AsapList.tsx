import { Badge } from "@gio/bigsu-ui";
import TicketCard from "./TicketCard";
import type { WaitingResponse } from "../types";
import { classifyTicket } from "../constants";

interface AsapListProps {
  tickets: WaitingResponse[];
}

function AsapList({ tickets }: AsapListProps) {
  const now = Math.floor(Date.now() / 1000);
  const asapTickets = tickets.filter(
    (t) => classifyTicket(now - t.timestamp) === "asap",
  );

  return (
    <section className="flex flex-col gap-2" aria-label="ASAP tickets">
      <div className="flex items-center gap-2 px-1">
        <h2 className="text-sm font-semibold text-text-primary">ASAP</h2>
        {asapTickets.length > 0 && (
          <Badge variant="danger">{asapTickets.length}</Badge>
        )}
      </div>
      <ul className="flex flex-col gap-2 overflow-y-auto">
        {asapTickets.map((ticket) => (
          <li key={ticket.id_ticket}>
            <TicketCard ticket={ticket} category="asap" />
          </li>
        ))}
      </ul>
    </section>
  );
}

export default AsapList;
