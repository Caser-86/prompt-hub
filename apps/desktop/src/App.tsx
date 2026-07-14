import { AppShell } from "./app/AppShell";
import { ErrorBoundary } from "./app/ErrorBoundary";

export function App() {
  return (
    <ErrorBoundary>
      <AppShell />
    </ErrorBoundary>
  );
}
