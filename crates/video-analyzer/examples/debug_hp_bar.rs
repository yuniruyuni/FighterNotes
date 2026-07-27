//! HP バーデコード詳細デバッグツール
//!
//! 使い方:
//!   cargo run --example debug_hp_bar -- [--side p1|p2] [--detail cy_from-cy_to] <png>...
//!
//! 例:
//!   cargo run --example debug_hp_bar -- --side p1 /tmp/frame_*.png
//!   cargo run --example debug_hp_bar -- --side p1 --detail 510-585 /tmp/frame_0005.png

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut side = "p1";
    let mut drive_mode = false;
    let mut input_mode = false;
    let mut track_mode = false;
    let mut detail_range: Option<(usize, usize)> = None;
    let mut files: Vec<&str> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--side" => {
                i += 1;
                side = &args[i];
            }
            "--drive" => {
                drive_mode = true;
            }
            "--input" => {
                input_mode = true;
            }
            "--track" => {
                track_mode = true;
            }
            "--detail" => {
                i += 1;
                let parts: Vec<&str> = args[i].splitn(2, '-').collect();
                if parts.len() == 2 {
                    let from = parts[0].parse::<usize>().expect("invalid cy_from");
                    let to = parts[1].parse::<usize>().expect("invalid cy_to");
                    detail_range = Some((from, to));
                }
            }
            f => files.push(f),
        }
        i += 1;
    }

    if files.is_empty() {
        eprintln!("usage: debug_hp_bar [--side p1|p2] [--detail cy_from-cy_to] <png>...");
        std::process::exit(1);
    }

    // --track: 複数 PNG を連続フレームとして row0 を補修表示
    if track_mode {
        let mut rows0 = Vec::new();
        for path in &files {
            let img = image::open(path).unwrap_or_else(|e| panic!("cannot open {path}: {e}"));
            let (w, h) = (img.width(), img.height());
            let rgba = image::DynamicImage::into_rgba8(img);
            let rows = video_analyzer::read_input_rows(rgba.as_raw(), w, h, side);
            rows0.push(rows.into_iter().next().expect("row0"));
        }
        let tracked = video_analyzer::repair_row0_sequence(&rows0);
        for (i, (t, path)) in tracked.iter().zip(files.iter()).enumerate() {
            println!(
                "{:4} {}: count={:>4} dir={:<2} badges={:<8} auto={} throw={} repaired={} unc={}",
                i,
                path,
                t.count.map_or("?".to_string(), |c| c.to_string()),
                t.dir.as_str(),
                t.badges
                    .iter()
                    .map(|b| b.label())
                    .collect::<Vec<_>>()
                    .join(" "),
                t.auto,
                t.throw,
                t.repaired,
                t.uncertain,
            );
        }
        return;
    }

    for path in &files {
        let img = image::open(path).unwrap_or_else(|e| panic!("cannot open {path}: {e}"));
        let width = img.width();
        let height = img.height();
        let rgba = image::DynamicImage::into_rgba8(img);
        let bytes = rgba.as_raw();

        // --input: 入力履歴モード
        if input_mode {
            let json = video_analyzer::input_history_debug_json(bytes, width, height, side);
            let v: serde_json::Value = serde_json::from_str(&json).expect("json parse error");
            println!("\n=== {} (input {}) ===", path, side);
            if let Some(rows) = v["rows"].as_array() {
                for (i, r) in rows.iter().enumerate() {
                    let empty = r["empty"].as_bool().unwrap_or(false);
                    if empty {
                        continue;
                    }
                    println!(
                        "  行{:2}: count={:>4} dir={:<2} badges={:<8} auto={} throw={} unc={}",
                        i,
                        r["count"]
                            .as_u64()
                            .map_or("?".to_string(), |c| c.to_string()),
                        r["dir"].as_str().unwrap_or("?"),
                        r["badges"].as_str().unwrap_or(""),
                        r["auto"].as_bool().unwrap_or(false),
                        r["throw"].as_bool().unwrap_or(false),
                        r["uncertain"].as_bool().unwrap_or(false),
                    );
                }
            }
            continue;
        }

        // --drive: ドライブゲージモード（side は "left"/"right" にマップ）
        if drive_mode {
            let dside = if side == "p1" || side == "left" {
                "left"
            } else {
                "right"
            };
            let json = video_analyzer::drive_bar_debug_json(bytes, width, height, dside);
            let v: serde_json::Value = serde_json::from_str(&json).expect("json parse error");
            println!("\n=== {} (drive {}) ===", path, dside);
            println!(
                "value={:.3}  burnout={}  recovery={:.3}  uncertain={}",
                v["value"].as_f64().unwrap_or(0.0),
                v["burnout"].as_bool().unwrap_or(false),
                v["recovery"].as_f64().unwrap_or(0.0),
                v["uncertain"].as_bool().unwrap_or(false),
            );
            println!("runs:");
            if let Some(runs) = v["runs"].as_array() {
                for r in runs {
                    println!(
                        "  {:6}  a={:3}–{:3}  w={:3}",
                        r["c"].as_str().unwrap_or("?"),
                        r["s"].as_u64().unwrap_or(0),
                        r["e"].as_u64().unwrap_or(0),
                        r["w"].as_u64().unwrap_or(0),
                    );
                }
            }
            continue;
        }

        let summary_json = video_analyzer::hp_bar_debug_json(bytes, width, height, side);

        // サマリーをパースして人間が読みやすい形式で出力
        let v: serde_json::Value = serde_json::from_str(&summary_json).expect("json parse error");

        println!("\n=== {} (side={}) ===", path, side);
        println!(
            "fill_ratio={:.4}  orange_fill={:.4}  uncertain={}",
            v["fill_ratio"].as_f64().unwrap_or(0.0),
            v["orange_fill"].as_f64().unwrap_or(0.0),
            v["uncertain"].as_bool().unwrap_or(false),
        );
        println!(
            "fill_edge_cy={}  damage_left_cy={}",
            v["fill_edge_cy"], v["damage_left_cy"],
        );

        // ゾーン一覧
        println!("zones:");
        if let Some(zones) = v["zones"].as_array() {
            for z in zones {
                println!(
                    "  {:6}  cy={:3}–{:3}  w={:3}",
                    z["c"].as_str().unwrap_or("?"),
                    z["s"].as_u64().unwrap_or(0),
                    z["e"].as_u64().unwrap_or(0),
                    z["w"].as_u64().unwrap_or(0),
                );
            }
        }

        // per-pixel 詳細（--detail 指定時）
        if let Some((from, to)) = detail_range {
            let detail_json =
                video_analyzer::hp_col_pixel_detail_json(bytes, width, height, side, from, to);
            let cols: serde_json::Value =
                serde_json::from_str(&detail_json).expect("detail json parse error");

            println!("\nper-column detail (cy {}–{}):", from, to);
            println!(
                "{:>4}  {:>6}  {:>5} {:>5} {:>5}  {:>4} {:>4} {:>4}  {:>5}",
                "cy", "cls", "nW", "nF", "nY", "nO", "tot", "n/a", "rows"
            );
            if let Some(cols_arr) = cols.as_array() {
                for col in cols_arr {
                    let cy = col["cy"].as_u64().unwrap_or(0);
                    let cls = col["col_cls"].as_str().unwrap_or("?");
                    let tot = col["total"].as_u64().unwrap_or(0);
                    let nw = col["nW"].as_u64().unwrap_or(0);
                    let nf = col["nF"].as_u64().unwrap_or(0);
                    let ny = col["nY"].as_u64().unwrap_or(0);
                    let no = col["nO"].as_u64().unwrap_or(0);
                    println!(
                        "{:>4}  {:>6}  {:>5} {:>5} {:>5}  {:>4} {:>4}",
                        cy, cls, nw, nf, ny, no, tot
                    );

                    if let Some(rows) = col["rows"].as_array() {
                        for row in rows {
                            println!(
                                "        ry={:2} gx={:4}  R={:3} G={:3} B={:3}  H={:3} S={:3} V={:3}  {}",
                                row["ry"].as_u64().unwrap_or(0),
                                row["gx"].as_u64().unwrap_or(0),
                                row["r"].as_u64().unwrap_or(0),
                                row["g"].as_u64().unwrap_or(0),
                                row["b"].as_u64().unwrap_or(0),
                                row["h"].as_f64().map(|v| v as u64).unwrap_or(0),
                                row["s"].as_f64().map(|v| v as u64).unwrap_or(0),
                                row["v"].as_f64().map(|v| v as u64).unwrap_or(0),
                                row["cls"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                }
            }
        }
    }
}
