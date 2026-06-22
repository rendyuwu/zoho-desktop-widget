import { useState } from "react";
import { Toaster } from "@gio/bigsu-ui";
import UpdateBanner from "./UpdateBanner";
import WidgetHeader, { type DisplayMode } from "./WidgetHeader";
import CountGrid from "./CountGrid";
import EmptyTicketState from "./EmptyTicketState";
import LoadingState from "./LoadingState";
import ErrorTicketState from "./ErrorTicketState";
import AsapList from "./AsapList";
import WaitingList from "./WaitingList";
import useTicketEvents from "../hooks/useTicketEvents";
import { classifyTicket } from "../constants";

interface WidgetProps {
  onLogout: () => void;
}

function Widget({ onLogout }: WidgetProps) {
  const { data, loading, error, tick } = useTicketEvents();
  const [mode, setMode] = useState<DisplayMode>("all");

  const waiting = data?.waiting_response ?? [];
  const asapCount = waiting.filter((t) =>
    classifyTicket(Math.floor(Date.now() / 1000) - t.timestamp) === "asap",
  ).length;

  const showCounts = mode === "all" || mode === "total";
  const showWaiting = mode === "all" || mode === "waiting";
  const showEmpty = !loading && !error && data !== null && waiting.length === 0;

  return (
    <div className="flex h-screen flex-col bg-app text-text-primary">
      <Toaster />
      <UpdateBanner />
      <WidgetHeader asapCount={asapCount} mode={mode} onModeChange={setMode} onLogout={onLogout} />
      {showCounts && <CountGrid data={data} loading={loading} />}
      {loading && <LoadingState />}
      {error && !data && <ErrorTicketState />}
      {showEmpty && showWaiting && <EmptyTicketState />}
      {!loading && !error && data && waiting.length > 0 && showWaiting && (
        <main className="flex-1 overflow-y-auto p-3" key={tick}>
          <div className="flex flex-col gap-3">
            {asapCount > 0 && <AsapList tickets={waiting} />}
            <WaitingList tickets={waiting} />
          </div>
        </main>
      )}
    </div>
  );
}

export default Widget;
