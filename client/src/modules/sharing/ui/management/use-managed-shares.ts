import { useCallback, useEffect, useState } from "react";
import { deleteStoredPublication } from "../../application/share-management-service.js";
import type { ManagedShare } from "../../domain/managed-share.js";
import {
  type Feedback,
  ShareManagement,
} from "../../domain/share-management.js";
import { useSharingServices } from "../SharingServicesProvider.js";

export function useManagedShares(sharePath: (id: string) => string) {
  const services = useSharingServices();
  const [snapshot, setSnapshot] = useState(() =>
    services.managedShares.load(services.now()),
  );
  const [feedback, setFeedback] = useState<Feedback>(() =>
    ShareManagement.emptyFeedback(),
  );
  const [deletingId, setDeletingId] = useState("");
  const refresh = useCallback(
    () => setSnapshot(services.managedShares.load(services.now())),
    [services],
  );

  useEffect(() => {
    return services.managedShares.subscribe(() => {
      refresh();
      setFeedback(ShareManagement.emptyFeedback());
    });
  }, [refresh, services]);

  const copy = async (value: string, label: string) => {
    try {
      await services.capabilities.copyText(value);
      setFeedback({ message: `${label}をコピーしました。`, tone: "success" });
    } catch {
      setFeedback({
        message: `${label}をコピーできませんでした。`,
        tone: "error",
      });
    }
  };

  const deleteManaged = async (share: ManagedShare) => {
    if (
      !services.capabilities.confirm(
        `「${share.label}」の共有結果を削除します。共有先からも閲覧できなくなります。続行しますか？`,
      )
    ) {
      return;
    }
    setDeletingId(share.id);
    setFeedback({ message: "共有結果を削除しています。", tone: "pending" });
    try {
      if (!(await deleteStoredPublication(share, services))) {
        setFeedback({
          message:
            "共有結果は削除しましたが、このブラウザの一覧から取り除けませんでした。",
          tone: "error",
        });
        return;
      }
      refresh();
      setFeedback({
        message:
          "共有結果を削除しました。新しいアクセスには約15秒以内に反映されます。",
        tone: "success",
      });
    } catch {
      setFeedback({
        message:
          "削除できませんでした。すでに削除済みか、公開期限を過ぎている可能性があります。",
        tone: "error",
      });
    } finally {
      setDeletingId("");
    }
  };

  return {
    snapshot,
    listFeedback: feedback.message
      ? feedback
      : ShareManagement.snapshotFeedback(snapshot),
    deletingId,
    refresh,
    copyDeleteCode: (share: ManagedShare) =>
      copy(share.deleteCode, "削除コード"),
    copyPublicUrl: (share: ManagedShare) =>
      copy(
        new URL(sharePath(share.id), services.capabilities.origin()).toString(),
        "公開URL",
      ),
    deleteManaged,
  };
}
