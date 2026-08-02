import { Copy, ExternalLink, Share2, Trash2 } from "lucide-react";
import { Link } from "wouter";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { sharingUnavailableReason } from "../domain/published-analysis.js";
import { buildXIntentUrl, NATIVE_SHARE_TEXT } from "../domain/share-links.js";
import { usePublication } from "./PublicationProvider.js";
import { useSharingServices } from "./SharingServicesProvider.js";

export function SharePanel({
  context,
  manageHref,
  report,
}: {
  context: AnalysisContext;
  manageHref: string;
  report: AdviceReport;
}) {
  const { capabilities } = useSharingServices();
  const { state, publish, retry, deleteShare, setFeedback } = usePublication();
  const { source, published, phase, storedLocally } = state;
  const busy = phase === "creating" || phase === "deleting";
  const showRetry = !published && (phase === "failed" || phase === "deleted");
  const showCreate = !published && !showRetry;
  const retryLabel =
    phase === "deleted" ? "共有URLを再作成" : "共有URLを再試行";
  const unavailableReason = sharingUnavailableReason(report.ruleset_version);

  const copy = async (value: string, label: string) => {
    try {
      await capabilities.copyText(value);
      setFeedback(`${label}をコピーしました。`, "success");
    } catch {
      setFeedback(`${label}をコピーできませんでした。`, "error");
    }
  };

  const openNativeShare = async () => {
    if (!published || !capabilities.canShare()) return;
    try {
      await capabilities.share({
        title: "Fighter Notes 分析結果",
        text: NATIVE_SHARE_TEXT,
        url: published.url,
      });
      setFeedback("共有メニューを開きました。", "success");
    } catch (error) {
      if (!capabilities.isCancelledShare(error)) {
        setFeedback("共有メニューを開けませんでした。", "error");
      }
    }
  };

  return (
    <div className="share-panel">
      <div className="share-heading">
        <strong>{published ? "解析結果の公開URL" : "解析結果を共有"}</strong>
        <span>
          {published
            ? "この端末では動画付き、共有先では動画なしの公開ページを表示します"
            : "解析しただけでは、解析結果をサーバーへ送信・公開しません"}
        </span>
      </div>
      <div className="share-controls">
        {showCreate && (
          <button
            type="button"
            className="share-command share-command-primary"
            aria-describedby="share-disclosure"
            title={unavailableReason ?? "共有URLを生成"}
            disabled={busy || unavailableReason !== undefined}
            onClick={() => void publish(report, context)}
          >
            <Share2 size={18} aria-hidden="true" />
            <span>{busy ? "共有URLを生成中…" : "共有URLを生成"}</span>
          </button>
        )}
        {showRetry && (
          <button
            type="button"
            className="share-command share-command-primary"
            aria-describedby="share-disclosure"
            title={retryLabel}
            disabled={!source || unavailableReason !== undefined}
            onClick={() => void retry()}
          >
            <Share2 size={18} aria-hidden="true" />
            <span>{retryLabel}</span>
          </button>
        )}
        {published && (
          <div className="share-actions">
            {capabilities.canShare() && (
              <button
                type="button"
                className="share-command"
                title="端末の共有メニューを開く"
                disabled={busy}
                onClick={() => void openNativeShare()}
              >
                <Share2 size={18} aria-hidden="true" />
                <span>共有</span>
              </button>
            )}
            <a
              className="share-command"
              href={buildXIntentUrl(published.url)}
              target="_blank"
              rel="noopener noreferrer"
              title="Xに投稿"
            >
              <ExternalLink size={18} aria-hidden="true" />
              <span>Xに投稿</span>
            </a>
            <button
              type="button"
              className="share-command"
              title="共有URLをコピー"
              disabled={busy}
              onClick={() => void copy(published.url, "共有URL")}
            >
              <Copy size={18} aria-hidden="true" />
              <span>公開URL</span>
            </button>
            <button
              type="button"
              className="share-command share-command-danger"
              title="共有結果を削除"
              disabled={busy}
              onClick={() => void deleteShare()}
            >
              <Trash2 size={18} aria-hidden="true" />
              <span>削除</span>
            </button>
          </div>
        )}
      </div>

      {!published && unavailableReason && (
        <p className="share-disclosure" role="note">
          {unavailableReason}
        </p>
      )}

      {!published && (
        <p className="share-disclosure" id="share-disclosure">
          「共有URLを生成」を押したときだけ、キャラクター、ラウンド集計、指摘、戦術統計に加え、両者のSA/CAレベル別使用回数と結果、自分側の利用文脈をサーバーへ送信し、30日間公開します。完全集計できない場合、検出できた件数だけを下限として表示します。
          動画、ファイル名、入力履歴、場面クリップ、SA/CAの正確なダメージ値と最終ゲージ量は送信しません。
          URLを知る人は閲覧でき、外部サービスのプレビューやスクリーンショットは削除・公開期限後も残る場合があります。
          削除コードはURLの生成時に発行してこのブラウザに保存し、別の端末からの削除にも使用できます。
          詳細は<Link href="/privacy">プライバシーポリシー</Link>
          を確認してください。
        </p>
      )}

      {published && source && (
        <div className="share-delete-code-panel">
          <strong>削除コード</strong>
          <code>{source.deleteCode}</code>
          <button
            type="button"
            className="share-command"
            title="削除コードをコピー"
            disabled={busy}
            onClick={() => void copy(source.deleteCode, "削除コード")}
          >
            <Copy size={18} aria-hidden="true" />
            <span>コードをコピー</span>
          </button>
          <span
            className="share-status share-storage-status"
            data-tone={storedLocally ? "success" : "error"}
            role="status"
            aria-live="polite"
          >
            {storedLocally
              ? "このブラウザに保存しました。別端末で削除する場合だけ控え、共有相手には送らないでください。"
              : "このブラウザには保存できませんでした。ページを離れる前に控え、共有相手には送らないでください。"}
          </span>
        </div>
      )}

      <div className="share-feedback">
        <span
          className="share-status"
          data-tone={state.tone}
          role="status"
          aria-live="polite"
        >
          {state.status}
        </span>
        {published && (
          <span className="share-expiry">
            公開期限:{" "}
            {new Intl.DateTimeFormat("ja-JP", { dateStyle: "long" }).format(
              new Date(published.expiresAt),
            )}
            （削除コードで期限前に削除できます）
          </span>
        )}
        <Link className="share-management-link" href={manageHref}>
          公開した共有を管理
        </Link>
      </div>
    </div>
  );
}
