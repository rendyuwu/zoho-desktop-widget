import { IconButton, Badge, bigsuToast } from "@gio/bigsu-ui";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type DisplayMode = "all" | "waiting" | "total";

interface WidgetHeaderProps {
  asapCount: number;
  mode: DisplayMode;
  onModeChange: (mode: DisplayMode) => void;
}

const MODE_LABEL: Record<DisplayMode, string> = {
  all: "All",
  waiting: "Waiting",
  total: "Total",
};

const MODE_ORDER: DisplayMode[] = ["all", "waiting", "total"];

function WidgetHeader({ asapCount, mode, onModeChange }: WidgetHeaderProps) {
  const handleReconnect = () => {
    invoke("reconnect_ws").then(() => {
      bigsuToast.info("Reconnecting…");
    }).catch(() => {
      bigsuToast.danger("Reconnect failed");
    });
  };

  const handleClose = () => {
    void getCurrentWindow().hide();
  };

  const cycleMode = () => {
    const idx = MODE_ORDER.indexOf(mode);
    const next = MODE_ORDER[(idx + 1) % MODE_ORDER.length];
    onModeChange(next);
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
        <button
          type="button"
          onClick={cycleMode}
          className="rounded px-2 py-0.5 text-xs font-medium text-text-secondary hover:bg-surface-hover hover:text-text-primary transition-colors"
          aria-label={`Switch view mode, current: ${MODE_LABEL[mode]}`}
        >
          {MODE_LABEL[mode]}
        </button>
        <IconButton
          icon="refresh"
          aria-label="Reconnect WebSocket"
          variant="ghost"
          size="sm"
          onClick={handleReconnect}
        />
        <IconButton
          icon="close"
          aria-label="Close widget to tray"
          variant="ghost"
          size="sm"
          onClick={handleClose}
        />
      </div>
    </header>
  );
}

export default WidgetHeader;
