import type { AdviceReport } from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "../../domain/scene-selection.js";

interface RoundSectionProps {
  report: AdviceReport;
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function RoundSection({ report, onSceneChange }: RoundSectionProps) {
  const rounds = report.round_summaries ?? [];
  return (
    <section className="summary-section" data-wm="Rounds">
      <h2>ラウンド経過</h2>
      {rounds.length === 0 ? (
        <p className="muted-note">ラウンドを検出できませんでした。</p>
      ) : (
        <div className="table-scroll">
          <table className="round-table">
            <thead>
              <tr>
                <th>R</th>
                <th>勝敗</th>
                <th>残りHP（自分）</th>
                <th>残りHP（相手）</th>
                <th>被弾回数</th>
                <th>失ったHP</th>
                <th>開幕被弾</th>
                <th>バーンアウト</th>
              </tr>
            </thead>
            <tbody>
              {rounds.map((round) => {
                const outcome =
                  round.won === null ? "—" : round.won ? "WIN" : "LOSE";
                const outcomeClass =
                  round.won === null ? "" : round.won ? "win" : "lose";
                const open = () =>
                  onSceneChange({
                    frame: round.start_frame,
                    card: null,
                    label: `ラウンド ${round.round_no} 開始`,
                  });
                return (
                  <tr
                    className="clickable"
                    key={round.round_no}
                    onClick={(event) => {
                      if (
                        event.target instanceof Element &&
                        event.target.closest("button")
                      ) {
                        return;
                      }
                      open();
                    }}
                  >
                    <td>
                      <button
                        type="button"
                        className="round-scene-button"
                        aria-label={`ラウンド ${round.round_no} の開始場面を動画で開く`}
                        title={`検出信頼度: ${round.detection_confidence || "未記録"}`}
                        onClick={open}
                      >
                        {round.round_no}
                        {round.detection_confidence === "medium" ? " ⚠" : ""}
                      </button>
                    </td>
                    <td className={outcomeClass}>{outcome}</td>
                    <td>{Math.round(round.own_hp_end * 100)}%</td>
                    <td>{Math.round(round.opp_hp_end * 100)}%</td>
                    <td>{round.own_hits_taken}</td>
                    <td>{Math.round(round.own_hp_lost * 100)}%</td>
                    <td>{round.early_hit ? "⚠" : ""}</td>
                    <td>
                      {round.own_burnouts > 0
                        ? "🔥".repeat(round.own_burnouts)
                        : ""}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
