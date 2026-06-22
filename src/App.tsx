import { Button, Badge } from "@gio/bigsu-ui";

function App() {
  return (
    <div className="bg-app text-text-primary p-4">
      <h1 className="text-lg font-semibold mb-2">Zoho Widget</h1>
      <div className="flex gap-2 items-center">
        <Button variant="primary">Test</Button>
        <Badge variant="info">BIGSU OK</Badge>
      </div>
    </div>
  );
}

export default App;
