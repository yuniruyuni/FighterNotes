type TimerHandle = ReturnType<typeof setTimeout>;

export interface AnalysisWatchdogHost {
  visibilityState(): DocumentVisibilityState;
  setTimer(callback: () => void, delayMs: number): TimerHandle;
  clearTimer(handle: TimerHandle): void;
  subscribeVisibility(callback: () => void): () => void;
}

export class AnalysisProgressWatchdog {
  readonly #host: AnalysisWatchdogHost;
  readonly #timeoutMs: number;
  readonly #onStall: () => void;
  readonly #unsubscribe: () => void;
  #timer: TimerHandle | null = null;
  #disposed = false;

  constructor(
    host: AnalysisWatchdogHost,
    timeoutMs: number,
    onStall: () => void,
  ) {
    this.#host = host;
    this.#timeoutMs = timeoutMs;
    this.#onStall = onStall;
    this.#unsubscribe = host.subscribeVisibility(() => this.pulse());
    this.pulse();
  }

  pulse(): void {
    if (this.#disposed) return;
    this.#clearTimer();
    if (this.#host.visibilityState() !== "visible") return;
    this.#timer = this.#host.setTimer(() => {
      this.#timer = null;
      this.#disposed = true;
      this.#unsubscribe();
      this.#onStall();
    }, this.#timeoutMs);
  }

  dispose(): void {
    if (this.#disposed) return;
    this.#disposed = true;
    this.#clearTimer();
    this.#unsubscribe();
  }

  #clearTimer(): void {
    if (this.#timer === null) return;
    this.#host.clearTimer(this.#timer);
    this.#timer = null;
  }
}

export function browserAnalysisWatchdogHost(
  target: Document = document,
): AnalysisWatchdogHost {
  return {
    visibilityState: () => target.visibilityState,
    setTimer: (callback, delayMs) => setTimeout(callback, delayMs),
    clearTimer: (handle) => clearTimeout(handle),
    subscribeVisibility(callback) {
      target.addEventListener("visibilitychange", callback);
      return () => target.removeEventListener("visibilitychange", callback);
    },
  };
}
