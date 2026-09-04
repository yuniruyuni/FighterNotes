//! Player identity signatures across candidate windows.
//!
//! 候補 window は互いに独立で、初期割り当ては「左=P1」を仮定する。だが
//! 短い window はめくりで側が入れ替わった状態から始まることがあり、その
//! 場合は以降の距離・向き・到達判定がすべて逆のプレイヤーに付く。
//!
//! Round 開始直後は側の入れ替わりが物理的に起きない。その確定期間に
//! 各プレイヤーのモーション領域の平均色を学習して window を跨いで保持し、
//! 確定情報のない window の初期化で「左=P1」仮定と照合する。色の対応が
//! 明確に逆を示すときだけ割り当てを反転する。

/// 学習した平均色を 1 標本ごとにどれだけ寄せるか。
const SIGNATURE_BLEND: f32 = 0.125;

#[derive(Default)]
pub(super) struct PlayerSignatures {
    colors: [Option<[f32; 3]>; 2],
}

impl PlayerSignatures {
    /// 側が確定しているフレームの観測で学習する。
    pub(super) fn learn(&mut self, side_index: usize, color: [f32; 3]) {
        match &mut self.colors[side_index] {
            Some(current) => {
                for channel in 0..3 {
                    current[channel] += (color[channel] - current[channel]) * SIGNATURE_BLEND;
                }
            }
            None => self.colors[side_index] = Some(color),
        }
    }

    /// 両プレイヤーぶん学習できているときだけ照合に使える。
    pub(super) fn pair(&self) -> Option<[[f32; 3]; 2]> {
        match (self.colors[0], self.colors[1]) {
            (Some(p1), Some(p2)) => Some([p1, p2]),
            _ => None,
        }
    }
}

/// 平均色どうしの距離。チャネルごとの絶対差の和。
pub(super) fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_need_both_players_before_pairing() {
        let mut signatures = PlayerSignatures::default();
        assert!(signatures.pair().is_none());
        signatures.learn(0, [70.0, 130.0, 205.0]);
        assert!(signatures.pair().is_none());
        signatures.learn(1, [205.0, 110.0, 60.0]);
        assert_eq!(
            signatures.pair(),
            Some([[70.0, 130.0, 205.0], [205.0, 110.0, 60.0]])
        );
    }

    #[test]
    fn learning_blends_toward_the_new_sample() {
        let mut signatures = PlayerSignatures::default();
        signatures.learn(0, [80.0, 80.0, 80.0]);
        signatures.learn(0, [160.0, 80.0, 0.0]);
        // 80 + (160-80)/8 = 90、80 + 0 = 80、80 - 10 = 70。
        assert_eq!(signatures.pair(), None);
        signatures.learn(1, [0.0, 0.0, 0.0]);
        let pair = signatures.pair().unwrap();
        assert_eq!(pair[0], [90.0, 80.0, 70.0]);
    }

    #[test]
    fn color_distance_sums_absolute_channel_differences() {
        assert_eq!(color_distance([10.0, 20.0, 30.0], [15.0, 5.0, 30.0]), 20.0);
        // 3 チャネルすべてが独立に効く。
        assert_eq!(color_distance([1.0, 2.0, 3.0], [3.0, 1.0, 1.0]), 5.0);
        assert_eq!(color_distance([0.0, 0.0, 7.0], [0.0, 0.0, 0.0]), 7.0);
        assert_eq!(color_distance([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]), 0.0);
    }
}
