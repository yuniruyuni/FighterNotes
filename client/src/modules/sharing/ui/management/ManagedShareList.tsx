import { Copy, Trash2 } from "lucide-react";
import type {
  ManagedShare,
  ManagedShareSnapshot,
} from "../../domain/managed-share.js";
import type { Feedback } from "../../domain/share-management.js";

interface ManagedShareListProps {
  snapshot: ManagedShareSnapshot;
  feedback: Feedback;
  deletingId: string;
  sharePath(id: string): string;
  onCopyPublicUrl(share: ManagedShare): void;
  onCopyDeleteCode(share: ManagedShare): void;
  onDelete(share: ManagedShare): void;
}

export function ManagedShareList(props: ManagedShareListProps) {
  return (
    <section
      className="management-section"
      aria-labelledby="local-share-heading"
    >
      <h2 id="local-share-heading">この端末で作成した共有</h2>
      <p>
        共有ID・削除コード・作成日時・期限・対戦キャラクターだけをこのブラウザに保存しています。
        ブラウザのデータを消すと一覧からも消えるため、別端末で使うコードは控えてください。
      </p>
      <div id="share-management-list">
        {props.snapshot.shares.map((share) => (
          <article className="managed-share-item" key={share.id}>
            <div className="managed-share-details">
              <a
                href={props.sharePath(share.id)}
                target="_blank"
                rel="noopener noreferrer"
              >
                {share.label}
              </a>
              <span>
                {formatDate(share.createdAt)} 作成・
                {formatDate(share.expiresAt)}
                まで公開
              </span>
              <span className="managed-share-code">
                削除コード: <code>{share.deleteCode}</code>
              </span>
            </div>
            <div className="managed-share-controls">
              <ShareCommand
                label="公開URLをコピー"
                disabled={Boolean(props.deletingId)}
                onClick={() => props.onCopyPublicUrl(share)}
              />
              <ShareCommand
                label="削除コードをコピー"
                disabled={Boolean(props.deletingId)}
                onClick={() => props.onCopyDeleteCode(share)}
              />
              <button
                type="button"
                className="share-command share-command-danger"
                title="共有結果を削除"
                disabled={Boolean(props.deletingId)}
                onClick={() => props.onDelete(share)}
              >
                <Trash2 size={17} aria-hidden="true" />
                <span>
                  {props.deletingId === share.id ? "削除中…" : "削除"}
                </span>
              </button>
            </div>
          </article>
        ))}
      </div>
      <span
        id="share-management-list-status"
        className="share-status"
        data-tone={props.feedback.tone}
        role="status"
        aria-live="polite"
      >
        {props.feedback.message}
      </span>
    </section>
  );
}

function ShareCommand({
  label,
  disabled,
  onClick,
}: {
  label: string;
  disabled: boolean;
  onClick(): void;
}) {
  return (
    <button
      type="button"
      className="share-command"
      title={label}
      disabled={disabled}
      onClick={onClick}
    >
      <Copy size={17} aria-hidden="true" />
      <span>{label.replace("をコピー", "")}</span>
    </button>
  );
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat("ja-JP", { dateStyle: "medium" }).format(
    new Date(value),
  );
}
