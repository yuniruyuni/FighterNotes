import type { ReactNode, Ref } from "react";
import {
  type AdviceReport,
  type AnalysisContext,
  formatCharacterId,
} from "~/modules/analysis/contracts.js";

function coveragePercent(observed: number | undefined, total: number): number {
  return Math.min(100, Math.round(((observed ?? 0) / total) * 100));
}

function sideCoverage(
  label: string,
  own: number | undefined,
  opponent: number | undefined,
  total: number,
): string {
  return `${label} 自分 ${coveragePercent(own, total)}% / 相手 ${coveragePercent(opponent, total)}%`;
}

export function SummaryOverview({
  context,
  report,
  sharing,
  headingRef,
}: {
  context: AnalysisContext;
  report: AdviceReport;
  sharing?: ReactNode;
  headingRef?: Ref<HTMLHeadingElement>;
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
      `ラウンド割当 ${Math.round(
        (report.coverage.analyzed_match_frames / report.coverage.match_frames) *
          100,
      )}%`,
    );
  }
  const detectorFrames = report.coverage?.detector_match_frames ?? 0;
  if (report.coverage && detectorFrames > 0) {
    badges.push(
      sideCoverage(
        "HP認識",
        report.coverage.own_hp_reliable_frames,
        report.coverage.opponent_hp_reliable_frames,
        detectorFrames,
      ),
      sideCoverage(
        "Drive認識",
        report.coverage.own_drive_reliable_frames,
        report.coverage.opponent_drive_reliable_frames,
        detectorFrames,
      ),
      sideCoverage(
        "SA認識",
        report.coverage.own_super_reliable_frames,
        report.coverage.opponent_super_reliable_frames,
        detectorFrames,
      ),
      `${sideCoverage(
        "入力直接観測",
        report.coverage.own_input_observed_frames,
        report.coverage.opponent_input_observed_frames,
        detectorFrames,
      )}（補間 自分 ${coveragePercent(report.coverage.own_input_repaired_frames, detectorFrames)}% / 相手 ${coveragePercent(report.coverage.opponent_input_repaired_frames, detectorFrames)}%）`,
      sideCoverage(
        "フレームメーター対応",
        report.coverage.own_meter_mapped_frames,
        report.coverage.opponent_meter_mapped_frames,
        detectorFrames,
      ),
    );
  }
  const spatialCandidates = report.coverage?.spatial_candidate_frames ?? 0;
  if (report.coverage && spatialCandidates > 0) {
    badges.push(
      `空間復号 ${report.coverage.spatial_sampled_frames ?? 0} / ${spatialCandidates} フレーム（距離利用可 ${report.coverage.spatial_usable_frames ?? 0}・人物直接観測 自分 ${report.coverage.own_spatial_observed_frames ?? 0} / 相手 ${report.coverage.opponent_spatial_observed_frames ?? 0}）`,
    );
  }
  const attackEvents = report.coverage?.attack_damage_events ?? 0;
  if (report.coverage && attackEvents > 0) {
    if (report.coverage.own_attack_damage_events !== undefined) {
      badges.push(
        `攻撃表示 厳格利用 自分 ${report.coverage.own_attack_damage_usable ?? 0} / ${report.coverage.own_attack_damage_events}・相手 ${report.coverage.opponent_attack_damage_usable ?? 0} / ${report.coverage.opponent_attack_damage_events ?? 0}（全帰属 ${report.coverage.attack_damage_linked ?? 0} / ${attackEvents}）`,
      );
    } else {
      badges.push(
        `攻撃表示 帰属 ${report.coverage.attack_damage_linked ?? 0} / ${attackEvents}（HP整合 ${report.coverage.attack_damage_consistent ?? 0}・不一致 ${report.coverage.attack_damage_mismatched ?? 0}・未照合 ${report.coverage.attack_damage_unverified ?? 0}）`,
      );
    }
  }
  if (report.analyzer_build_id) {
    badges.push(`解析器 ${report.analyzer_build_id}`);
  }
  const ownIsP1 = context.ownSide === "p1";
  const own = ownIsP1 ? context.p1 : context.p2;
  const opponent = ownIsP1 ? context.p2 : context.p1;
  const ownSideLabel = ownIsP1 ? "1P（左）" : "2P（右）";
  const opponentSideLabel = ownIsP1 ? "2P（右）" : "1P（左）";

  return (
    <section className="summary-section" data-wm="Summary">
      <h2
        id="summary-view-heading"
        className="workspace-focus-heading"
        ref={headingRef}
        tabIndex={-1}
      >
        解析結果サマリー
      </h2>
      <div className="analysis-side-confirmation">
        <span>解析対象</span>
        <strong>
          {ownSideLabel}・{formatCharacterId(own.character)}
        </strong>
        <span>
          相手: {opponentSideLabel}・{formatCharacterId(opponent.character)}
        </span>
      </div>
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
