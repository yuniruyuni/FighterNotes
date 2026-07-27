// メータータイムラインの型と逆引きインデックス。

export interface RustTimelineEntry {
  game_frame: number;
  state: string;
  video_frame_first: number;
  video_frame_last: number;
  confidence: number;
}

export interface RustTimelineSegment {
  segment_id: number;
  entries: RustTimelineEntry[];
}

export interface RustMeterTimeline {
  side: string;
  segments: RustTimelineSegment[];
}

export interface RustTimeline {
  left: RustMeterTimeline;
  right: RustMeterTimeline;
  // video_frame (string key) -> [segment_id, abs]
  video_map: Record<string, [number, number]>;
}

// ─── タイムライン逆引きインデックス ──────────────────────────────────────────

export interface IndexedTimeline {
  ivals: [number, number, RustTimelineEntry & { segment_id: number }][];
  byGf: Map<string, RustTimelineEntry & { segment_id: number }>;
  // video_frame -> {segment_id, game_frame} — 全ビデオフレームの確定マッピング
  videoMap: Map<number, { segment_id: number; game_frame: number }>;
}

export function buildIndex(
  tl: RustMeterTimeline,
  rawVideoMap: Record<string, [number, number]>,
): IndexedTimeline {
  const ivals: IndexedTimeline["ivals"] = [];
  const byGf = new Map<string, RustTimelineEntry & { segment_id: number }>();
  for (const seg of tl.segments) {
    for (const e of seg.entries) {
      const entry = { ...e, segment_id: seg.segment_id };
      byGf.set(`${seg.segment_id}:${e.game_frame}`, entry);
      if (e.video_frame_first >= 0) {
        ivals.push([e.video_frame_first, e.video_frame_last, entry]);
      }
    }
  }
  ivals.sort((a, b) => a[0] - b[0]);

  // video_map を Map に変換（string キー → number）
  const videoMap = new Map<
    number,
    { segment_id: number; game_frame: number }
  >();
  for (const [vfStr, [segId, abs]] of Object.entries(rawVideoMap)) {
    videoMap.set(Number(vfStr), { segment_id: segId, game_frame: abs });
  }

  return { ivals, byGf, videoMap };
}

export function finalValueAt(
  ivals: IndexedTimeline["ivals"],
  frameIdx: number,
): (RustTimelineEntry & { segment_id: number }) | null {
  const insertionIndex = upperBound(ivals, frameIdx);
  if (insertionIndex === 0) return null;
  const [, videoFrameLast, entry] = ivals[insertionIndex - 1];
  return frameIdx <= videoFrameLast ? entry : null;
}

function upperBound(ivals: IndexedTimeline["ivals"], frameIdx: number): number {
  let low = 0;
  let high = ivals.length;
  for (const _entry of ivals) {
    if (low >= high) break;
    const middle = low + Math.floor((high - low) / 2);
    if (ivals[middle][0] <= frameIdx) low = middle + 1;
    else high = middle;
  }
  return low;
}
