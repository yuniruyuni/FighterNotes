//! e2e 検証: スタック済みストリップ PNG 列から全パイプラインを実行する。
//!
//! 入力 PNG: 1920x184（rows 0-69 = HUD y64-133、rows 70-105 = 入力 y232-267、
//!            rows 106-183 = フレームメーター y796-873）。
//! 旧 106 行（メーターなし）も受け付ける（stun ゲートはフォールバック動作）。
//! 抽出例:
//!   ffmpeg -i video.mp4 -vf "split=3[a][b][c];[a]crop=1920:70:0:64[hud];\
//!     [b]crop=1920:36:0:232[inp];[c]crop=1920:78:0:796[met];\
//!     [hud][inp][met]vstack=inputs=3" -start_number 0 f_%06d.png
//!
//! 使い方:
//!   cargo run --release --example analyze_stacked -- --side p2 [--events] /tmp/strips/f_*.png

use video_analyzer::{FrameFeatures, InputRow};

const W: u32 = 1920;
const H: u32 = 1080;
const HUD_H: usize = 70;
const INPUT_H: usize = 36;
const METER_H: usize = 78;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut side = "p1".to_string();
    let mut own_char: Option<String> = None;
    let mut show_events = false;
    let mut dump: Option<(usize, usize)> = None;
    let mut meter_dump: Option<(i64, i64)> = None;
    let mut dump_features: Option<String> = None;
    let mut files: Vec<&str> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--side" => {
                i += 1;
                side = args[i].clone();
            }
            "--events" => show_events = true,
            "--own-char" => {
                i += 1;
                own_char = Some(args[i].clone());
            }
            "--dump" => {
                i += 1;
                let parts: Vec<&str> = args[i].splitn(2, '-').collect();
                dump = Some((parts[0].parse().unwrap(), parts[1].parse().unwrap()));
            }
            "--meter-dump" => {
                i += 1;
                let parts: Vec<&str> = args[i].splitn(2, '-').collect();
                meter_dump = Some((parts[0].parse().unwrap(), parts[1].parse().unwrap()));
            }
            "--dump-features" => {
                i += 1;
                dump_features = Some(args[i].clone());
            }
            f => files.push(f),
        }
        i += 1;
    }
    if files.is_empty() {
        eprintln!("usage: analyze_stacked [--side p1|p2] [--events] <png>...");
        std::process::exit(1);
    }
    files.sort();

    let mut features: Vec<FrameFeatures> = Vec::with_capacity(files.len());
    let mut input_rows: Vec<(InputRow, InputRow)> = Vec::with_capacity(files.len());
    let mut tracker = meter_tracker::MeterTracker::new();
    let mut has_meter = false;

    for (fi, path) in files.iter().enumerate() {
        let img = image::open(path).unwrap_or_else(|e| panic!("cannot open {path}: {e}"));
        let rgba = image::DynamicImage::into_rgba8(img);
        let buf = rgba.as_raw();
        let hud = &buf[..W as usize * HUD_H * 4];
        let inp = &buf[W as usize * HUD_H * 4..W as usize * (HUD_H + INPUT_H) * 4];
        if buf.len() >= W as usize * (HUD_H + INPUT_H + METER_H) * 4 {
            has_meter = true;
            let met = &buf
                [W as usize * (HUD_H + INPUT_H) * 4..W as usize * (HUD_H + INPUT_H + METER_H) * 4];
            let (l, r) = frame_meter::extract_row_obs_from_strip(met, W, H);
            tracker.update(fi as i64, l, r);
        }

        let (raw_left, left_unc) =
            video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(hud, W, H, "p1");
        let (raw_right, right_unc) =
            video_analyzer::hp_fill_ratio_with_quality_from_hud_strip(hud, W, H, "p2");
        let left_hp = if left_unc { -1.0 } else { raw_left };
        let right_hp = if right_unc { -1.0 } else { raw_right };
        let (own_hp, opponent_hp) = if side == "p1" {
            (left_hp, right_hp)
        } else {
            (right_hp, left_hp)
        };
        let left_score = video_analyzer::hp_bar_score_from_hud_strip(hud, W, H, "p1");
        let right_score = video_analyzer::hp_bar_score_from_hud_strip(hud, W, H, "p2");
        let left_drive = video_analyzer::drive_gauge_read_from_hud_strip(hud, W, H, "left");
        let right_drive = video_analyzer::drive_gauge_read_from_hud_strip(hud, W, H, "right");

        features.push(FrameFeatures {
            frame_index: fi as u32,
            fps: 60.0,
            own_hp,
            opponent_hp,
            is_match_screen: left_score >= 0.035 && right_score >= 0.025,
            own_meter_state: None,
            opponent_meter_state: None,
            left_hp_score: left_score,
            right_hp_score: right_score,
            left_drive_ratio: if left_drive.burnout {
                left_drive.recovery
            } else {
                left_drive.value / 6.0
            },
            right_drive_ratio: if right_drive.burnout {
                right_drive.recovery
            } else {
                right_drive.value / 6.0
            },
            left_burnout: left_drive.burnout,
            right_burnout: right_drive.burnout,
            left_drive_uncertain: left_drive.uncertain,
            right_drive_uncertain: right_drive.uncertain,
            left_hp_raw: raw_left,
            right_hp_raw: raw_right,
            left_hp_raw_quality: if left_unc { 1.0 } else { 0.0 },
            right_hp_raw_quality: if right_unc { 1.0 } else { 0.0 },
        });

        let p1 = video_analyzer::read_input_row0_from_strip(inp, W, "p1");
        let p2 = video_analyzer::read_input_row0_from_strip(inp, W, "p2");
        input_rows.push((p1, p2));

        if (fi + 1) % 1000 == 0 {
            eprintln!("{} / {} frames", fi + 1, files.len());
        }
    }

    if let Some((a, b)) = dump {
        for f in features
            .iter()
            .filter(|f| (f.frame_index as usize) >= a && (f.frame_index as usize) <= b)
        {
            eprintln!(
                "f{}: match={} L raw={:.3} q={:.0} | R raw={:.3} q={:.0} | BO L={} R={}",
                f.frame_index,
                f.is_match_screen,
                f.left_hp_raw,
                f.left_hp_raw_quality,
                f.right_hp_raw,
                f.right_hp_raw_quality,
                f.left_burnout,
                f.right_burnout,
            );
        }
        return;
    }

    // wasm の finish() と同一の確定層処理（pipeline に集約済み）
    video_analyzer::finalize_features(&mut features);

    let p1_rows: Vec<InputRow> = input_rows.iter().map(|(a, _)| a.clone()).collect();
    let p2_rows: Vec<InputRow> = input_rows.iter().map(|(_, b)| b.clone()).collect();
    let p1_tracked = video_analyzer::repair_row0_sequence(&p1_rows);
    let p2_tracked = video_analyzer::repair_row0_sequence(&p2_rows);

    // --dump-features: 確定層の出力（イベント層の入力となる中間表現）を
    // offline で調査できるよう JSON へ保存する。
    if let Some(path) = dump_features {
        tracker.finish();
        #[derive(serde::Serialize)]
        struct PipelineDump<'a> {
            side: &'a str,
            features: &'a [FrameFeatures],
            p1_tracked: &'a [video_analyzer::TrackedInput],
            p2_tracked: &'a [video_analyzer::TrackedInput],
            meter_left: &'a meter_tracker::MeterTimeline,
            meter_right: &'a meter_tracker::MeterTimeline,
        }
        let json = serde_json::to_string(&PipelineDump {
            side: &side,
            features: &features,
            p1_tracked: &p1_tracked,
            p2_tracked: &p2_tracked,
            meter_left: &tracker.left,
            meter_right: &tracker.right,
        })
        .unwrap();
        std::fs::write(&path, json).unwrap();
        eprintln!("dumped features to {path}");
        return;
    }

    if let Some((a, b)) = meter_dump {
        tracker.finish();
        for (name, tl) in [("P1", &tracker.left), ("P2", &tracker.right)] {
            eprintln!("── {name} メータータイムライン vf{a}-vf{b} ──");
            for seg in &tl.segments {
                for e in &seg.entries {
                    if e.video_frame_last < a || e.video_frame_first > b {
                        continue;
                    }
                    let dwell = e.video_frame_last - e.video_frame_first + 1;
                    eprintln!(
                        "  vf{:>5}-{:>5} (dwell {:>2}) gf{:>3} {}{}",
                        e.video_frame_first,
                        e.video_frame_last,
                        dwell,
                        e.game_frame,
                        e.state,
                        if dwell >= 5 { "  ← 停止" } else { "" },
                    );
                }
            }
        }
        return;
    }

    let meter = if has_meter {
        tracker.finish();
        Some((&tracker.left, &tracker.right))
    } else {
        None
    };
    eprintln!(
        "メーター: {}",
        if has_meter {
            "あり"
        } else {
            "なし・フォールバック"
        }
    );

    let events =
        video_analyzer::build_match_events(&features, &p1_tracked, &p2_tracked, meter, &side);

    eprintln!("\n═══ イベント層 ═══");
    for r in &events.rounds {
        eprintln!(
            "Round {}: f{}-f{} 勝者={:?} P1終={:.2} P2終={:.2}",
            r.round_no, r.start_frame, r.end_frame, r.winner, r.p1_hp_end, r.p2_hp_end
        );
    }
    eprintln!("ダメージ: {} 件, ジャンプ: {} 件, 投げ: {} 件, バーンアウト: {} 件, コンタクト: {} 件 (hit {}/block {})",
        events.damage.len(), events.jumps.len(), events.throws.len(), events.burnouts.len(),
        events.contacts.len(),
        events.contacts.iter().filter(|c| c.hit).count(),
        events.contacts.iter().filter(|c| !c.hit).count());
    eprintln!(
        "入力セグメント: P1={} P2={}",
        events.segments[0].len(),
        events.segments[1].len()
    );

    if show_events {
        eprintln!("\n── ダメージ ──");
        for d in &events.damage {
            eprintln!(
                "  f{}-f{} R{} P{} -{:.3} ({:.2}→{:.2})",
                d.start_frame, d.end_frame, d.round_no, d.victim, d.drop, d.hp_before, d.hp_after
            );
        }
        eprintln!("\n── ジャンプ ──");
        for j in &events.jumps {
            eprintln!("  f{} R{} P{} {:?}", j.frame, j.round_no, j.side, j.outcome);
        }
        eprintln!("\n── 投げ ──");
        for t in &events.throws {
            eprintln!(
                "  f{} R{} P{} connected={}",
                t.frame, t.round_no, t.thrower, t.connected
            );
        }
        eprintln!("\n── コンタクト ──");
        for c in &events.contacts {
            eprintln!(
                "  f{} R{} P{}→P{} {}",
                c.frame,
                c.round_no,
                c.attacker,
                c.victim,
                if c.hit { "HIT" } else { "BLOCK" }
            );
        }
        eprintln!("\n── バーンアウト ──");
        for b in &events.burnouts {
            eprintln!(
                "  f{}-f{} R{} P{} hp_lost={:.3}",
                b.start_frame, b.end_frame, b.round_no, b.side, b.hp_lost
            );
        }
        eprintln!("\n── 確反機会 ──");
        for p in &events.punishes {
            eprintln!(
                "  f{} R{} P{} +{}F {:?} pressed={} punished={:.3}",
                p.frame, p.round_no, p.side, p.advantage, p.outcome, p.pressed, p.punished_drop
            );
        }
        eprintln!("\n── 無敵技ぶっぱ被弾 ──");
        for r in &events.reversals {
            eprintln!(
                "  f{} R{} P{} blocked={} -{:.3}",
                r.frame, r.round_no, r.side, r.blocked, r.drop
            );
        }
        eprintln!("\n── ガード入力崩れ ──");
        for g in &events.guard_breaks {
            eprintln!(
                "  f{} R{} P{} {}→{} -{:.3}",
                g.frame, g.round_no, g.side, g.guard_dir, g.broke_to, g.drop
            );
        }
    }

    let report =
        video_analyzer::advice::build_report(&features, &events, &side, own_char.as_deref());
    eprintln!("\n═══ レポート ═══");
    eprintln!("summary: {}", report.summary);
    if let Some(st) = &report.input_stats {
        eprintln!(
            "入力統計: 入力{} ジャンプ{}({:.1}/分, 落とされ{} 通し{}) 投げ{}/{} ボタン{} AUTO率{:.0}% DI{} しゃがみ{:.0}%",
            st.total_inputs, st.jumps, st.jumps_per_min, st.jump_got_hit, st.jump_landed,
            st.throw_hits, st.throw_attempts, st.button_presses, st.auto_ratio * 100.0,
            st.di_presses, st.crouch_ratio * 100.0
        );
    }
    for rs in &report.round_summaries {
        eprintln!(
            "R{}: won={:?} own_end={:.2} opp_end={:.2} 被弾{}回(-{:.2}) 開幕被弾={} BO={}",
            rs.round_no,
            rs.won,
            rs.own_hp_end,
            rs.opp_hp_end,
            rs.own_hits_taken,
            rs.own_hp_lost,
            rs.early_hit,
            rs.own_burnouts
        );
    }
    for c in &report.cards {
        eprintln!(
            "\n▼ [{}] {} (severity {:.2}, {} 場面)",
            c.id,
            c.title,
            c.severity,
            c.evidence.len()
        );
        eprintln!("  {}", c.description);
        for e in &c.evidence {
            eprintln!("    f{} {}", e.frame, e.label);
        }
    }

    // JSON 全体も stdout に出す（機械検証用）
    println!("{}", serde_json::to_string(&report).unwrap());
}
