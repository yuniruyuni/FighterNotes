// SA ゲージの生読み・確定値・時間推移を比較するデバッグオーバーレイ。

import type { HpFrameData } from "~/modules/analysis/contracts.js";
import type {
  DebugSuperInspection,
  DebugSuperRoi,
  DebugSuperSideInspection,
} from "../../../../application/debug-frame-inspection.js";
import { METER_X1, METER_X2 } from "./meter.js";

const SUPER_TIMELINE_Y = 214;
const SUPER_TIMELINE_ROW_H = 10;
const SUPER_TIMELINE_GAP = 2;
const SUPER_TIMELINE_WINDOW = 150;

const SUPER_PANEL_Y = 1037;
const SUPER_PANEL_BAR_H = 14;
const SUPER_PANEL_GAP = 2;
const SUPER_PANEL_X = { left: 20, right: 1570 } as const;
const SUPER_PANEL_W = 330;

export function drawSuperRoiOverlay(
  context: CanvasRenderingContext2D,
  inspection: DebugSuperInspection,
  canvasWidth: number,
  canvasHeight: number,
): void {
  const scaleX = canvasWidth / 1920;
  const scaleY = canvasHeight / 1080;
  for (const [side, data] of [
    ["P1", inspection.left],
    ["P2", inspection.right],
  ] as const) {
    drawRoi(
      context,
      data.label_roi,
      "rgba(255,220,70,0.95)",
      `${side} SA label`,
      scaleX,
      scaleY,
    );
    drawRoi(
      context,
      data.bar_roi,
      side === "P1" ? "rgba(255,60,180,0.95)" : "rgba(50,190,255,0.95)",
      `${side} SA bar`,
      scaleX,
      scaleY,
    );
  }
}

function drawRoi(
  context: CanvasRenderingContext2D,
  roi: DebugSuperRoi,
  color: string,
  label: string,
  scaleX: number,
  scaleY: number,
): void {
  const x = roi.x1 * scaleX;
  const y = roi.y1 * scaleY;
  const width = (roi.x2 - roi.x1) * scaleX;
  const height = (roi.y2 - roi.y1) * scaleY;
  context.strokeStyle = color;
  context.lineWidth = 2;
  context.strokeRect(x, y, width, height);
  context.fillStyle = color;
  context.font = `bold ${Math.round(10 * scaleY)}px monospace`;
  context.fillText(label, x, y - 3 * scaleY);
}

export function drawSuperGaugeReproduced(
  context: CanvasRenderingContext2D,
  inspection: DebugSuperInspection,
  frame: HpFrameData | undefined,
  canvasWidth: number,
  canvasHeight: number,
): void {
  const scaleX = canvasWidth / 1920;
  const scaleY = canvasHeight / 1080;
  drawSidePanel(
    context,
    "left",
    "P1",
    inspection.left,
    frame?.left_super_value ?? 0,
    frame?.left_super_uncertain ?? true,
    frame?.left_ca_ready ?? false,
    scaleX,
    scaleY,
  );
  drawSidePanel(
    context,
    "right",
    "P2",
    inspection.right,
    frame?.right_super_value ?? 0,
    frame?.right_super_uncertain ?? true,
    frame?.right_ca_ready ?? false,
    scaleX,
    scaleY,
  );
}

function drawSidePanel(
  context: CanvasRenderingContext2D,
  side: "left" | "right",
  label: string,
  raw: DebugSuperSideInspection,
  finalValue: number,
  finalUncertain: boolean,
  finalCaReady: boolean,
  scaleX: number,
  scaleY: number,
): void {
  const x = SUPER_PANEL_X[side] * scaleX;
  const width = SUPER_PANEL_W * scaleX;
  const rawY = SUPER_PANEL_Y * scaleY;
  const finalY = (SUPER_PANEL_Y + SUPER_PANEL_BAR_H + SUPER_PANEL_GAP) * scaleY;
  const height = SUPER_PANEL_BAR_H * scaleY;
  const anchorRight = side === "right";
  const color =
    side === "left" ? "rgba(235,35,150,0.90)" : "rgba(30,170,245,0.90)";
  const rawLevel = raw.critical_art
    ? "CA"
    : (raw.displayed_level?.toString() ?? "?");

  drawGaugeBar(
    context,
    x,
    rawY,
    width,
    height,
    raw.value,
    raw.uncertain,
    raw.critical_art,
    anchorRight,
    color,
    `${label} raw ${formatValue(raw.value, raw.uncertain)} [${rawLevel}]`,
    scaleY,
  );
  drawGaugeBar(
    context,
    x,
    finalY,
    width,
    height,
    finalValue,
    finalUncertain,
    finalCaReady,
    anchorRight,
    color,
    `${label} final ${formatValue(finalValue, finalUncertain)}${finalCaReady ? " CA" : ""}`,
    scaleY,
  );
}

function drawGaugeBar(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  value: number,
  uncertain: boolean,
  caReady: boolean,
  anchorRight: boolean,
  color: string,
  label: string,
  scaleY: number,
): void {
  context.fillStyle = "rgba(14,14,14,0.92)";
  context.fillRect(x, y, width, height);
  const fillWidth = (Math.max(0, Math.min(3, value)) / 3) * width;
  context.fillStyle = uncertain
    ? "rgba(105,105,105,0.70)"
    : caReady
      ? "rgba(255,210,45,0.92)"
      : color;
  context.fillRect(
    anchorRight ? x + width - fillWidth : x,
    y,
    fillWidth,
    height,
  );

  context.strokeStyle = "rgba(255,255,255,0.38)";
  context.lineWidth = 1;
  for (let stock = 1; stock < 3; stock++) {
    const stockX = x + (width * stock) / 3;
    context.beginPath();
    context.moveTo(stockX, y);
    context.lineTo(stockX, y + height);
    context.stroke();
  }

  context.fillStyle = uncertain ? "rgb(255,185,100)" : "rgba(255,255,255,0.96)";
  context.font = `bold ${Math.round(10 * scaleY)}px monospace`;
  context.fillText(label, x + 4, y + height * 0.76);
}

function formatValue(value: number, uncertain: boolean): string {
  return `${value.toFixed(2)}${uncertain ? " ?" : ""}`;
}

export function drawSuperTimeline(
  context: CanvasRenderingContext2D,
  frames: HpFrameData[],
  frameIndex: number,
  canvasWidth: number,
  canvasHeight: number,
): void {
  const scaleX = canvasWidth / 1920;
  const scaleY = canvasHeight / 1080;
  const x1 = METER_X1 * scaleX;
  const x2 = METER_X2 * scaleX;
  const width = x2 - x1;
  const columnCount = SUPER_TIMELINE_WINDOW * 2;
  const columnWidth = width / columnCount;
  const leftY = SUPER_TIMELINE_Y * scaleY;
  const rightY =
    (SUPER_TIMELINE_Y + SUPER_TIMELINE_ROW_H + SUPER_TIMELINE_GAP) * scaleY;
  const rowHeight = SUPER_TIMELINE_ROW_H * scaleY;

  context.fillStyle = "rgba(14,14,14,0.92)";
  context.fillRect(
    x1 - 40 * scaleX,
    leftY - 2 * scaleY,
    width + 44 * scaleX,
    (SUPER_TIMELINE_ROW_H * 2 + SUPER_TIMELINE_GAP + 4) * scaleY,
  );

  for (
    let delta = -SUPER_TIMELINE_WINDOW;
    delta < SUPER_TIMELINE_WINDOW;
    delta++
  ) {
    const targetFrame = frameIndex + delta;
    const x = x1 + (delta + SUPER_TIMELINE_WINDOW) * columnWidth;
    const drawWidth = Math.max(1, columnWidth);
    const frame = frames[targetFrame];
    if (!frame) {
      context.fillStyle = "#0a0a0a";
      context.fillRect(x, leftY, drawWidth, rowHeight);
      context.fillRect(x, rightY, drawWidth, rowHeight);
      continue;
    }
    context.fillStyle = superColor(
      frame.left_super_value,
      frame.left_super_uncertain,
      frame.left_ca_ready,
      frame.is_match_screen,
      "left",
    );
    context.fillRect(x, leftY, drawWidth, rowHeight);
    context.fillStyle = superColor(
      frame.right_super_value,
      frame.right_super_uncertain,
      frame.right_ca_ready,
      frame.is_match_screen,
      "right",
    );
    context.fillRect(x, rightY, drawWidth, rowHeight);
  }

  const markerX = x1 + SUPER_TIMELINE_WINDOW * columnWidth;
  context.strokeStyle = "white";
  context.lineWidth = 1;
  context.beginPath();
  context.moveTo(markerX, leftY);
  context.lineTo(markerX, rightY + rowHeight);
  context.stroke();

  context.fillStyle = "rgb(200,200,200)";
  context.font = `${Math.round(10 * scaleY)}px monospace`;
  context.fillText("SA L", x1 - 38 * scaleX, leftY + rowHeight * 0.72);
  context.fillText("SA R", x1 - 38 * scaleX, rightY + rowHeight * 0.72);
}

function superColor(
  value: number,
  uncertain: boolean,
  caReady: boolean,
  isMatchScreen: boolean,
  side: "left" | "right",
): string {
  if (!isMatchScreen) return "#1a1a1a";
  if (uncertain) return "#4b4b4b";
  if (caReady) return "rgb(235,190,30)";
  const lightness = 24 + (Math.max(0, Math.min(3, value)) / 3) * 34;
  return side === "left"
    ? `hsl(326,82%,${lightness}%)`
    : `hsl(198,82%,${lightness}%)`;
}
