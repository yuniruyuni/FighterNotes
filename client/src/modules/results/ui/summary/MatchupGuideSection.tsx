import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { formatCharacterId } from "~/modules/analysis/domain/character.js";
import { selectMatchupGuide } from "~/modules/results/domain/matchup-guide.js";
import { MATCHUP_GUIDE_VERSION, MATCHUP_GUIDES } from "./matchup-guide-data.js";

/**
 * キャラ対の手引き。人手で書かれた知識のうち、この試合で実際に観測された
 * 場面に対応する項目だけを表示する。統計・指摘カードとは別枠。
 */
export function MatchupGuideSection({
  report,
  context,
}: {
  report: AdviceReport;
  context: AnalysisContext;
}) {
  const ownIsP2 = context.ownSide === "p2";
  const opponentCharacter = ownIsP2
    ? context.p1.character
    : context.p2.character;
  const items = selectMatchupGuide(
    MATCHUP_GUIDES,
    report,
    opponentCharacter ?? "",
  );
  if (items.length === 0) return null;
  return (
    <section className="summary-section" data-wm="MatchupGuide">
      <h2>キャラ対の手引き({formatCharacterId(opponentCharacter)})</h2>
      <p className="muted-note">
        人手で書かれた手引きのうち、この試合で観測された場面に対応する項目
        だけを表示しています。解析結果ではありません(確認時点:{" "}
        {MATCHUP_GUIDE_VERSION})。
      </p>
      <ul>
        {items.map((item) => (
          <li key={item.id}>{item.text}</li>
        ))}
      </ul>
    </section>
  );
}
