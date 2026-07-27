import { useManagedShares } from "./use-managed-shares.js";
import { useManualShareDeletion } from "./use-manual-share-deletion.js";

export function useShareManagement(
  initialId: string,
  sharePath: (id: string) => string,
) {
  const managed = useManagedShares(sharePath);
  const manual = useManualShareDeletion(initialId, managed.refresh);
  return { ...managed, ...manual };
}
