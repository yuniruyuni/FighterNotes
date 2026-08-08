//! イベント層から先（助言・空間再評価）まで通して確かめるテスト。
//!
//! イベントの組み立て自体は `match-event-layer` 側で検査している。ここに
//! 置くのは、組み上がったイベントが助言や空間候補としてどう扱われるかまで
//! 見ないと意味を持たないものだけ。観測列の組み立ては
//! `match_event_layer::test_support` を共有する。

mod event_to_report {
    mod strike_whiff_accepts_stable_mid_but_rejects_far;
    mod test_ground_attack_chain_does_not_confirm_takeoff;
    mod test_jump_obscured_hp_contact;
    mod test_old_movement_run_does_not_confirm_new_takeoff;
    mod three_round_match_maps_winners_to_p2_report;
}
