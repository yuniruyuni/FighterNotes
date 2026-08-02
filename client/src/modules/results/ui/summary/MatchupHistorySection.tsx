import { useState } from "react";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { formatCharacterId } from "~/modules/analysis/contracts.js";
import type { AnalysisHistoryRecord } from "../../domain/history.js";
import { defensiveResponseBias } from "../../domain/history.js";
import { formatTacticRateWithCount } from "./tactic-stat-format.js";
import { useMatchupHistory } from "./use-matchup-history.js";

interface MatchupHistorySectionProps {
  file: File;
  context: AnalysisContext;
  report: AdviceReport;
}

export function MatchupHistorySection({
  file,
  context,
  report,
}: MatchupHistorySectionProps) {
  const history = useMatchupHistory(file, context, report);

  return (
    <section className="summary-section" data-wm="History">
      <h2>キャラクター別の対戦履歴</h2>
      <HistoryStorageControls
        history={history}
        rulesetVersion={report.ruleset_version}
      />
      <MatchupHistorySummary history={history} />
    </section>
  );
}

function HistoryStorageControls({
  history,
  rulesetVersion,
}: {
  history: ReturnType<typeof useMatchupHistory>;
  rulesetVersion: number;
}) {
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);
  const [confirmingClear, setConfirmingClear] = useState(false);
  const currentRulesetCount = history.records.filter(
    (record) => record.rulesetVersion === rulesetVersion,
  ).length;
  const controlsDisabled = history.phase === "loading" || history.busy !== null;

  const confirmDelete = async (id: string) => {
    if (await history.deleteRecord(id)) setPendingDeleteId(null);
  };
  const confirmClear = async () => {
    if (await history.clearHistory()) setConfirmingClear(false);
  };

  return (
    <div className="history-storage-panel">
      <div className="history-storage-heading">
        <div>
          <strong>このブラウザの解析履歴</strong>
          <p>
            対戦傾向の集計だけを最大200件保存します。動画とファイル名は保存しません。
          </p>
        </div>
        <label className="history-saving-toggle">
          <input
            checked={history.saving.enabled}
            disabled={controlsDisabled || !history.saving.persistent}
            onChange={(event) => {
              void history.setSavingEnabled(event.currentTarget.checked);
            }}
            type="checkbox"
          />
          今後の解析履歴を保存する
        </label>
      </div>

      {history.phase !== "loading" && !history.saving.persistent && (
        <p className="history-storage-warning" role="status">
          保存設定を利用できないため、新しい解析履歴は保存しません。
        </p>
      )}
      <p className="history-storage-disclosure">
        初期設定はONです。OFFにしても既存の履歴は残ります。解析履歴の削除は、共有URLや削除コードには影響しません。
      </p>
      {history.notice && (
        <p
          className="history-storage-notice"
          data-tone={history.notice.kind}
          role={history.notice.kind === "error" ? "alert" : "status"}
        >
          {history.notice.message}
        </p>
      )}

      <details className="history-management">
        <summary>
          保存済み履歴を管理（全判定版 {history.records.length}件）
        </summary>
        <p>
          現在の判定版は{currentRulesetCount}件、旧判定版は
          {history.records.length - currentRulesetCount}件です。
        </p>
        {history.records.length === 0 ? (
          <p className="muted-note">削除できる解析履歴はありません。</p>
        ) : (
          <ul className="history-record-list">
            {history.records.map((record) => (
              <li key={record.id}>
                <HistoryRecordDescription record={record} />
                {pendingDeleteId === record.id ? (
                  <span className="history-confirmation">
                    <span>この1件を削除しますか？</span>
                    <button
                      disabled={history.busy !== null}
                      onClick={() => void confirmDelete(record.id)}
                      type="button"
                    >
                      削除する
                    </button>
                    <button
                      disabled={history.busy !== null}
                      onClick={() => setPendingDeleteId(null)}
                      type="button"
                    >
                      キャンセル
                    </button>
                  </span>
                ) : (
                  <button
                    aria-label={`${historyRecordLabel(record)}を削除`}
                    disabled={history.busy !== null}
                    onClick={() => {
                      setConfirmingClear(false);
                      setPendingDeleteId(record.id);
                    }}
                    type="button"
                  >
                    削除
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}

        <div className="history-clear-controls">
          {confirmingClear ? (
            <div className="history-clear-confirmation">
              <strong>
                旧判定版を含む全{history.records.length}件を削除しますか？
              </strong>
              <span>この操作は取り消せません。共有情報は削除されません。</span>
              <div>
                <button
                  className="history-danger-button"
                  disabled={history.busy !== null}
                  onClick={() => void confirmClear()}
                  type="button"
                >
                  すべて削除する
                </button>
                <button
                  disabled={history.busy !== null}
                  onClick={() => setConfirmingClear(false)}
                  type="button"
                >
                  キャンセル
                </button>
              </div>
            </div>
          ) : (
            <button
              className="history-danger-button"
              disabled={history.records.length === 0 || history.busy !== null}
              onClick={() => {
                setPendingDeleteId(null);
                setConfirmingClear(true);
              }}
              type="button"
            >
              解析履歴をすべて削除
            </button>
          )}
        </div>
      </details>
    </div>
  );
}

function HistoryRecordDescription({
  record,
}: {
  record: AnalysisHistoryRecord;
}) {
  return (
    <span className="history-record-description">
      <strong>{historyRecordLabel(record)}</strong>
      <span>
        {formatHistoryDate(record.createdAt)}・判定版 {record.rulesetVersion}・
        {record.rounds}R
      </span>
    </span>
  );
}

function historyRecordLabel(record: AnalysisHistoryRecord): string {
  return `${formatCharacterId(record.ownCharacter)} vs ${formatCharacterId(
    record.opponentCharacter,
  )}`;
}

function formatHistoryDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "保存日時不明";
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function MatchupHistorySummary({
  history,
}: {
  history: ReturnType<typeof useMatchupHistory>;
}) {
  if (history.phase === "loading") {
    return <p className="muted-note">履歴を集計中...</p>;
  }
  if (history.phase === "error") return null;
  if (history.summaries.length === 0) {
    return (
      <p className="muted-note">現在の判定版の対戦履歴はまだありません。</p>
    );
  }
  return (
    <div className="table-scroll">
      <table className="round-table history-table">
        <thead>
          <tr>
            <th>組み合わせ</th>
            <th>試合</th>
            <th>対空</th>
            <th>DI返し</th>
            <th>生ラッシュ対処</th>
            <th>不利後の回答偏り</th>
            <th>バーンアウト収支</th>
          </tr>
        </thead>
        <tbody>
          {history.summaries.map((summary) => {
            const burnoutBalance = Math.round(
              (summary.burnoutHpDealt - summary.burnoutHpLost) * 100,
            );
            const responseBias = defensiveResponseBias(summary);
            return (
              <tr key={summary.key}>
                <td>
                  {formatCharacterId(summary.ownCharacter)} vs{" "}
                  {formatCharacterId(summary.opponentCharacter)}
                </td>
                <td>{summary.matches}</td>
                <td>
                  {formatTacticRateWithCount(
                    summary.antiAirSuccesses,
                    summary.antiAirOpportunities,
                  )}
                </td>
                <td>
                  {formatTacticRateWithCount(
                    summary.diReturned,
                    summary.diFaced,
                    summary.diUnconfirmed,
                  )}
                </td>
                <td>
                  {formatTacticRateWithCount(
                    summary.rawRushesDefended,
                    summary.rawRushesFaced,
                    summary.rawRushesUnconfirmed,
                  )}
                </td>
                <td>{formatResponseBias(responseBias)}</td>
                <td>
                  {burnoutBalance > 0 ? "+" : ""}
                  {burnoutBalance}%
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function formatResponseBias(
  bias: ReturnType<typeof defensiveResponseBias>,
): string {
  if (!bias) return "-";
  const action = bias.action === "strike" ? "最速打撃" : "最速投げ";
  return `${action} ${bias.selectionPercent}%（被弾${bias.losses}/${bias.selections}）`;
}
