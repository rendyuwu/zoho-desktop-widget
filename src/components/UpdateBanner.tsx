import { useEffect, useState, useCallback } from "react";
import { Button, Toaster, bigsuToast } from "@gio/bigsu-ui";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface UpdateAvailableEvent {
  version: string;
  body?: string;
}

function UpdateBanner() {
  const [version, setVersion] = useState<string | null>(null);
  const [body, setBody] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<UpdateAvailableEvent>("update-available", (event) => {
      setVersion(event.payload.version);
      setBody(event.payload.body ?? null);
      setDismissed(false);
      bigsuToast.info(`Update available — v${event.payload.version}`, {
        description: event.payload.body ?? undefined,
      });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const handleInstall = useCallback(async () => {
    setInstalling(true);
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "install_update",
      );
      if (!result.success) {
        bigsuToast.danger("Update failed", {
          description: result.error ?? "Unknown error",
        });
        setInstalling(false);
      }
    } catch (e) {
      bigsuToast.danger("Update failed", {
        description: String(e),
      });
      setInstalling(false);
    }
  }, []);

  const handleLater = useCallback(() => {
    setDismissed(true);
  }, []);

  if (!version || dismissed) return null;

  return (
    <>
      <Toaster />
      <div
        className="flex items-center justify-between gap-2 border-b border-brand-soft bg-brand-soft px-3 py-2"
        role="status"
        aria-live="polite"
      >
        <div className="min-w-0 flex-1">
          <p className="text-xs font-semibold text-text-primary">
            Update available — v{version}
          </p>
          {body && (
            <p className="truncate text-xs text-text-secondary">{body}</p>
          )}
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <Button
            variant="primary"
            size="sm"
            loading={installing}
            onClick={handleInstall}
          >
            {installing ? "Installing…" : "Update & Restart"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleLater}
            disabled={installing}
          >
            Later
          </Button>
        </div>
      </div>
    </>
  );
}

export default UpdateBanner;
