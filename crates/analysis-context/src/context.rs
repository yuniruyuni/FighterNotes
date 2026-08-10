//! Match metadata that is independent from per-frame perception.

/// Metadata for one player slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerContext {
    /// Character key used by frame data and character-specific detectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    /// Reserved for `classic`, `modern`, or future control schemes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_type: Option<String>,
}

impl PlayerContext {
    pub fn with_character(character: &str) -> Self {
        Self {
            character: non_empty(character),
            control_type: None,
        }
    }

    fn normalize(&mut self) {
        self.character = self.character.take().and_then(|v| non_empty(&v));
        self.control_type = self.control_type.take().and_then(|v| non_empty(&v));
    }
}

/// Session-wide metadata passed through the pipeline alongside frame features.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AnalysisContext {
    /// User-controlled side. Invalid values are normalized to `p1`.
    pub own_side: String,
    pub p1: PlayerContext,
    pub p2: PlayerContext,
    /// SF6 battle version used to produce the replay, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battle_version: Option<String>,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new("p1")
    }
}

impl AnalysisContext {
    pub fn new(own_side: &str) -> Self {
        Self {
            own_side: normalize_side(own_side).to_string(),
            p1: PlayerContext::default(),
            p2: PlayerContext::default(),
            battle_version: None,
        }
    }

    /// Builds context from the legacy own/opponent character arguments.
    pub fn from_characters(
        own_side: &str,
        own_char: Option<&str>,
        opponent_char: Option<&str>,
    ) -> Self {
        let mut context = Self::new(own_side);
        context.set_characters(own_char.unwrap_or(""), opponent_char.unwrap_or(""));
        context
    }

    pub fn own_side(&self) -> &str {
        normalize_side(&self.own_side)
    }

    pub fn own_player(&self) -> &PlayerContext {
        if self.own_side() == "p2" {
            &self.p2
        } else {
            &self.p1
        }
    }

    pub fn opponent_player(&self) -> &PlayerContext {
        if self.own_side() == "p2" {
            &self.p1
        } else {
            &self.p2
        }
    }

    pub fn own_character(&self) -> Option<&str> {
        self.own_player().character.as_deref()
    }

    pub fn opponent_character(&self) -> Option<&str> {
        self.opponent_player().character.as_deref()
    }

    pub fn player(&self, side: u8) -> &PlayerContext {
        if side == 2 {
            &self.p2
        } else {
            &self.p1
        }
    }

    /// Updates character metadata using the legacy own/opponent convention.
    pub fn set_characters(&mut self, own_char: &str, opponent_char: &str) {
        let (own, opponent) = if self.own_side() == "p2" {
            (&mut self.p2, &mut self.p1)
        } else {
            (&mut self.p1, &mut self.p2)
        };
        own.character = non_empty(own_char);
        opponent.character = non_empty(opponent_char);
    }

    /// Normalizes user-provided JSON while keeping the analyzer's side authoritative.
    pub fn normalize_for_side(&mut self, own_side: &str) {
        self.own_side = normalize_side(own_side).to_string();
        self.p1.normalize();
        self.p2.normalize();
        self.battle_version = self.battle_version.take().and_then(|v| non_empty(&v));
    }
}

fn normalize_side(side: &str) -> &'static str {
    if side.eq_ignore_ascii_case("p2") {
        "p2"
    } else {
        "p1"
    }
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_characters_are_mapped_to_player_slots() {
        let p1 = AnalysisContext::from_characters("p1", Some("KEN"), Some("DHALSIM"));
        assert_eq!(p1.p1.character.as_deref(), Some("KEN"));
        assert_eq!(p1.p2.character.as_deref(), Some("DHALSIM"));
        assert_eq!(p1.own_character(), Some("KEN"));
        assert_eq!(p1.opponent_character(), Some("DHALSIM"));

        let p2 = AnalysisContext::from_characters("p2", Some("BLANKA"), Some("DHALSIM"));
        assert_eq!(p2.p1.character.as_deref(), Some("DHALSIM"));
        assert_eq!(p2.p2.character.as_deref(), Some("BLANKA"));
        assert_eq!(p2.own_character(), Some("BLANKA"));
        assert_eq!(p2.opponent_character(), Some("DHALSIM"));
    }

    #[test]
    fn json_shape_keeps_future_metadata() {
        let json = r#"{
            "ownSide":"p2",
            "p1":{"character":"DHALSIM","controlType":"classic"},
            "p2":{"character":"BLANKA","controlType":"modern"},
            "battleVersion":"2026.06"
        }"#;
        let context: AnalysisContext = serde_json::from_str(json).unwrap();
        assert_eq!(context.player(1).character.as_deref(), Some("DHALSIM"));
        assert_eq!(context.player(2).control_type.as_deref(), Some("modern"));
        assert_eq!(context.battle_version.as_deref(), Some("2026.06"));
    }

    #[test]
    fn normalization_discards_blank_metadata_and_invalid_side() {
        let mut context: AnalysisContext = serde_json::from_str(
            r#"{"ownSide":"other","p1":{"character":"  "},"p2":{},"battleVersion":" "}"#,
        )
        .unwrap();
        context.normalize_for_side("other");
        assert_eq!(context.own_side(), "p1");
        assert_eq!(context.p1.character, None);
        assert_eq!(context.battle_version, None);
    }

    // ── 値の正規化 ───────────────────────────────────────────────────────

    /// 空白だけの名前は、入っていないのと同じ。UI の空欄がそのまま
    /// キャラクター名として下へ流れると、表引きが必ず外れる。
    #[test]
    fn blank_names_are_treated_as_absent() {
        let mut context = AnalysisContext {
            own_side: "p1".to_string(),
            p1: PlayerContext {
                character: Some("  ".to_string()),
                control_type: Some(String::new()),
            },
            p2: PlayerContext {
                character: Some("  KEN  ".to_string()),
                control_type: Some(" modern ".to_string()),
            },
            battle_version: Some("   ".to_string()),
        };

        context.normalize_for_side("p1");

        assert_eq!(context.p1.character, None, "空白を名前として残している");
        assert_eq!(context.p1.control_type, None);
        assert_eq!(
            context.p2.character.as_deref(),
            Some("KEN"),
            "前後の空白を残している"
        );
        assert_eq!(context.p2.control_type.as_deref(), Some("modern"));
        assert_eq!(context.battle_version, None);
    }

    /// 中身のある版数は残す。空白だけを落とす。
    #[test]
    fn a_real_battle_version_survives_normalization() {
        let mut context = AnalysisContext::new("p1");
        context.battle_version = Some("  2026-07  ".to_string());

        context.normalize_for_side("p1");

        assert_eq!(context.battle_version.as_deref(), Some("2026-07"));
    }

    /// 側の指定は解析器のものが優先する。JSON に入っていた値では
    /// 上書きしない。
    #[test]
    fn the_analyzer_side_wins_over_whatever_the_json_carried() {
        let mut context = AnalysisContext::new("p1");
        context.own_side = "p9".to_string();

        context.normalize_for_side("p2");

        assert_eq!(context.own_side, "p2");
    }

    /// 側の綴りは大文字小文字を問わない。それ以外は P1 として扱う。
    #[test]
    fn the_side_spelling_is_forgiving_but_defaults_to_p1() {
        for spelling in ["p2", "P2", "P2"] {
            assert_eq!(AnalysisContext::new(spelling).own_side, "p2", "{spelling}");
        }
        for spelling in ["p1", "P1", "", "left", "2p"] {
            assert_eq!(AnalysisContext::new(spelling).own_side, "p1", "{spelling}");
        }
    }

    /// 名前だけを渡す入口でも、空白は落とす。
    #[test]
    fn a_player_built_from_a_blank_name_has_no_character() {
        assert_eq!(PlayerContext::with_character("  ").character, None);
        assert_eq!(
            PlayerContext::with_character(" KEN ").character.as_deref(),
            Some("KEN")
        );
    }

    /// 既定の文脈は P1 視点。
    #[test]
    fn the_default_context_is_seen_from_p1() {
        let context = AnalysisContext::default();

        assert_eq!(context.own_side, "p1");
        assert_eq!(context.p1, PlayerContext::default());
        assert_eq!(context.p2, PlayerContext::default());
        assert_eq!(context.battle_version, None);
    }
}
