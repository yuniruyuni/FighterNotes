import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "~/app/App.js";
import { trackDailyUse } from "./daily-usage";

const root = document.getElementById("root");
if (!root) throw new Error("React root element was not found");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

trackDailyUse();
