import { IconButton, Badge, Button } from "@gio/bigsu-ui";
import { invoke } from "@tauri-apps/api/core";

interface WidgetHeaderProps {
  asapCount: number;
  onLogout: () => void;
}

function WidgetHeader({ asapCount, onLogout }: WidgetHeaderProps) {
  const handleReconnect = () => {
    invoke("reconnect_ws");
  };

  return (
    <header
      className="flex items-center justify-between border-b border-border-default px-3 py-2"
      data-tauri-drag-region
    >
      <div className="flex items-center gap-2">
        <h1 className="text-sm font-semibold text-text-primary">Zoho Tickets</h1>
        {asapCount > 0 && (
          <Badge variant="danger">{asapCount} ASAP</Badge>
        )}
      </div>
      <div className="flex items-center gap-1">
        <IconButton
          icon="refresh"
          aria-label="Reconnect WebSocket"
          variant="ghost"
          size="sm"
          onClick={handleReconnect}
        />
        <Button variant="ghost" size="sm" onClick={onLogout}>
          Sign out
        </Button>
      </div>
    </header>
  );
}

export default WidgetHeader;
