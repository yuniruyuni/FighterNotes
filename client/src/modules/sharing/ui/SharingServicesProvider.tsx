import { createContext, type ReactNode, useContext } from "react";
import type { SharingServices } from "../application/ports.js";

const SharingServicesContext = createContext<SharingServices | null>(null);

export function SharingServicesProvider({
  children,
  services,
}: {
  children: ReactNode;
  services: SharingServices;
}) {
  return (
    <SharingServicesContext.Provider value={services}>
      {children}
    </SharingServicesContext.Provider>
  );
}

export function useSharingServices(): SharingServices {
  const services = useContext(SharingServicesContext);
  if (!services) {
    throw new Error(
      "useSharingServices must be used within SharingServicesProvider",
    );
  }
  return services;
}
