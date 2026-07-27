import { describe, expect, test } from "bun:test";
import {
  AnalysisProgressWatchdog,
  type AnalysisWatchdogHost,
} from "./progress-watchdog.js";

class FakeWatchdogHost implements AnalysisWatchdogHost {
  state: DocumentVisibilityState = "visible";
  callback: (() => void) | null = null;
  visibilityCallback: (() => void) | null = null;

  visibilityState(): DocumentVisibilityState {
    return this.state;
  }

  setTimer(callback: () => void): ReturnType<typeof setTimeout> {
    this.callback = callback;
    return 1 as unknown as ReturnType<typeof setTimeout>;
  }

  clearTimer(): void {
    this.callback = null;
  }

  subscribeVisibility(callback: () => void): () => void {
    this.visibilityCallback = callback;
    return () => {
      this.visibilityCallback = null;
    };
  }

  changeVisibility(state: DocumentVisibilityState): void {
    this.state = state;
    this.visibilityCallback?.();
  }

  expire(): void {
    const callback = this.callback;
    this.callback = null;
    callback?.();
  }
}

describe("analysis progress watchdog", () => {
  test("表示中に進捗が止まると通知する", () => {
    const host = new FakeWatchdogHost();
    let stalls = 0;
    new AnalysisProgressWatchdog(host, 30_000, () => {
      stalls += 1;
    });

    host.expire();
    expect(stalls).toBe(1);
  });

  test("非表示中は監視を止め、復帰時に監視時間を取り直す", () => {
    const host = new FakeWatchdogHost();
    let stalls = 0;
    const watchdog = new AnalysisProgressWatchdog(host, 30_000, () => {
      stalls += 1;
    });

    host.changeVisibility("hidden");
    host.expire();
    expect(stalls).toBe(0);

    host.changeVisibility("visible");
    expect(host.callback).not.toBeNull();
    watchdog.pulse();
    host.expire();
    expect(stalls).toBe(1);
  });
});
