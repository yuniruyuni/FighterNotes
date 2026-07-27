import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { formatCharacterId } from "~/modules/analysis/contracts.js";
import { defensiveResponseBias, rate } from "../../domain/history.js";
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
      <MatchupHistoryContent history={history} />
    </section>
  );
}

function MatchupHistoryContent({
  history,
}: {
  history: ReturnType<typeof useMatchupHistory>;
}) {
  if (history.phase === "loading") {
    return <p className="muted-note">履歴を集計中...</p>;
  }
  if (history.phase === "error") {
    return <p className="muted-note">対戦履歴を読み込めませんでした。</p>;
  }
  if (history.summaries.length === 0) {
    return <p className="muted-note">対戦履歴はまだありません。</p>;
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
                  {formatRateWithCount(
                    summary.antiAirSuccesses,
                    summary.antiAirOpportunities,
                  )}
                </td>
                <td>
                  {formatRateWithCount(summary.diReturned, summary.diFaced)}
                </td>
                <td>
                  {formatRateWithCount(
                    summary.rawRushesDefended,
                    summary.rawRushesFaced,
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

function formatRateWithCount(successes: number, opportunities: number): string {
  return opportunities === 0
    ? "-"
    : `${rate(successes, opportunities)} (${successes}/${opportunities})`;
}

function formatResponseBias(
  bias: ReturnType<typeof defensiveResponseBias>,
): string {
  if (!bias) return "-";
  const action = bias.action === "strike" ? "最速打撃" : "最速投げ";
  return `${action} ${bias.selectionPercent}%（被弾${bias.losses}/${bias.selections}）`;
}
