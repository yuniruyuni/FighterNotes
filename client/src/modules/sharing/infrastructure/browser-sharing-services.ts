import type { SharingServices } from "../application/ports.js";
import { generateDeleteCode } from "../domain/delete-code.js";
import { BrowserShareCapabilities } from "./browser-share-capabilities.js";
import { browserManagedShareRepository } from "./managed-share-store.js";
import { browserPublishedAnalysisGateway } from "./share-api.js";

export const browserSharingServices: SharingServices = {
  gateway: browserPublishedAnalysisGateway,
  managedShares: browserManagedShareRepository,
  capabilities: BrowserShareCapabilities,
  generateDeleteCode: () =>
    generateDeleteCode((values) => crypto.getRandomValues(values)),
  now: () => new Date(),
};
