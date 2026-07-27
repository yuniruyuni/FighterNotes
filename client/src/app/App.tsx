import { AppProviders } from "./providers.js";
import { AppRoutes } from "./routes.js";

export function App() {
  return (
    <AppProviders>
      <AppRoutes />
    </AppProviders>
  );
}
