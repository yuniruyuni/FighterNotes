import { deleteCredentialCandidates } from "../domain/delete-code.js";
import type { ManagedShare } from "../domain/managed-share.js";
import type { SharingServices } from "./ports.js";

export async function deleteStoredPublication(
  share: ManagedShare,
  services: SharingServices,
): Promise<boolean> {
  await services.gateway.delete({ id: share.id }, share.deleteCode);
  return services.managedShares.remove(share.id);
}

export async function deletePublicationWithCredential(
  id: string,
  credential: string,
  services: SharingServices,
): Promise<void> {
  let lastError: unknown;
  for (const candidate of deleteCredentialCandidates(credential)) {
    try {
      await services.gateway.delete({ id }, candidate);
      return;
    } catch (error) {
      lastError = error;
    }
  }
  throw lastError;
}
