import { ErrorState } from "@gio/bigsu-ui";
import { invoke } from "@tauri-apps/api/core";

interface ErrorTicketStateProps {
  onRetry?: () => void;
}

function ErrorTicketState({ onRetry }: ErrorTicketStateProps) {
  const handleRetry = () => {
    invoke("reconnect_ws");
    onRetry?.();
  };

  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <ErrorState
        title="Could not load tickets"
        description="WebSocket connection timed out. Try reconnecting."
        onRetry={handleRetry}
      />
    </div>
  );
}

export default ErrorTicketState;
