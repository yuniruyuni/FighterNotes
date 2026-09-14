import type { AdviceReport } from "~/modules/analysis/contracts.js";

/**
 * キャラ対の手引き 1 項目。内容は人手で書かれた知識であり、解析結果では
 * ない。表示は「その試合で実際に観測された場面」に紐づく項目だけに絞り、
 * 観測ゼロの知識は押し売りしない。
 */
export interface MatchupGuideItem {
  id: string;
  /** どれか 1 つでも観測されていれば表示する(OR)。 */
  triggers: readonly GuideTrigger[];
  text: string;
}

/** 手引きの表示条件。レポートの観測事実にだけ紐づける。 */
export type GuideTrigger =
  /** 同定された相手の技(opponent_move_stats)の記譜。 */
  | { kind: "move"; notations: readonly string[] }
  /** tactic_stats の遭遇数が 1 以上。 */
  | { kind: "stat"; stat: GuideStatKey }
  /** 指摘カードが出ている。 */
  | { kind: "card"; id: string };

/** 手引きの引き金に使える遭遇数。 */
export type GuideStatKey =
  | "jump_ins_allowed"
  | "raw_drive_rushes_faced"
  | "throws_taken"
  | "di_faced";

export type MatchupGuideCatalog = Readonly<
  Record<string, readonly MatchupGuideItem[]>
>;

function triggerObserved(report: AdviceReport, trigger: GuideTrigger): boolean {
  switch (trigger.kind) {
    case "move":
      // 旧レポートは opponent_move_stats を持たないので観測なしとして扱う。
      return trigger.notations.some(
        (notation) =>
          report.opponent_move_stats?.some(
            (move) => move.name === notation && move.touches >= 1,
          ) === true,
      );
    case "stat":
      return (report.tactic_stats[trigger.stat] ?? 0) >= 1;
    case "card":
      return report.cards.some((card) => card.id === trigger.id);
  }
}

/**
 * 相手キャラの手引きから、この試合で観測された場面に対応する項目だけを
 * カタログの記載順で返す。手引きの無いキャラ(未指定の空文字を含む)は
 * カタログの参照が外れて空になる。
 */
export function selectMatchupGuide(
  catalog: MatchupGuideCatalog,
  report: AdviceReport,
  opponentCharacter: string,
): MatchupGuideItem[] {
  const items = catalog[opponentCharacter];
  if (items === undefined) return [];
  return items.filter((item) =>
    item.triggers.some((trigger) => triggerObserved(report, trigger)),
  );
}
