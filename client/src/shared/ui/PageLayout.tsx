import type { ReactNode } from "react";
import { AppFooter } from "./AppFooter.js";
import { AppHeader } from "./AppHeader.js";

export function PageLayout({ children }: { children: ReactNode }) {
  return (
    <div className="page-layout">
      <AppHeader />
      <div className="page-layout-content">{children}</div>
      <AppFooter />
    </div>
  );
}
