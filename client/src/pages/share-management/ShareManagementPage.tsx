import { Link } from "wouter";
import { paths } from "~/app/paths.js";
import {
  ManagedShareList,
  ManualShareDeletionForm,
  useShareManagement,
} from "~/modules/sharing/index.js";

export function ShareManagementPage({
  initialId = "",
}: {
  initialId?: string;
}) {
  const management = useShareManagement(initialId, paths.share);
  return (
    <main id="screen-share-management">
      <section className="card management-card">
        <h1>公開した分析結果を管理</h1>
        <p className="management-notice">
          共有URLの作成時に削除コードを発行し、このブラウザに保存します。
          削除後、新しいアクセスには約15秒以内に反映されます。
        </p>
        <ManagedShareList
          snapshot={management.snapshot}
          feedback={management.listFeedback}
          deletingId={management.deletingId}
          sharePath={paths.share}
          onCopyPublicUrl={(share) => void management.copyPublicUrl(share)}
          onCopyDeleteCode={(share) => void management.copyDeleteCode(share)}
          onDelete={(share) => void management.deleteManaged(share)}
        />
        <ManualShareDeletionForm
          reference={management.reference}
          deleteCredential={management.deleteCredential}
          busy={management.manualBusy}
          feedback={management.manualFeedback}
          onReferenceChange={management.setReference}
          onCredentialChange={management.setDeleteCredential}
          onSubmit={() => void management.deleteManually()}
        />
        <Link className="share-management-link" href={paths.home}>
          Fighter Notes に戻る
        </Link>
      </section>
    </main>
  );
}
