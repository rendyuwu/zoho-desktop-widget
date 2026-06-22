import { MetricCard, Badge } from "@gio/bigsu-ui";
import type { TicketPayload } from "../types";

interface CountGridProps {
  data: TicketPayload | null;
  loading: boolean;
}

interface CountConfig {
  label: string;
  period: string;
  value: number;
}

function getCountTone(count: number): {
  variant: "danger" | "warning";
  label: string;
} | null {
  if (count > 9) return { variant: "danger", label: "High" };
  if (count >= 6) return { variant: "warning", label: "Watch" };
  return null;
}

function CountGrid({ data, loading }: CountGridProps) {
  const totalTickets = data?.total_ticket ?? [];
  const onholdTickets = data?.onhold_ticket ?? [];

  const findTotal = (status: string) =>
    totalTickets.find((t) => t.status === status)?.total ?? 0;

  const findOnhold = (tag: string) =>
    onholdTickets.find((t) => t.tag === tag)?.total ?? 0;

  const cards: CountConfig[] = [
    { label: "GIO Open", period: "Current", value: findTotal("Open") },
    { label: "GIO On Progress", period: "Current", value: findTotal("On Progress") },
    { label: "GIO On Hold", period: "Current", value: findTotal("On Hold Tickets") },
    { label: "OnHold Abuse", period: "Current", value: findOnhold("abuse") },
    { label: "OnHold Incident", period: "Current", value: findOnhold("incident") },
    { label: "OnHold Sales", period: "Current", value: findOnhold("sales") },
  ];

  return (
    <section className="grid grid-cols-2 gap-2 px-3 py-2" aria-label="Ticket counts">
      {cards.map((card) => {
        const tone = getCountTone(card.value);
        return (
          <MetricCard
            key={card.label}
            label={card.label}
            value={
              <span className="flex items-center gap-1.5">
                {String(card.value)}
                {tone && (
                  <Badge variant={tone.variant}>
                    {tone.label}
                  </Badge>
                )}
              </span>
            }
            period={card.period}
            loading={loading}
          />
        );
      })}
    </section>
  );
}

export default CountGrid;
