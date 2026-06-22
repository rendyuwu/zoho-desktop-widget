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
        <div
          role="tablist"
          aria-label="View mode"
          className="flex items-center rounded-md bg-surface p-0.5"
        >
          {MODE_ORDER.map((m) => (
            <button
              key={m}
              type="button"
              role="tab"
              aria-selected={mode === m}
              onClick={() => onModeChange(m)}
              className={
                "rounded px-2 py-0.5 text-xs font-medium transition-colors " +
                (mode === m
                  ? "bg-brand-soft text-text-primary"
                  : "text-text-muted hover:text-text-primary")
              }
            >
              {MODE_LABEL[m]}
            </button>
          ))}
        </div>
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
