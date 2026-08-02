import type { AnalysisProgress } from "../domain/result.js";

const DEFAULT_UPDATE_INTERVAL_MS = 100;

type TimerHandle = ReturnType<typeof setTimeout>;

export interface AnalysisProgressScheduler {
  now(): number;
  setTimer(callback: () => void, delayMs: number): TimerHandle;
  clearTimer(handle: TimerHandle): void;
}

interface PendingProgress {
  readonly progress: number;
  readonly status: string;
  readonly stage: string;
}

export class AnalysisProgressReporter {
  readonly #notify: AnalysisProgress;
  readonly #scheduler: AnalysisProgressScheduler;
  readonly #updateIntervalMs: number;
  #pending: PendingProgress | null = null;
  #timer: TimerHandle | null = null;
  #lastNotificationAt = Number.NEGATIVE_INFINITY;
  #lastProgress = 0;
  #lastStage = "";
  #disposed = false;

  constructor(
    notify: AnalysisProgress,
    scheduler: AnalysisProgressScheduler = browserProgressScheduler,
    updateIntervalMs = DEFAULT_UPDATE_INTERVAL_MS,
  ) {
    this.#notify = notify;
    this.#scheduler = scheduler;
    this.#updateIntervalMs = updateIntervalMs;
  }

  readonly report: AnalysisProgress = (progress, status) => {
    if (this.#disposed) return;
    const normalized = Math.max(
      this.#lastProgress,
      this.#pending?.progress ?? 0,
      clampProgress(progress),
    );
    const stage = analysisProgressStage(status);
    this.#pending = { progress: normalized, status, stage };
    const elapsed = this.#scheduler.now() - this.#lastNotificationAt;
    if (
      normalized >= 1 ||
      this.#lastStage !== stage ||
      elapsed >= this.#updateIntervalMs
    ) {
      this.#emitPending();
      return;
    }
    if (this.#timer === null) {
      this.#timer = this.#scheduler.setTimer(
        () => {
          this.#timer = null;
          this.#emitPending();
        },
        Math.max(0, this.#updateIntervalMs - elapsed),
      );
    }
  };

  finish(): void {
    if (this.#disposed || this.#lastProgress >= 1) return;
    this.#pending = {
      progress: 1,
      status: "解析完了",
      stage: "complete",
    };
    this.#emitPending();
  }

  dispose(): void {
    if (this.#disposed) return;
    this.#disposed = true;
    this.#pending = null;
    this.#clearTimer();
  }

  #emitPending(): void {
    const pending = this.#pending;
    if (!pending || this.#disposed) return;
    this.#pending = null;
    this.#clearTimer();
    this.#lastNotificationAt = this.#scheduler.now();
    this.#lastProgress = pending.progress;
    this.#lastStage = pending.stage;
    this.#notify(pending.progress, pending.status);
  }

  #clearTimer(): void {
    if (this.#timer === null) return;
    this.#scheduler.clearTimer(this.#timer);
    this.#timer = null;
  }
}

export function analysisProgressStage(status: string): string {
  if (/^フレーム \d+ \/ \d+/.test(status)) return "frames";
  if (/^位置関係 \d+ \/ \d+/.test(status)) return "spatial";
  if (status === "位置関係を確認中…") return "spatial";
  if (status === "レポート生成中…") return "report";
  if (status === "解析完了") return "complete";
  return status;
}

function clampProgress(progress: number): number {
  if (!Number.isFinite(progress)) return 0;
  return Math.max(0, Math.min(1, progress));
}

const browserProgressScheduler: AnalysisProgressScheduler = {
  now: () => performance.now(),
  setTimer: (callback, delayMs) => setTimeout(callback, delayMs),
  clearTimer: (handle) => clearTimeout(handle),
};
