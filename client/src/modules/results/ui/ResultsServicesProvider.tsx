import { createContext, type ReactNode, useContext } from "react";
import type { ResultsServices } from "../application/ports.js";

const ResultsServicesContext = createContext<ResultsServices | null>(null);

export function ResultsServicesProvider({
  children,
  services,
}: {
  children: ReactNode;
  services: ResultsServices;
}) {
  return (
    <ResultsServicesContext.Provider value={services}>
      {children}
    </ResultsServicesContext.Provider>
  );
}

export function useResultsServices(): ResultsServices {
  const services = useContext(ResultsServicesContext);
  if (!services) {
    throw new Error(
      "useResultsServices must be used within ResultsServicesProvider",
    );
  }
  return services;
}
