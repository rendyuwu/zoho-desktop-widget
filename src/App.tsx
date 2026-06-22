import WidgetHeader from "./components/WidgetHeader";
import EmptyTicketState from "./components/EmptyTicketState";
import LoadingState from "./components/LoadingState";
import ErrorTicketState from "./components/ErrorTicketState";
import useTicketEvents from "./hooks/useTicketEvents";
import { ASAP_THRESHOLD } from "./constants";

function countAsap(tickets: { timestamp: number }[]): number {
  const now = Math.floor(Date.now() / 1000);
  return tickets.filter((t) => now - t.timestamp >= ASAP_THRESHOLD).length;
}

function App() {
  const { data, loading, error } = useTicketEvents();

  const waiting = data?.waiting_response ?? [];
  const asapCount = countAsap(waiting);
  const showEmpty = !loading && !error && data !== null && waiting.length === 0;

  return (
    <div className="flex h-screen flex-col bg-app text-text-primary">
      <WidgetHeader asapCount={asapCount} />
      {loading && <LoadingState />}
      {error && !data && <ErrorTicketState />}
      {showEmpty && <EmptyTicketState />}
      {!loading && !error && data && waiting.length > 0 && (
        <main className="flex-1 overflow-y-auto p-3">
          <p className="text-sm text-text-muted">
            {waiting.length} ticket{waiting.length !== 1 ? "s" : ""} waiting
          </p>
        </main>
      )}
    </div>
  );
}

export default App;
