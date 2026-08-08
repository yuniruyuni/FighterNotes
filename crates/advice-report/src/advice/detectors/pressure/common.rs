use crate::advice::{
    MIN_DECISION_BIAS_LOSSES, MIN_DECISION_BIAS_OPPORTUNITIES, MIN_DECISION_BIAS_PERCENT,
    MIN_DECISION_BIAS_SELECTIONS,
};

/// 読み合いを含む回答を「偏り」と呼ぶ共通条件。
///
/// 機会数・同一回答数・その回答での損失数・選択率のすべてが揃った場合だけ
/// 原因診断へ上げ、単発の読み負けを癖とは扱わない。
pub(crate) fn is_biased(opportunities: usize, selections: usize, losses: usize) -> bool {
    opportunities >= MIN_DECISION_BIAS_OPPORTUNITIES
        && selections >= MIN_DECISION_BIAS_SELECTIONS
        && losses >= MIN_DECISION_BIAS_LOSSES
        && selections * 100 >= opportunities * MIN_DECISION_BIAS_PERCENT
}
