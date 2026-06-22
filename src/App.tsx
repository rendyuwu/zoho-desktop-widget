import UpdateBanner from "./components/UpdateBanner";
import WidgetHeader from "./components/WidgetHeader";
import CountGrid from "./components/CountGrid";
import EmptyTicketState from "./components/EmptyTicketState";
import LoadingState from "./components/LoadingState";
import ErrorTicketState from "./components/ErrorTicketState";
import AsapList from "./components/AsapList";
import WaitingList from "./components/WaitingList";
import useTicketEvents from "./hooks/useTicketEvents";
import { classifyTicket } from "./constants";

function App() {
  const { data, loading, error, tick } = useTicketEvents();

  const waiting = data?.waiting_response ?? [];
  const asapCount = waiting.filter((t) =>
    classifyTicket(Math.floor(Date.now() / 1000) - t.timestamp) === "asap",
  ).length;
  const showEmpty = !loading && !error && data !== null && waiting.length === 0;

  return (
    <div className="flex h-screen flex-col bg-app text-text-primary">
      <UpdateBanner />
      <WidgetHeader asapCount={asapCount} />
      <CountGrid data={data} loading={loading} />
      {loading && <LoadingState />}
      {error && !data && <ErrorTicketState />}
      {showEmpty && <EmptyTicketState />}
      {!loading && !error && data && waiting.length > 0 && (
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

export default App;
