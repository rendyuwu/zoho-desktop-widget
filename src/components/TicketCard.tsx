import { Badge } from "@gio/bigsu-ui";
import type { WaitingResponse, TicketCategory } from "../types";
import { formatElapsed, stripHtml } from "../constants";

interface TicketCardProps {
  ticket: WaitingResponse;
  category: TicketCategory;
}

const URGENCY_BADGE: Record<
  TicketCategory,
  { variant: "danger" | "warning" | "info"; label: string }
> = {
  asap: { variant: "danger", label: "ASAP" },
  warning: { variant: "warning", label: "Warning" },
  new: { variant: "info", label: "New" },
};

function TicketCard({ ticket, category }: TicketCardProps) {
  const now = Math.floor(Date.now() / 1000);
  const elapsed = now - ticket.timestamp;
  const urgency = URGENCY_BADGE[category];
  const subject = stripHtml(ticket.subject);

  return (
    <div className="rounded-lg border border-border-default bg-surface p-2.5 shadow-sm">
      <div className="flex items-center justify-between gap-2">
        <Badge variant="neutral">{ticket.department}</Badge>
        <Badge variant={urgency.variant}>{urgency.label}</Badge>
      </div>
      <div className="mt-1.5 flex items-center justify-between gap-2">
        <span className="font-mono text-xs text-text-muted">
          #{ticket.id_ticket}
        </span>
        <span className="text-xs text-text-secondary">
          {formatElapsed(elapsed)}
        </span>
      </div>
      <p className="mt-1.5 line-clamp-2 text-sm text-text-primary">
        {subject}
      </p>
    </div>
  );
}

export default TicketCard;
