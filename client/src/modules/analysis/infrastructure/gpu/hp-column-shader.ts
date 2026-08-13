/**
 * HP バーの列分類。1 呼び出しが 1 列を受け持つ。
 *
 * `classify_hp_pixel` と `classify_hp_col` を写したもの。数値の扱いで
 * 気をつけている点が 3 つある。
 *
 * - Rust の `round` は 0 から遠い方へ、WGSL の `round` は偶数へ丸める。
 *   傾き 0.75 では 4.5 のような値が出るので、明示的に実装する。
 * - 採択の閾値は `n / total >= 0.50` のような割り算だが、行数は高々数十
 *   なので `n * 2 >= total` の整数比較と一致する。GPU の除算精度に
 *   結果が左右されないよう、整数のまま比べる。
 * - 色相は 3 チャンネル全部に依存するため表に置けない。ここだけは同じ式を
 *   GPU でも計算し、実データで CPU の答えと突き合わせて確かめる。
 */
import shader from "../../../../../../crates/hud-vision/shaders/hp_column.wgsl" with {
  type: "text",
};

export const HP_COLUMN_SHADER: string = shader;
