import { describe, expect, test } from "bun:test";
import { buildIndex, finalValueAt, type RustMeterTimeline } from "./timeline";

// 合成タイムライン: 2 セグメント、エントリは (gf, state, vf_first, vf_last)
const TL: RustMeterTimeline = {
  side: "left",
  segments: [
    {
      segment_id: 0,
      entries: [
        {
          game_frame: 10,
          state: "counter",
          video_frame_first: 100,
          video_frame_last: 103,
          confidence: 1.0,
        },
        {
          game_frame: 11,
          state: "active",
          video_frame_first: 104,
          video_frame_last: 110,
          confidence: 1.0,
        },
        // ホールド中などで video フレーム対応がないエントリ（vf=-1）
        {
          game_frame: 12,
          state: "stun",
          video_frame_first: -1,
          video_frame_last: -1,
          confidence: 0.5,
        },
      ],
    },
    {
      segment_id: 1,
      entries: [
        {
          game_frame: 0,
          state: "punish_counter",
          video_frame_first: 200,
          video_frame_last: 230,
          confidence: 1.0,
        },
      ],
    },
  ],
};

const VIDEO_MAP: Record<string, [number, number]> = {
  "100": [0, 10],
  "104": [0, 11],
  "200": [1, 0],
};

describe("buildIndex", () => {
  test("gf 逆引き（segment:gf キー）", () => {
    const idx = buildIndex(TL, VIDEO_MAP);
    expect(idx.byGf.get("0:11")?.state).toBe("active");
    expect(idx.byGf.get("1:0")?.state).toBe("punish_counter");
  });

  test("vf=-1 のエントリは区間インデックスに入らない（byGf には入る）", () => {
    const idx = buildIndex(TL, VIDEO_MAP);
    expect(idx.byGf.get("0:12")?.state).toBe("stun");
    expect(idx.ivals.every(([vff]) => vff >= 0)).toBe(true);
  });

  test("video_map は数値キーの Map に変換される", () => {
    const idx = buildIndex(TL, VIDEO_MAP);
    expect(idx.videoMap.get(104)).toEqual({ segment_id: 0, game_frame: 11 });
    expect(idx.videoMap.get(999)).toBeUndefined();
  });

  test("video frame 0を含め、入力順によらず区間開始順へ並べる", () => {
    const entry = (gameFrame: number, first: number, last: number) => ({
      game_frame: gameFrame,
      state: `state-${gameFrame}`,
      video_frame_first: first,
      video_frame_last: last,
      confidence: 1,
    });
    const index = buildIndex(
      {
        side: "left",
        segments: [
          { segment_id: 0, entries: [entry(2, 20, 29), entry(0, 0, 9)] },
          { segment_id: 1, entries: [entry(1, 10, 19)] },
        ],
      },
      {},
    );

    expect(index.ivals.map(([first]) => first)).toEqual([0, 10, 20]);
    expect(finalValueAt(index.ivals, 0)?.game_frame).toBe(0);
  });
});

describe("finalValueAt（二分探索の区間逆引き）", () => {
  const idx = buildIndex(TL, VIDEO_MAP);

  test("区間の内側・両端で正しいエントリを返す", () => {
    expect(finalValueAt(idx.ivals, 100)?.state).toBe("counter");
    expect(finalValueAt(idx.ivals, 103)?.state).toBe("counter");
    expect(finalValueAt(idx.ivals, 104)?.state).toBe("active");
    expect(finalValueAt(idx.ivals, 110)?.state).toBe("active");
    expect(finalValueAt(idx.ivals, 215)?.state).toBe("punish_counter");
  });

  test("区間の隙間・範囲外では null", () => {
    expect(finalValueAt(idx.ivals, 99)).toBeNull(); // 先頭より前
    expect(finalValueAt(idx.ivals, 150)).toBeNull(); // セグメント間の隙間
    expect(finalValueAt(idx.ivals, 231)).toBeNull(); // 末尾より後
  });

  test("セグメント境界でも segment_id が正しい", () => {
    expect(finalValueAt(idx.ivals, 110)?.segment_id).toBe(0);
    expect(finalValueAt(idx.ivals, 200)?.segment_id).toBe(1);
  });
});
