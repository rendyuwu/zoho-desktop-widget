import { useEffect, useState, useCallback } from "react";
import { Button, bigsuToast } from "@gio/bigsu-ui";
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
    let cancelled = false;

    listen<UpdateAvailableEvent>("update-available", (event) => {
      if (cancelled) return;
      setVersion(event.payload.version);
      setBody(event.payload.body ?? null);
      setDismissed(false);
      bigsuToast.info(`Update available — v${event.payload.version}`, {
        description: event.payload.body ?? undefined,
      });
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
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
        bigsuToast.danger("Could not install update", {
          description: result.error ?? "Unknown error",
        });
        setInstalling(false);
      }
    } catch (e) {
      bigsuToast.danger("Could not install update", {
        description: e instanceof Error ? e.message : String(e),
      });
      setInstalling(false);
    }
  }, []);

  const handleLater = useCallback(() => {
    setDismissed(true);
  }, []);

  if (!version || dismissed) return null;

  return (
    <div
      className="flex items-center justify-between gap-2 border-b border-brand-500 bg-brand-soft px-3 py-2"
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
  );
}

export default UpdateBanner;
