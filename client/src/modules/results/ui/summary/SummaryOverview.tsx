import type { ReactNode } from "react";
import type { AdviceReport } from "~/modules/analysis/contracts.js";

export function SummaryOverview({
  report,
  sharing,
}: {
  report: AdviceReport;
  sharing?: ReactNode;
}) {
  const cards = report.cards ?? [];
  const diagnosisCount = cards.filter(
    (card) => card.kind === "diagnosis",
  ).length;
  const observationCount = cards.filter(
    (card) => card.kind === "observation" || card.kind === undefined,
  ).length;
  const badges = [
    `${report.rounds_detected} ラウンド`,
    `被弾候補 ${report.damage_taken_events.length} 件`,
    `改善ポイント ${diagnosisCount} 件`,
    `確認場面 ${observationCount} 件`,
  ];
  if (report.coverage && report.coverage.match_frames > 0) {
    badges.push(
      `解析範囲 ${Math.round(
        (report.coverage.analyzed_match_frames / report.coverage.match_frames) *
          100,
      )}%`,
    );
  }
  if (report.analyzer_build_id) {
    badges.push(`解析器 ${report.analyzer_build_id}`);
  }

  return (
    <section className="summary-section" data-wm="Summary">
      <h2>解析結果サマリー</h2>
      <div className="summary-box">{report.summary}</div>
      <p className="summary-caveat">
        解析結果は映像からの推定です。正確な記録ではなく、見直しのための参考情報として利用してください。
      </p>
      <div className="summary-badges">
        {badges.map((badge) => (
          <span className="badge" key={badge}>
            {badge}
          </span>
        ))}
      </div>
      <div>
        {(report.analysis_warnings ?? []).map((warning) => (
          <p className="analysis-warning" key={warning}>
            ⚠ {warning}
          </p>
        ))}
      </div>
      {sharing}
    </section>
  );
}
