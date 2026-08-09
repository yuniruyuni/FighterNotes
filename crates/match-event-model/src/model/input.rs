/// 同一入力の継続区間（確定層トラッカー出力のセグメント化）。
///
/// `repaired_frames` は前後の観測から補間されたフレーム数。入力そのものを
/// 根拠に因果診断を出す場合は、少なくとも 1 フレームの直接観測を要求する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InputEvidence {
    pub observed_frames: u32,
    pub repaired_frames: u32,
}

impl Default for InputEvidence {
    fn default() -> Self {
        // ruleset v3 以前の JSON / テスト用リテラルは「観測済み」として読む。
        // 新しい解析結果では build_segments が実数を必ず設定する。
        Self {
            observed_frames: 1,
            repaired_frames: 0,
        }
    }
}

impl InputEvidence {
    pub fn has_direct_observation(self) -> bool {
        self.observed_frames > 0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputSegment {
    pub start_frame: u32,
    pub end_frame: u32,
    pub dir: String,
    /// バッジラベル（"弱", "中", "強", "SP", "DP", "DI"）
    pub badges: Vec<String>,
    pub auto: bool,
    pub throw: bool,
    #[serde(default)]
    pub evidence: InputEvidence,
}

impl InputSegment {
    /// 攻撃ボタン（円バッジ / AUTO / SP / DP / DI / 投げ）を含むか
    pub fn has_button(&self) -> bool {
        !self.badges.is_empty() || self.auto || self.throw
    }

    pub fn is_drive_impact(&self) -> bool {
        self.badges.iter().any(|badge| badge == "DI")
            || (self.badges.iter().any(|badge| badge == "強P")
                && self.badges.iter().any(|badge| badge == "強K"))
    }
}
