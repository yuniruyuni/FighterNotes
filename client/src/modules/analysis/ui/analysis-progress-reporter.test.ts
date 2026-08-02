import { describe, expect, test } from "bun:test";
import {
  AnalysisProgressReporter,
  type AnalysisProgressScheduler,
} from "./analysis-progress-reporter.js";

class FakeScheduler implements AnalysisProgressScheduler {
  time = 0;
  timer: { callback: () => void; at: number } | null = null;

  now(): number {
    return this.time;
  }

  setTimer(
    callback: () => void,
    delayMs: number,
  ): ReturnType<typeof setTimeout> {
    this.timer = { callback, at: this.time + delayMs };
    return 1 as unknown as ReturnType<typeof setTimeout>;
  }

  clearTimer(): void {
    this.timer = null;
  }

  advance(milliseconds: number): void {
    this.time += milliseconds;
    const timer = this.timer;
    if (!timer || timer.at > this.time) return;
    this.timer = null;
    timer.callback();
  }
}

describe("AnalysisProgressReporter", () => {
  test("全フレームの値を受け取りながらReact通知を時間上限で集約する", () => {
    const scheduler = new FakeScheduler();
    const notifications: Array<[number, string]> = [];
    const reporter = new AnalysisProgressReporter(
      (progress, status) => notifications.push([progress, status]),
      scheduler,
      100,
    );

    for (let frame = 1; frame <= 10_000; frame += 1) {
      reporter.report((frame / 10_000) * 0.9, `フレーム ${frame} / 10000`);
    }

    expect(notifications).toEqual([[0.00009, "フレーム 1 / 10000"]]);
    expect(scheduler.timer).not.toBeNull();
    scheduler.advance(99);
    expect(notifications).toHaveLength(1);
    scheduler.advance(1);
    expect(notifications).toEqual([
      [0.00009, "フレーム 1 / 10000"],
      [0.9, "フレーム 10000 / 10000"],
    ]);
  });

  test("工程変更と100%を即時通知し、進捗を後退させない", () => {
    const scheduler = new FakeScheduler();
    const notifications: Array<[number, string]> = [];
    const reporter = new AnalysisProgressReporter(
      (progress, status) => notifications.push([progress, status]),
      scheduler,
      100,
    );

    reporter.report(0.4, "フレーム 400 / 1000");
    reporter.report(0.3, "フレーム 401 / 1000");
    reporter.report(0.9, "位置関係 1 / 20");
    reporter.report(1, "レポート生成中…");

    expect(notifications).toEqual([
      [0.4, "フレーム 400 / 1000"],
      [0.9, "位置関係 1 / 20"],
      [1, "レポート生成中…"],
    ]);
    expect(scheduler.timer).toBeNull();
  });

  test("完了を補完し、dispose後の遅延通知を止める", () => {
    const scheduler = new FakeScheduler();
    const notifications: Array<[number, string]> = [];
    const reporter = new AnalysisProgressReporter(
      (progress, status) => notifications.push([progress, status]),
      scheduler,
      100,
    );

    reporter.report(0.2, "フレーム 1 / 10");
    reporter.finish();
    reporter.finish();
    reporter.dispose();
    reporter.report(0.8, "フレーム 8 / 10");
    scheduler.advance(100);

    expect(notifications).toEqual([
      [0.2, "フレーム 1 / 10"],
      [1, "解析完了"],
    ]);
  });
});
