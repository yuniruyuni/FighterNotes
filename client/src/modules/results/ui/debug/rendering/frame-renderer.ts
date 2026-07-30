import type { AnalysisSide } from "~/modules/analysis/contracts.js";
import { buildIndex } from "~/modules/analysis/contracts.js";
import type {
  DebugDriveSideInspection,
  DebugFrameInspector,
  DebugHpGeometry,
  DebugHpInspection,
} from "../../../application/debug-frame-inspection.js";
import { frameToSeconds } from "../../../domain/frame-time.js";
import type {
  DebugOverlayVisibility,
  DebugViewerData,
} from "../debug-viewer-model.js";
import { drawAttackInfoDebug } from "./overlays/attack-info.js";
import {
  drawDriveBarReproduced,
  drawDriveColRow,
  drawDriveRoiOverlay,
  drawDriveTimeline,
} from "./overlays/drive.js";
import {
  drawHpBarReproduced,
  drawHpColActive,
  drawHpRoiOverlay,
  drawHpTimeline,
} from "./overlays/hp.js";
import {
  drawInputHistoryDebug,
  drawTrackedInputRow0,
} from "./overlays/input.js";
import { drawFinalOverlay, drawRawOverlay } from "./overlays/meter.js";
import {
  drawSuperGaugeReproduced,
  drawSuperRoiOverlay,
  drawSuperTimeline,
} from "./overlays/super.js";

export class DebugFrameRenderer {
  readonly #context: CanvasRenderingContext2D;
  readonly #timeline;
  readonly #totalFrames: number;
  #rendering = false;

  constructor(
    private readonly canvas: HTMLCanvasElement,
    private readonly onFrameInfo: (label: string) => void,
    private readonly data: DebugViewerData,
    private readonly ownSide: AnalysisSide,
    private readonly inspector: DebugFrameInspector,
    private readonly hpGeometry: DebugHpGeometry,
  ) {
    const context = canvas.getContext("2d");
    if (!context) throw new Error("デバッグ用Canvasを初期化できませんでした。");
    this.#context = context;
    this.#timeline = {
      left: buildIndex(data.timeline.left, data.timeline.video_map ?? {}),
      right: buildIndex(data.timeline.right, data.timeline.video_map ?? {}),
    };
    this.#totalFrames =
      data.frameCount > 0 ? data.frameCount : data.hpFeatures.length;
  }

  async render(
    frameIndex: number,
    source: CanvasImageSource,
    visibility: DebugOverlayVisibility,
    closeSource = false,
  ): Promise<void> {
    if (this.#rendering) {
      if (closeSource && "close" in source) source.close();
      return;
    }
    this.#rendering = true;
    try {
      this.#draw(frameIndex, source, visibility);
    } finally {
      if (closeSource && "close" in source) source.close();
      this.#rendering = false;
    }
  }

  #draw(
    frameIndex: number,
    source: CanvasImageSource,
    visibility: DebugOverlayVisibility,
  ) {
    const context = this.#context;
    const width = this.canvas.width;
    const height = this.canvas.height;
    context.drawImage(source, 0, 0, width, height);
    drawFinalOverlay(
      context,
      "left",
      this.#timeline.left,
      frameIndex,
      width,
      height,
    );
    drawFinalOverlay(
      context,
      "right",
      this.#timeline.right,
      frameIndex,
      width,
      height,
    );

    const rgba = this.#readPixelsWhenNeeded(visibility, width, height);
    if (visibility.raw && rgba) {
      const raw = this.inspector.inspectMeter(rgba, width, height);
      drawRawOverlay(context, "left", raw.left, width, height, visibility.hue);
      drawRawOverlay(
        context,
        "right",
        raw.right,
        width,
        height,
        visibility.hue,
      );
    }
    if (visibility.hp) this.#drawHp(rgba, frameIndex, width, height);
    if (visibility.drive && rgba) {
      this.#drawDrive(rgba, frameIndex, width, height);
    }
    if (visibility.super && rgba) {
      this.#drawSuper(rgba, frameIndex, width, height);
    }
    if (visibility.input && rgba) {
      const input = this.inspector.inspectInput(rgba, width, height);
      drawInputHistoryDebug(context, input, width, height);
      drawTrackedInputRow0(
        context,
        this.data.trackedInputs,
        frameIndex,
        width,
        height,
      );
    }
    if (visibility.attackInfo && rgba) {
      drawAttackInfoDebug(
        context,
        this.inspector.inspectAttackInfo(rgba, width, height),
        this.data.attackInfo,
        frameIndex,
        width,
        height,
      );
    }
    this.#drawFrameLabel(frameIndex, height);
  }

  #readPixelsWhenNeeded(
    visibility: DebugOverlayVisibility,
    width: number,
    height: number,
  ): Uint8Array | null {
    if (
      !visibility.raw &&
      !visibility.hp &&
      !visibility.drive &&
      !visibility.super &&
      !visibility.input &&
      !visibility.attackInfo
    ) {
      return null;
    }
    return new Uint8Array(
      this.#context.getImageData(0, 0, width, height).data.buffer,
    );
  }

  #drawHp(
    rgba: Uint8Array | null,
    frameIndex: number,
    width: number,
    height: number,
  ) {
    if (!rgba) {
      drawHpTimeline(
        this.#context,
        this.data.hpFeatures,
        frameIndex,
        width,
        height,
      );
      return;
    }
    const hp = this.inspector.inspectHp(rgba, width, height);
    const frame = this.data.hpFeatures[frameIndex];
    if (frame?.is_match_screen) applyAnalyzedHp(hp, frame, this.ownSide);
    drawHpRoiOverlay(this.#context, this.hpGeometry, width, height);
    drawHpBarReproduced(this.#context, this.hpGeometry, hp, width, height);
    drawHpColActive(this.#context, this.hpGeometry, hp, width, height);
    drawHpTimeline(
      this.#context,
      this.data.hpFeatures,
      frameIndex,
      width,
      height,
    );
  }

  #drawDrive(
    rgba: Uint8Array,
    frameIndex: number,
    width: number,
    height: number,
  ) {
    const drive = this.inspector.inspectDrive(rgba, width, height);
    const frame = this.data.hpFeatures[frameIndex];
    if (frame?.is_match_screen) {
      applyAnalyzedDrive(
        drive.left,
        frame.left_drive_ratio,
        frame.left_burnout,
        frame.left_drive_uncertain,
      );
      applyAnalyzedDrive(
        drive.right,
        frame.right_drive_ratio,
        frame.right_burnout,
        frame.right_drive_uncertain,
      );
    }
    drawDriveRoiOverlay(this.#context, drive, width, height);
    drawDriveBarReproduced(this.#context, drive, width, height);
    drawDriveColRow(this.#context, drive, width, height);
    drawDriveTimeline(
      this.#context,
      this.data.hpFeatures,
      frameIndex,
      width,
      height,
    );
  }

  #drawSuper(
    rgba: Uint8Array,
    frameIndex: number,
    width: number,
    height: number,
  ) {
    const superGauge = this.inspector.inspectSuper(rgba, width, height);
    const frame = this.data.hpFeatures[frameIndex];
    drawSuperRoiOverlay(this.#context, superGauge, width, height);
    drawSuperGaugeReproduced(this.#context, superGauge, frame, width, height);
    drawSuperTimeline(
      this.#context,
      this.data.hpFeatures,
      frameIndex,
      width,
      height,
    );
  }

  #drawFrameLabel(frameIndex: number, height: number) {
    const time = frameToSeconds(frameIndex, this.data.frameTimestamps).toFixed(
      3,
    );
    this.#context.fillStyle = "rgb(0,220,220)";
    this.#context.font = `${Math.round(height / 45)}px monospace`;
    this.#context.fillText(
      `frame ${frameIndex}/${this.#totalFrames - 1}  ${time}s`,
      10,
      Math.round(height / 45) + 4,
    );
    this.onFrameInfo(`f${frameIndex} / ${time}s`);
  }
}

function applyAnalyzedHp(
  hp: DebugHpInspection,
  frame: DebugViewerData["hpFeatures"][number],
  ownSide: AnalysisSide,
) {
  const leftFill = ownSide === "p1" ? frame.own_hp : frame.opponent_hp;
  const rightFill = ownSide === "p1" ? frame.opponent_hp : frame.own_hp;
  if (leftFill >= 0) hp.left_fill = leftFill;
  if (rightFill >= 0) hp.right_fill = rightFill;

  const leftWidth = hp.left_col_yellow.length || 1;
  const rightWidth = hp.right_col_yellow.length || 1;
  const leftStart = leftWidth - Math.round(hp.left_fill * leftWidth);
  const rightEnd = Math.round(hp.right_fill * rightWidth);
  hp.left_yellow_fill =
    countTrue(hp.left_col_yellow.slice(leftStart)) / leftWidth;
  hp.right_yellow_fill =
    countTrue(hp.right_col_yellow.slice(0, rightEnd)) / rightWidth;
}

function applyAnalyzedDrive(
  side: DebugDriveSideInspection,
  ratio: number,
  burnout: boolean,
  uncertain: boolean,
) {
  side.burnout = burnout;
  side.recovery = burnout ? ratio : 0;
  side.value = burnout ? 0 : ratio * 6;
  side.uncertain = uncertain;
}

function countTrue(values: readonly boolean[]): number {
  return values.filter(Boolean).length;
}
