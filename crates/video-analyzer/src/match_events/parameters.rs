/// ラウンド開始とみなす「両者ほぼ全快」の閾値と最小持続フレーム
/// （確定層 temporal::confirm_hp の単調リセットと同一値）
pub(crate) use crate::temporal::{FULL_HP, FULL_MIN_RUN};
/// 全快区間同士をマージする条件: 間の最小 HP がこれ以上なら同一ラウンド前区間
/// （遮蔽による全快検出の途切れであって、実ダメージではない）
pub(crate) const MERGE_MIN_HP: f32 = 0.85;
/// KO とみなす HP と持続フレーム
pub(crate) const KO_HP: f32 = 0.02;
pub(crate) const KO_MIN_RUN: usize = 15;
/// ダメージシーケンスのグループ化ギャップ（コンボのヒット間隔上限）
pub(crate) const DMG_GAP: usize = 45;
/// SA 暗転をまたぐ場合だけ許す前後の実ゲーム時間。HP 表示の減少完了から
/// キャンセル演出開始までの遅れを含む（検証済み SA 例では実測 48F）。
pub(crate) const DMG_GAP_ACROSS_FREEZE: u32 = 49;
/// 1 減少ステップの最小量（読み取りノイズ除去）
pub(crate) const DMG_EPS: f32 = 0.0015;
/// シーケンスとして採用する最小合計ドロップ
pub(crate) const DMG_MIN_DROP: f32 = 0.015;
// ジャンプ仕様（https://note.com/kiyotea/n/n8ba32418e034 実測）:
//   予備動作 地上 4F → 空中 38F（攻撃時 39F）→ 着地硬直 3F。
//   空ジャンプ着地は 1F 目からガード可能。攻撃後の着地硬直は完全無防備。
// HP ベースの窓（フォールバック用）は HP 減少検出の遅延 +3F を含む。
// advice 層のダメージ帰属排他（暴れカードとの重複防止）でも同じ窓を使う。
/// 対空 + 着地狩りの被弾窓上限（空中 〜f+42、着地硬直 〜f+45、+遅延 3F）
pub(crate) const JUMP_SELF_HIT_WINDOW: u32 = 48;
/// GotHit は入力からこのフレーム数以降の被弾のみ。これ未満は予備動作狩られ
pub(crate) const JUMP_SELF_HIT_MIN: u32 = 8;
/// ジャンプ攻撃ヒットの窓（最速の昇り攻撃 ≈f+9、最遅 f+42 + 遅延 3F）。
/// これより後の相手被弾は着地後の地上技（実測 f1717: +51F の中足）であり
/// 「飛び込みが通った」ではない
pub(crate) const JUMP_ATTACK_MIN: u32 = 9;
pub(crate) const JUMP_ATTACK_MAX: u32 = 45;
/// ジャンプとみなす上入力セグメントの最小持続（video frames）。
/// 上入力は 1F で成立する。「3F」はジャンプの予備動作（離地までの時間）で
/// あって入力の保持時間ではない（利用者指摘 2026-07-09: タップジャンプの
/// プレイヤーはジャンプ回数 0 になっていた）。単発の方向誤読は入力
/// トラッカーの窓内多数決補修と uncertain セグメント切断が既に除去している
pub(crate) const JUMP_MIN_HOLD: u32 = 1;
/// 同じ上入力が方向グリフの揺れで短く分割された場合だけを統合する。
/// ジャンプ1サイクル全体を窓にすると、着地際に入力した次のジャンプを消す。
pub(crate) const JUMP_INPUT_FRAGMENT_GAP: u32 = 6;
/// 同じ長いメーターランに複数の上入力が重なった曖昧候補の最小間隔。
/// 物理1サイクル未満の候補は同じジャンプの入力揺れとして統合する。
pub(crate) const JUMP_AMBIGUOUS_REUSE_GAP: u32 = 45;
// コンタクト（メーター）ベースの窓。コンタクトはヒット瞬間そのものなので
// 遅延マージンは ±2F 程度でよい
/// 予備動作狩られの上限（予備 4F + 2F）
pub(crate) const JUMP_C_PRE_MAX: u32 = 6;
/// 対空 + 着地狩り被弾の上限（着地硬直末 f+45 + 2F）
pub(crate) const JUMP_C_HIT_MAX: u32 = 47;
/// ジャンプ攻撃接触の窓（空中のみ。最遅 f+42 + 2F）
pub(crate) const JUMP_C_ATK_MIN: u32 = 7;
pub(crate) const JUMP_C_ATK_MAX: u32 = 44;
/// ヒットストップとみなす最小停止フレーム数（実測 10。通常進行は 1）
/// 最速行動を記録する最小不利幅。-1Fから結果とセットで記録し、
/// 行動自体を一律のミスとは断定しない。
pub(crate) const MINUS_PRESS_THRESHOLD: u32 = 1;
/// 不利ボタン: 指摘対象とする最大不利幅。ガード硬直差はどんな技でも
/// この程度に収まり、これを超える「不利」はダウン・被コンボ由来
/// （ガード後の暴れとは別のシナリオ）なので対象外
pub(crate) const MINUS_PRESS_MAX: u32 = 15;
/// 不利ボタン: 押下からこのフレーム数以内の被弾/接触で結果を分類する
pub(crate) const MINUS_PRESS_OUTCOME_WINDOW: u32 = 30;
/// 不利ボタン: 押下直後このフレーム数以内に Invincible が出たら無敵技
/// （reversal の領分）として除外する
pub(crate) const MINUS_PRESS_INV_WINDOW: usize = 15;
/// 空振り: 攻撃判定の前後で接触を探す猶予。コンタクト抽出の境界ずれと
/// ヒットストップ表示の遅れを吸収する。
pub(crate) const WHIFF_CONTACT_GRACE: u32 = 4;
/// 空振り: 攻撃判定の終了後、反撃を受けたと結び付ける窓。これを超えてから
/// の被弾は、その空振りの硬直を狩られた結果とは扱わない。
pub(crate) const WHIFF_PUNISH_WINDOW: u32 = 40;
/// 攻め継続の判断機会として扱う最小有利フレーム。+1/+2 は最速技でも次の
/// 攻撃を確定させられず、動かないことを「有利を捨てた」とは呼べない。
pub(crate) const ADVANTAGE_THRESHOLD: u32 = 3;
/// 有利側の発生開始を探す猶予。相手が動けるようになるフレームまでに発生が
/// 始まっていれば、有利のうちに攻めたとみなす。入力表示とメーターの
/// 境界ずれを吸収するぶんだけ広げる。
pub(crate) const ADVANTAGE_ACTION_GRACE: usize = 2;
/// 有利を使わなかった場合に、相手が攻撃を始めたかを見る窓。これを超えて
/// から始まった攻撃は、この有利フレームの結果とは扱わない。
pub(crate) const ADVANTAGE_OUTCOME_WINDOW: u32 = 45;
pub(crate) const PAUSE_MIN: i64 = 5;
/// 両者の停止スパンに要求する最小重なり
pub(crate) const PAUSE_OVERLAP_MIN: i64 = 4;
/// 確反機会とみなす最小有利フレーム（これ未満は最速技でも確定しない）
/// 空中ラン終端から着地・接触までのマージン（video frames）。
/// AA ヒットはランを stun で止めるため接触はラン終端の直後に来る
pub(crate) const JUMP_LAND_EPS: u32 = 5;
/// ジャンプ確認: 上入力の近傍でメーターの移動系ラン（緑 counter /
/// シアン motion）を探す窓。開始側 -15 は接触フリーズによる入力表示ラグ
/// （検証済みジャンプ例: 11vf 停止で入力表示がラン開始より遅れる）、
/// 終端側 +8 は離地までの予備動作（f5924 実測: 入力 +4vf でラン開始）
pub(crate) const JUMP_CONFIRM_BACK: u32 = 15;
pub(crate) const JUMP_CONFIRM_FWD: u32 = 8;
/// 移動ランが入力表示より先に見える場合に許すゲーム内フレーム差。
/// 入力履歴の表示はヒットストップ中に遅れることがあるが、その間の game
/// frame は進まない。実ゲーム時間まで進んでいれば、既に実行中の空中化する
/// 必殺技など別行動のランであり、後から表示された上入力の離陸証拠ではない。
pub(crate) const JUMP_CONFIRM_BACK_GF: i64 = 2;
/// ジャンプと認める移動系ランの最小 game frame 長。ジャンプの空中時間は
/// 38-45gf、攻撃の発生（同じ緑表示）はほとんど 4-13gf。短くても直後に
/// 自分が stun になっていれば予備動作狩られ（PreJumpClipped）として認める
pub(crate) const JUMP_CONFIRM_MIN_GF: i64 = 15;

/// 演出フリーズ（SA 暗転・投げ演出）とみなす最小 dwell（video frames）。
/// 通常ヒットストップは ≤20vf 程度、投げ抜け演出 63vf・SA3 ≈100vf（実測）
pub(crate) const FREEZE_MIN_DWELL: i64 = 30;
/// フリーズスパン同士を連結する最大ギャップ（video frames）
pub(crate) const FREEZE_MERGE_GAP: u32 = 10;
/// 被弾コンタクトがフリーズ終端からこの範囲内ならそのフリーズに帰属する
pub(crate) const FREEZE_ATTACH_GAP: u32 = 10;

pub(crate) const PUNISH_MIN_ADV: u32 = 4;
/// 反撃試行の追跡窓。後隙終端後の攻撃判定・接触も観測して、
/// 「遅れて届いた」と「接触しなかった」を区別するために使う。
pub(crate) const PUNISH_FOLLOWUP_WINDOW: u32 = 20;
/// Recovery 終端とコンタクト抽出の境界ずれを吸収する猶予。
/// これを超えて着弾した攻撃は、時間上の確定反撃成功とはみなさない。
pub(crate) const PUNISH_CONTACT_ALIGNMENT_GRACE: u32 = 4;
/// 機会検証: 相手の後隙開始の直前このフレーム内に自分の攻撃接触があれば、
/// その後隙は「自分の技をガード/被弾した結果」であり確反機会ではない
pub(crate) const PUNISH_CAUSE_LOOKBACK: u32 = 30;
/// 反撃候補の起点: 相手の後隙開始直前に block コンタクトを探す範囲。
/// block は攻撃判定の接触だけを示し、本体間距離は後段の空間解析で確認する。
pub(crate) const PUNISH_MISSED_LOOKBACK: u32 = 20;
/// 無敵技の接触/被弾判定窓
pub(crate) const REVERSAL_WINDOW: u32 = 45;
pub(crate) const REVERSAL_PUNISH_WINDOW: u32 = 60;
/// 同じ無敵技の inv 表示に弾・stun・other が短く混入した場合の結合幅。
/// 実測では1行動中の inv_proj が最大3F途切れていた。
pub(crate) const REVERSAL_INV_MERGE_GAP: usize = 6;
/// inv ラン終了後、own Active（技の攻撃判定）を探す窓（video frames）。
/// DP/SA は inv が切れて 0-3F で active が出る。投げ抜け・被投げ・
/// 起き上がりのシステム無敵は active が続かない
pub(crate) const REVERSAL_ACT_LOOKAHEAD: usize = 10;
/// ガード入力崩れとみなす最小ドロップ（崩しの初段は小さいことが多い。
/// f8995 実測: 崩れの初段は -3%）
pub(crate) const GUARD_BREAK_MIN_DROP: f32 = 0.02;
/// ブロック硬直を遡る窓
pub(crate) const GB_LOOKBACK: usize = 30;
/// ブロック硬直（stun + ガード方向保持）とみなす最小フレーム数
pub(crate) const GB_MIN_BLOCK: usize = 4;
/// 空振り後の被弾をこのフレーム数まで紐付ける
pub(crate) const PUNISH_PUNISHED_WINDOW: u32 = 45;
/// stun 配列が無いときのフォールバック: 被弾シーケンス末尾からこのフレーム数は
/// やられ継続中とみなし、上入力をジャンプにしない
pub(crate) const JUMP_DMG_TAIL: u32 = 60;
/// 投げ成立の最小ドロップ
pub(crate) const THROW_MIN_DROP: f32 = 0.04;
/// バーンアウト run のギャップ許容
pub(crate) const BO_GAP: usize = 30;
/// バーンアウト期間の最小持続。SF6 のバーンアウトは回復に十数秒かかるため
/// 短時間で解消する期間は原理的に偽物（ただしラウンド終端まで続いた場合は
/// 「突入直後に KO」の実例があるので長さを問わない）
pub(crate) const BO_MIN_FRAMES: u32 = 180;
pub(crate) const BO_ROUND_END_MARGIN: u32 = 90;
/// 既に KO 済み（この値以下）の側の後続イベントは無視する
pub(crate) const DEAD_HP: f32 = 0.05;

// ── 本体 ─────────────────────────────────────────────────────────────────────
