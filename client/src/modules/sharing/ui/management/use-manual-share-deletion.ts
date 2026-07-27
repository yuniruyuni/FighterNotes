import { useEffect, useState } from "react";
import { deletePublicationWithCredential } from "../../application/share-management-service.js";
import {
  type Feedback,
  ShareManagement,
} from "../../domain/share-management.js";
import { useSharingServices } from "../SharingServicesProvider.js";

export function useManualShareDeletion(
  initialId: string,
  refreshManagedShares: () => void,
) {
  const services = useSharingServices();
  const [feedback, setFeedback] = useState<Feedback>(() =>
    ShareManagement.emptyFeedback(),
  );
  const [reference, setReference] = useState(initialId);
  const [deleteCredential, setDeleteCredential] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => setReference(initialId), [initialId]);

  const deleteManually = async () => {
    const request = ShareManagement.manualDeletionRequest(
      reference,
      deleteCredential,
      services.capabilities.origin(),
    );
    if (!request.valid) {
      setFeedback(request.feedback);
      return;
    }
    if (
      !services.capabilities.confirm("この共有結果を削除します。続行しますか？")
    ) {
      return;
    }

    setBusy(true);
    setFeedback({ message: "共有結果を削除しています。", tone: "pending" });
    try {
      await deletePublicationWithCredential(
        request.id,
        request.credential,
        services,
      );
      services.managedShares.remove(request.id);
      refreshManagedShares();
      setDeleteCredential("");
      setFeedback({
        message:
          "共有結果を削除しました。新しいアクセスには約15秒以内に反映されます。",
        tone: "success",
      });
    } catch {
      setFeedback({
        message:
          "削除できませんでした。共有URL、削除コード、公開期限を確認してください。",
        tone: "error",
      });
    } finally {
      setBusy(false);
    }
  };

  return {
    manualFeedback: feedback,
    reference,
    deleteCredential,
    manualBusy: busy,
    setReference,
    setDeleteCredential,
    deleteManually,
  };
}
