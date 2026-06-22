import { Toaster } from "@gio/bigsu-ui";
import LoginScreen from "./components/LoginScreen";
import Widget from "./components/Widget";
import LoadingState from "./components/LoadingState";
import useAuth from "./hooks/useAuth";

function App() {
  const { status, savedUsername, initialError, login } = useAuth();

  if (status === "checking") {
    return (
      <div className="flex h-screen flex-col bg-app text-text-primary">
        <LoadingState />
      </div>
    );
  }

  if (status === "unauthenticated") {
    return (
      <>
        <Toaster />
        <LoginScreen
          defaultUsername={savedUsername}
          initialError={initialError}
          onLogin={login}
        />
      </>
    );
  }

  return <Widget />;
}

export default App;
