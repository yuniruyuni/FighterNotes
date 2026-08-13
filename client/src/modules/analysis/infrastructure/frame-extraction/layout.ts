export const ANALYSIS_WIDTH = 1920;
export const ANALYSIS_HEIGHT = 1080;

export interface AnalysisStrip {
  readonly y: number;
  readonly height: number;
  readonly byteLength: number;
}

function strip(y: number, height: number): AnalysisStrip {
  return {
    y,
    height,
    byteLength: ANALYSIS_WIDTH * height * 4,
  };
}

export const ANALYSIS_STRIPS = {
  hud: strip(64, 70),
  input: strip(232, 36),
  meter: strip(796, 78),
} as const;

/**
 * フレームメーターと画面下端の SA ゲージを、1 回の createImageBitmap で
 * 切り出すための下部アトラス。WASM へ渡す strip の大きさは変更せず、
 * 必要な領域だけを各 strip canvas へ描画する。
 */
export const LOWER_ATLAS_LAYOUT = {
  source: {
    x: 55,
    y: ANALYSIS_STRIPS.meter.y,
    width: 1810,
    height: 234,
  },
  meter: {
    source: {
      x: 304,
      y: 0,
      width: 1200,
      height: ANALYSIS_STRIPS.meter.height,
    },
    target: {
      x: 359,
      y: 0,
      width: 1200,
      height: ANALYSIS_STRIPS.meter.height,
    },
  },
} as const;

/**
 * 入力履歴stripと中央の攻撃情報を1回のcreateImageBitmapで取得する中段
 * アトラス。個別の攻撃情報bitmapを増やさず、従来どおり1フレーム3枚の
 * bitmap生成に収める。
 */
export const MID_ATLAS_LAYOUT = {
  source: { x: 0, y: 174, width: ANALYSIS_WIDTH, height: 94 },
  input: {
    source: { x: 0, y: 58, width: ANALYSIS_WIDTH, height: 36 },
    target: { x: 0, y: 0, width: ANALYSIS_WIDTH, height: 36 },
  },
} as const;

/**
 * FIGHT の中央画像を低頻度で縮小し、HUD strip の未使用中央領域へ埋め込む。
 * target は HP（x<=853 / x>=1067）と drive（x<=895 / x>=1025）の
 * 読み取り範囲に重ならない。
 */
export const FIGHT_MARKER_LAYOUT = {
  sampleInterval: 4,
  source: {
    x: 400,
    y: 300,
    width: 1120,
    height: 455,
  },
  target: {
    x: 896,
    y: 9,
    width: 128,
    height: 52,
  },
} as const;

/**
 * トレーニング表示の攻撃情報（直近ダメージ/補正率、コンボ値/最大値、
 * 上段・中段・下段・投げ）を meter strip の未使用左右端へ等倍で詰める。
 *
 * frame meter が使用する x=359..1558 とは重ならない。数字と属性を別々に
 * 切り出すことで、80px ある3行を78pxへ縮小せず、既存の数字統計モデルを
 * そのまま利用できる。
 */
export const ATTACK_INFO_LAYOUT = {
  p1: {
    numeric: {
      source: { x: 600, y: 0, width: 190, height: 56 },
      target: { x: 0, y: 0, width: 190, height: 56 },
    },
    attribute: {
      source: { x: 749, y: 62, width: 32, height: 20 },
      target: { x: 200, y: 0, width: 32, height: 20 },
    },
  },
  p2: {
    numeric: {
      source: { x: 1136, y: 0, width: 190, height: 56 },
      target: { x: 1559, y: 0, width: 190, height: 56 },
    },
    attribute: {
      source: { x: 1141, y: 62, width: 32, height: 20 },
      target: { x: 1759, y: 0, width: 32, height: 20 },
    },
  },
} as const;

/**
 * 画面下端の SA ゲージから、数値/CA ラベルと部分ゲージだけを HUD strip の
 * 未使用領域へ埋め込む。HUD 全体の転送量・getImageData 回数は増やさない。
 *
 * bar target は y>=32 のため HP（y<31）と重ならず、左右端に置くことで
 * drive / FIGHT の読み取り範囲にも重ならない。
 */
export const SUPER_GAUGE_LAYOUT = {
  left: {
    label: {
      source: { x: 0, y: 159, width: 90, height: 75 },
      target: { x: 0, y: 0, width: 90, height: 70 },
    },
    bar: {
      source: { x: 90, y: 179, width: 265, height: 50 },
      target: { x: 100, y: 32, width: 265, height: 38 },
    },
  },
  right: {
    bar: {
      source: { x: 1455, y: 179, width: 265, height: 50 },
      target: { x: 1555, y: 32, width: 265, height: 38 },
    },
    label: {
      source: { x: 1720, y: 159, width: 90, height: 75 },
      target: { x: 1830, y: 0, width: 90, height: 70 },
    },
  },
} as const;

/**
 * `VideoFrame.copyTo` 用に、アトラス相対の座標を動画フレームの絶対座標へ直した窓。
 *
 * I420 の彩度は 2px 単位で、奇数 x から読むと canvas 経路と値がずれる。
 * `readX` / `readWidth` は偶数境界へ広げた読み出し範囲で、`source.x - readX`
 * だけずらして strip へ書き写すと従来と同じ画素になる。
 */
export interface CopyWindow {
  readonly key: string;
  readonly source: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
  readonly target: { readonly x: number; readonly y: number };
  readonly readX: number;
  readonly readWidth: number;
}

function copyWindowOf(
  key: string,
  atlas: { readonly x: number; readonly y: number },
  patch: {
    readonly source: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
    readonly target: { readonly x: number; readonly y: number };
  },
): CopyWindow {
  const x = atlas.x + patch.source.x;
  const readX = x - (x % 2);
  const readWidth = alignUpToEven(patch.source.width + (x - readX));
  return {
    key,
    source: {
      x,
      y: atlas.y + patch.source.y,
      width: patch.source.width,
      height: patch.source.height,
    },
    target: { x: patch.target.x, y: patch.target.y },
    readX,
    readWidth,
  };
}

function alignUpToEven(value: number): number {
  return value + (value % 2);
}

/** 攻撃情報は中段アトラスから meter strip の左右端へ等倍で詰める。 */
export const ATTACK_INFO_COPY_WINDOWS: readonly CopyWindow[] = [
  copyWindowOf(
    "p1-numeric",
    MID_ATLAS_LAYOUT.source,
    ATTACK_INFO_LAYOUT.p1.numeric,
  ),
  copyWindowOf(
    "p1-attribute",
    MID_ATLAS_LAYOUT.source,
    ATTACK_INFO_LAYOUT.p1.attribute,
  ),
  copyWindowOf(
    "p2-numeric",
    MID_ATLAS_LAYOUT.source,
    ATTACK_INFO_LAYOUT.p2.numeric,
  ),
  copyWindowOf(
    "p2-attribute",
    MID_ATLAS_LAYOUT.source,
    ATTACK_INFO_LAYOUT.p2.attribute,
  ),
];

/** frame meter は下部アトラス経由で strip の同じ x へ等倍で入る。 */
export const METER_COPY_WINDOW: CopyWindow = copyWindowOf(
  "meter",
  LOWER_ATLAS_LAYOUT.source,
  LOWER_ATLAS_LAYOUT.meter,
);

/**
 * 縮小して詰める領域を、動画フレームの絶対座標へ直した読み出し窓。
 *
 * 縮小は canvas でしかできないが、`createImageBitmap` にフレームを渡すと
 * 切り出し範囲に関係なくフレーム全体を変換する。元領域だけ `copyTo` で
 * 読み、その画素から小さな bitmap を作れば、費用は領域の大きさで収まる。
 *
 * `readX` / `readY` は I420 の彩度に合わせて偶数へ広げた読み出し原点で、
 * `offsetX` / `offsetY` だけずらした先が本来の領域になる。
 */
export interface ScaledCopyWindow {
  readonly key: string;
  readonly readX: number;
  readonly readY: number;
  readonly readWidth: number;
  readonly readHeight: number;
  readonly offsetX: number;
  readonly offsetY: number;
  readonly source: { readonly width: number; readonly height: number };
  readonly target: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
}

function scaledCopyWindowOf(
  key: string,
  atlas: { readonly x: number; readonly y: number },
  patch: {
    readonly source: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
    readonly target: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
  },
): ScaledCopyWindow {
  const x = atlas.x + patch.source.x;
  const y = atlas.y + patch.source.y;
  const readX = x - (x % 2);
  const readY = y - (y % 2);
  const offsetX = x - readX;
  const offsetY = y - readY;
  return {
    key,
    readX,
    readY,
    readWidth: alignUpToEven(patch.source.width + offsetX),
    readHeight: alignUpToEven(patch.source.height + offsetY),
    offsetX,
    offsetY,
    source: { width: patch.source.width, height: patch.source.height },
    target: patch.target,
  };
}

/** SA ゲージの数値・CA ラベルと部分ゲージ。HUD strip の未使用域へ縮小して詰める。 */
export const SUPER_GAUGE_COPY_WINDOWS: readonly ScaledCopyWindow[] = [
  scaledCopyWindowOf(
    "left-label",
    LOWER_ATLAS_LAYOUT.source,
    SUPER_GAUGE_LAYOUT.left.label,
  ),
  scaledCopyWindowOf(
    "left-bar",
    LOWER_ATLAS_LAYOUT.source,
    SUPER_GAUGE_LAYOUT.left.bar,
  ),
  scaledCopyWindowOf(
    "right-bar",
    LOWER_ATLAS_LAYOUT.source,
    SUPER_GAUGE_LAYOUT.right.bar,
  ),
  scaledCopyWindowOf(
    "right-label",
    LOWER_ATLAS_LAYOUT.source,
    SUPER_GAUGE_LAYOUT.right.label,
  ),
];

/**
 * GPU が復号フレームから直接切り出す等倍の矩形。
 *
 * `src` はフレーム全体、`dst` は 3 つの strip を縦に並べた 1 枚の中の位置。
 * 縮小の要る SA ゲージと FIGHT は含まない。canvas の高品質縮小と 1 だけ
 * ずれる画素が出るため、そこだけは合成に残してある。
 */
export interface StripRect {
  readonly src: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
  readonly dst: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
}

/** 3 つの strip を縦に並べた 1 枚の中での、各 strip の先頭行。 */
export const PACKED_BANDS = {
  hud: 0,
  meter: ANALYSIS_STRIPS.hud.height,
  input: ANALYSIS_STRIPS.hud.height + ANALYSIS_STRIPS.meter.height,
} as const;

export const PACKED_HEIGHT = PACKED_BANDS.input + ANALYSIS_STRIPS.input.height;

function absoluteRect(
  atlas: { readonly x: number; readonly y: number },
  patch: {
    readonly source: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
    readonly target: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
  },
  band: number,
): StripRect {
  return {
    src: {
      x: atlas.x + patch.source.x,
      y: atlas.y + patch.source.y,
      width: patch.source.width,
      height: patch.source.height,
    },
    dst: {
      x: patch.target.x,
      y: band + patch.target.y,
      width: patch.target.width,
      height: patch.target.height,
    },
  };
}

/**
 * 重ね書きの手前までの数。
 *
 * SA ゲージと FIGHT は strip の上で HP バーの走査範囲と重なる。同じ
 * ディスパッチに混ぜると、どちらが後に書くか決まらない。土台を書き終えて
 * から重ねる。
 */
export const STRIP_BASE_RECTS = 1;

export const STRIP_RECTS: readonly StripRect[] = [
  {
    src: {
      x: 0,
      y: ANALYSIS_STRIPS.hud.y,
      width: ANALYSIS_WIDTH,
      height: ANALYSIS_STRIPS.hud.height,
    },
    dst: {
      x: 0,
      y: PACKED_BANDS.hud,
      width: ANALYSIS_WIDTH,
      height: ANALYSIS_STRIPS.hud.height,
    },
  },
  // SA ゲージと FIGHT は縮小して写す。ここだけ canvas の縮小と 1〜2 違う。
  ...[SUPER_GAUGE_LAYOUT.left, SUPER_GAUGE_LAYOUT.right].flatMap((side) =>
    [side.label, side.bar].map((patch) =>
      absoluteRect(LOWER_ATLAS_LAYOUT.source, patch, PACKED_BANDS.hud),
    ),
  ),
  absoluteRect({ x: 0, y: 0 }, FIGHT_MARKER_LAYOUT, PACKED_BANDS.hud),
  absoluteRect(
    LOWER_ATLAS_LAYOUT.source,
    LOWER_ATLAS_LAYOUT.meter,
    PACKED_BANDS.meter,
  ),
  ...[ATTACK_INFO_LAYOUT.p1, ATTACK_INFO_LAYOUT.p2].flatMap((side) =>
    [side.numeric, side.attribute].map((patch) =>
      absoluteRect(MID_ATLAS_LAYOUT.source, patch, PACKED_BANDS.meter),
    ),
  ),
  absoluteRect(
    MID_ATLAS_LAYOUT.source,
    MID_ATLAS_LAYOUT.input,
    PACKED_BANDS.input,
  ),
];
