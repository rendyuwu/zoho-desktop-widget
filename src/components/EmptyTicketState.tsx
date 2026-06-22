import { EmptyState as BigsuEmptyState } from "@gio/bigsu-ui";

function EmptyTicketState() {
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <BigsuEmptyState
        icon="requests"
        title="No tickets waiting"
        description="Tickets waiting for a response will appear here."
      />
    </div>
  );
}

export default EmptyTicketState;
