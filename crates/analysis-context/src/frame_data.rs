//! キャラクター別フレームデータ（確定反撃の選択肢提案用）。
//!
//! `data/frame_data.json` は中立から出せる地上技の確反候補、
//! `data/attack_data.json` は Classic / Modern 入力と上中下・空中属性の
//! 照合に使う。公開 repository には解析時に必要な正規化済み field だけを置き、
//! schema、件数、checksum は `data/manifest.json` と offline validator で固定する。
//! キャラ名はユーザー入力（UI のセレクト）で受け取る。

use std::collections::HashMap;
use std::sync::OnceLock;

/// 1 技ぶんのフレームデータ（確反提案に必要な最小情報）。
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveData {
    pub name: String,
    pub startup: u32,
    pub damage: u32,
    /// 公式ページのセクション由来: normal / unique / special / super /
    /// throw / common
    pub category: String,
}

/// 打撃のガード属性。公式表の「上 / 中 / 下」と空中技区分に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrikeKind {
    High,
    Overhead,
    Low,
    Air,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AttackInputDirection {
    Any,
    Standing,
    Neutral,
    Down,
    Horizontal,
    DownDiagonal,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackInputPattern {
    direction: AttackInputDirection,
    buttons: Vec<String>,
    auto: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackMoveData {
    startup: u32,
    kind: StrikeKind,
    classic_inputs: Vec<AttackInputPattern>,
    modern_inputs: Vec<AttackInputPattern>,
}

fn table() -> &'static HashMap<String, Vec<MoveData>> {
    static TABLE: OnceLock<HashMap<String, Vec<MoveData>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../data/frame_data.json"))
            .expect("frame_data.json のパースに失敗")
    })
}

fn attack_table() -> &'static HashMap<String, Vec<AttackMoveData>> {
    static TABLE: OnceLock<HashMap<String, Vec<AttackMoveData>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../data/attack_data.json"))
            .expect("attack_data.json のパースに失敗")
    })
}

/// 入力履歴と実測発生フレームから打撃属性を推定する。
///
/// 最短候補だけに絞っても属性が複数残る場合は、誤分類を避けるため None。
pub fn strike_kind_for_input(
    character: &str,
    dir: &str,
    badges: &[String],
    auto: bool,
    airborne: bool,
    observed_startup: Option<u32>,
) -> Option<StrikeKind> {
    let moves = attack_table().get(&character.to_uppercase())?;
    let classic = input_is_classic(badges)?;
    let mut candidates: Vec<(&AttackMoveData, u32)> = moves
        .iter()
        .filter(|move_data| (move_data.kind == StrikeKind::Air) == airborne)
        .filter_map(|move_data| {
            let patterns = if classic {
                &move_data.classic_inputs
            } else {
                &move_data.modern_inputs
            };
            patterns
                .iter()
                .any(|pattern| input_matches(pattern, dir, badges, auto))
                .then(|| {
                    let error =
                        observed_startup.map_or(0, |startup| startup.abs_diff(move_data.startup));
                    (move_data, error)
                })
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }

    if observed_startup.is_some() {
        let best_error = candidates.iter().map(|(_, error)| *error).min()?;
        const STARTUP_TOLERANCE: u32 = 3;
        if best_error > STARTUP_TOLERANCE {
            return None;
        }
        candidates.retain(|(_, error)| *error == best_error);
    }

    let kind = candidates.first()?.0.kind;
    candidates
        .iter()
        .all(|(move_data, _)| move_data.kind == kind)
        .then_some(kind)
}

fn input_is_classic(badges: &[String]) -> Option<bool> {
    if badges.is_empty() {
        return None;
    }
    if badges.iter().all(|badge| {
        matches!(
            badge.as_str(),
            "弱P" | "中P" | "強P" | "弱K" | "中K" | "強K"
        )
    }) {
        return Some(true);
    }
    badges
        .iter()
        .all(|badge| matches!(badge.as_str(), "弱" | "中" | "強" | "SP"))
        .then_some(false)
}

fn input_matches(pattern: &AttackInputPattern, dir: &str, badges: &[String], auto: bool) -> bool {
    pattern.auto == auto
        && direction_matches(pattern.direction, dir)
        && pattern.buttons.len() == badges.len()
        && pattern
            .buttons
            .iter()
            .all(|button| badges.iter().any(|badge| badge == button))
}

fn direction_matches(expected: AttackInputDirection, actual: &str) -> bool {
    match expected {
        AttackInputDirection::Any => actual != "?",
        AttackInputDirection::Standing => matches!(actual, "N" | "L" | "R"),
        AttackInputDirection::Neutral => actual == "N",
        AttackInputDirection::Down => matches!(actual, "D" | "DL" | "DR"),
        AttackInputDirection::Horizontal => matches!(actual, "L" | "R"),
        AttackInputDirection::DownDiagonal => matches!(actual, "DL" | "DR"),
    }
}

/// 収録キャラクター名の一覧（UI のセレクト用）。
pub fn character_names() -> Vec<&'static str> {
    let mut v: Vec<&str> = table().keys().map(|s| s.as_str()).collect();
    v.sort_unstable();
    v
}

/// 有利フレーム `advantage` で確定する技の候補（ダメージ降順、最大 `limit` 件）。
///
/// 同一発生の派生違い等で重複が多いため、発生フレームごとに最大ダメージの
/// 技だけを残してから上位を返す。キャラ名が不明なら空。
pub fn punish_options(character: &str, advantage: u32, limit: usize) -> Vec<&'static MoveData> {
    let Some(moves) = table().get(&character.to_uppercase()) else {
        return Vec::new();
    };
    // 発生ごとの最大ダメージ技。スーパーアーツはゲージ前提なので
    // 汎用提案からは外す（コマンドは 236236 系とは限らないためカテゴリで判定）
    let mut best_by_startup: HashMap<u32, &MoveData> = HashMap::new();
    for m in moves {
        if m.startup == 0 || m.startup > advantage || m.damage == 0 {
            continue;
        }
        if m.category == "super" {
            continue;
        }
        let e = best_by_startup.entry(m.startup).or_insert(m);
        if m.damage > e.damage {
            *e = m;
        }
    }
    let mut v: Vec<&MoveData> = best_by_startup.into_values().collect();
    v.sort_by(|a, b| b.damage.cmp(&a.damage).then(a.startup.cmp(&b.startup)));
    v.truncate(limit);
    v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RisingReversalKind {
    Motion,
    Charge,
}

/// Whether a character can remain airborne beyond the normal jump window.
///
/// The frame meter can show one long motion run for air stalls and floats. That
/// run is only allowed to extend jump attribution for explicitly calibrated
/// characters; applying it to every character merges adjacent ground actions.
pub fn has_extended_airtime(character: Option<&str>) -> bool {
    character.is_some_and(|name| name.eq_ignore_ascii_case("DHALSIM"))
}

/// Which OD rising-special command family exists in the official move table.
///
/// This intentionally recognizes command families rather than guessing from
/// a character name. Spatial advice still requires an overlap-distance visual
/// observation; this capability check alone never emits advice.
pub fn rising_reversal_kind(character: &str) -> Option<RisingReversalKind> {
    let moves = table().get(&character.to_uppercase())?;
    if moves.iter().any(|move_data| {
        if move_data.category != "special" {
            return false;
        }
        let command = move_data.name.as_str();
        let od = command.contains("PP") || command.contains("KK");
        command.starts_with("623") && od
    }) {
        return Some(RisingReversalKind::Motion);
    }
    moves
        .iter()
        .any(|move_data| {
            if move_data.category != "special" {
                return false;
            }
            let command = move_data.name.as_str();
            let od = command.contains("PP") || command.contains("KK");
            command.starts_with("[2]8") && od
        })
        .then_some(RisingReversalKind::Charge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_data_loads() {
        let names = character_names();
        assert!(names.len() >= 31, "{names:?}");
        assert!(names.contains(&"KEN"));
        assert!(names.contains(&"INGRID"));
        assert!(names.contains(&"YASMINE"));
    }

    #[test]
    fn embedded_data_schema_rejects_unused_fields() {
        let move_with_extra_field =
            r#"{"name":"LP","startup":4,"damage":300,"category":"normal","id":"100"}"#;
        assert!(serde_json::from_str::<MoveData>(move_with_extra_field).is_err());

        let attack_with_extra_field = r#"{
            "startup": 4,
            "kind": "high",
            "classic_inputs": [],
            "modern_inputs": [],
            "name": "unused"
        }"#;
        assert!(serde_json::from_str::<AttackMoveData>(attack_with_extra_field).is_err());
    }

    #[test]
    fn official_attack_data_matches_modern_and_classic_inputs() {
        assert_eq!(
            strike_kind_for_input("INGRID", "D", &["弱".to_string()], false, false, Some(5),),
            Some(StrikeKind::Low)
        );
        assert_eq!(
            strike_kind_for_input("INGRID", "R", &["中".to_string()], false, false, Some(21),),
            Some(StrikeKind::Overhead)
        );
        assert_eq!(
            strike_kind_for_input("INGRID", "N", &["弱".to_string()], true, false, Some(4),),
            Some(StrikeKind::High)
        );
        assert_eq!(
            strike_kind_for_input("INGRID", "DL", &["弱K".to_string()], false, false, Some(5),),
            Some(StrikeKind::Low)
        );
        assert_eq!(
            strike_kind_for_input("YASMINE", "D", &["弱K".to_string()], false, false, Some(5),),
            Some(StrikeKind::Low)
        );
    }

    #[test]
    fn attack_matching_keeps_ambiguous_or_unobserved_inputs_unknown() {
        assert_eq!(
            strike_kind_for_input("INGRID", "N", &["弱".to_string()], true, true, Some(6),),
            Some(StrikeKind::Air)
        );
        assert_eq!(
            strike_kind_for_input("UNKNOWN", "D", &["弱".to_string()], false, false, Some(5),),
            None
        );
        assert_eq!(
            strike_kind_for_input("INGRID", "D", &["DI".to_string()], false, false, Some(5),),
            None
        );
    }

    #[test]
    fn test_ingrid_punish_options() {
        // イングリッドは発生 4F の弱P があるため有利 7F で候補が出るはず
        let opts = punish_options("INGRID", 7, 3);
        assert!(!opts.is_empty());
        for m in &opts {
            assert!(m.startup <= 7 && m.damage > 0, "{m:?}");
        }
    }

    #[test]
    fn yasmine_punish_options_exclude_automatic_follow_up_hits() {
        let opts = punish_options("YASMINE", 60, 100);
        assert!(!opts.is_empty());
        assert!(
            opts.iter()
                .all(|move_data| !move_data.name.contains("Alon(2)")),
            "{opts:?}"
        );
    }

    #[test]
    fn test_super_arts_excluded() {
        // SA はゲージ前提なので提案から外す。コマンドが 236236 系でない SA
        // （ザンギエフの 720P 等）もカテゴリで除外されること
        for opt in punish_options("ZANGIEF", 10, 10) {
            assert_ne!(opt.name, "720P", "{opt:?}");
        }
        for opt in punish_options("GUILE", 10, 10) {
            assert!(!opt.name.starts_with("[4]646"), "{opt:?}");
        }
    }

    #[test]
    fn test_punish_options_respects_advantage() {
        let opts = punish_options("KEN", 7, 3);
        assert!(!opts.is_empty());
        for m in &opts {
            assert!(m.startup <= 7, "{m:?}");
        }
        // ダメージ降順
        for w in opts.windows(2) {
            assert!(w[0].damage >= w[1].damage);
        }
        // 不明キャラは空
        assert!(punish_options("UNKNOWN", 7, 3).is_empty());
    }

    #[test]
    fn test_rising_reversal_capability_comes_from_move_commands() {
        assert_eq!(
            rising_reversal_kind("BLANKA"),
            Some(RisingReversalKind::Charge)
        );
        assert_eq!(
            rising_reversal_kind("KEN"),
            Some(RisingReversalKind::Motion)
        );
        assert_eq!(
            rising_reversal_kind("YASMINE"),
            Some(RisingReversalKind::Motion)
        );
        assert_eq!(rising_reversal_kind("ZANGIEF"), None);
        assert_eq!(rising_reversal_kind("UNKNOWN"), None);
    }

    #[test]
    fn extended_airtime_is_an_explicit_character_capability() {
        assert!(has_extended_airtime(Some("DHALSIM")));
        assert!(has_extended_airtime(Some("dhalsim")));
        assert!(!has_extended_airtime(Some("LUKE")));
        assert!(!has_extended_airtime(None));
    }

    // ── 発生フレームによる絞り込み ───────────────────────────────────────

    /// 観測した発生が表のどの技とも合わなければ、その入力からは
    /// 何も決めない。
    #[test]
    fn an_observed_startup_far_from_the_table_matches_nothing() {
        let low = ["弱".to_string()];

        assert_eq!(
            strike_kind_for_input("INGRID", "D", &low, false, false, Some(5)),
            Some(StrikeKind::Low),
            "表どおりの発生を落としている"
        );
        assert_eq!(
            strike_kind_for_input("INGRID", "D", &low, false, false, Some(50)),
            None,
            "かけ離れた発生を受け入れている"
        );
    }

    /// 発生が観測できていなければ、同じ入力に属性の違う技が並ぶ限り
    /// 決められない。断定より分からないままの方がよい。
    #[test]
    fn without_an_observed_startup_an_ambiguous_input_stays_unknown() {
        assert_eq!(
            strike_kind_for_input("INGRID", "D", &["弱".to_string()], false, false, None),
            None
        );
    }

    /// ボタンの見えない入力からは技を決めない。
    #[test]
    fn an_input_without_buttons_decides_nothing() {
        assert_eq!(
            strike_kind_for_input("INGRID", "D", &[], false, false, Some(5)),
            None
        );
    }

    /// クラシックとモダンのバッジが混ざった入力は、どちらの表とも
    /// 突き合わせない。読み取りが崩れている。
    #[test]
    fn a_mixture_of_classic_and_modern_badges_decides_nothing() {
        let mixed = ["弱P".to_string(), "中".to_string()];

        assert_eq!(
            strike_kind_for_input("INGRID", "D", &mixed, false, false, Some(5)),
            None
        );
    }

    /// 地上と空中は別の表を見る。地上の入力を空中技にしない。
    #[test]
    fn the_ground_and_air_tables_are_kept_apart() {
        let light = ["弱".to_string()];
        let ground = strike_kind_for_input("INGRID", "N", &light, true, false, Some(4));
        let air = strike_kind_for_input("INGRID", "N", &light, true, true, Some(6));

        assert_eq!(ground, Some(StrikeKind::High));
        assert_eq!(air, Some(StrikeKind::Air));
    }

    // ── 確定反撃の候補 ───────────────────────────────────────────────────

    /// 有利フレームぴったりで出る技は候補に入れる。1F でも足りなければ
    /// 入れない。
    #[test]
    fn a_move_that_fits_the_advantage_exactly_is_offered() {
        let fastest = punish_options("KEN", 20, 100)
            .iter()
            .map(|move_data| move_data.startup)
            .min()
            .expect("候補がある");

        assert!(
            !punish_options("KEN", fastest, 100).is_empty(),
            "ちょうどの技を外している"
        );
        assert!(
            punish_options("KEN", fastest - 1, 100).is_empty(),
            "間に合わない技を勧めている"
        );
    }

    /// 件数の上限を守る。長い一覧は読めない。
    #[test]
    fn the_offered_list_is_capped() {
        assert_eq!(punish_options("KEN", 60, 3).len(), 3);
        assert!(punish_options("KEN", 60, 0).is_empty());
    }

    /// 同じ発生の技は、一番ダメージの高いものだけを残す。派生違いで
    /// 同じ発生の技が並ぶと、一覧が埋まって読めない。
    #[test]
    fn only_the_strongest_move_of_each_startup_is_offered() {
        let options = punish_options("KEN", 60, 100);

        let mut startups: Vec<u32> = options.iter().map(|move_data| move_data.startup).collect();
        startups.sort_unstable();
        let unique = startups.len();
        startups.dedup();

        assert_eq!(startups.len(), unique, "同じ発生の技が並んでいる");
    }

    /// 同じ発生に技が並ぶなら、勧めるのはダメージの一番高いもの。
    #[test]
    fn the_offered_move_is_the_strongest_of_its_startup() {
        let moves = table().get("KEN").expect("KEN の技表");
        let offered = punish_options("KEN", 60, 100);

        for move_data in &offered {
            let strongest = moves
                .iter()
                .filter(|candidate| {
                    candidate.startup == move_data.startup
                        && candidate.category != "super"
                        && candidate.damage > 0
                })
                .map(|candidate| candidate.damage)
                .max()
                .expect("同じ発生の技がある");
            assert_eq!(
                move_data.damage, strongest,
                "発生 {}F で弱い方を勧めている",
                move_data.startup
            );
        }
    }

    /// 使えない技が一つ混ざっても、その先の技まで諦めない。
    #[test]
    fn an_unusable_move_does_not_end_the_search() {
        let moves = table().get("KEN").expect("KEN の技表");
        let unusable = moves
            .iter()
            .position(|move_data| move_data.startup == 0 || move_data.damage == 0)
            .expect("使えない技が表にある");
        let usable_after = moves[unusable + 1..].iter().any(|move_data| {
            move_data.startup > 0 && move_data.startup <= 60 && move_data.damage > 0
        });

        assert!(usable_after, "この表では確かめられない");
        assert!(
            punish_options("KEN", 60, 100).len() > 1,
            "途中で探すのをやめている"
        );
    }

    /// ダメージの分からない技は勧めない。
    #[test]
    fn moves_without_damage_are_not_offered() {
        assert!(punish_options("KEN", 60, 100)
            .iter()
            .all(|move_data| move_data.damage > 0));
    }
}
