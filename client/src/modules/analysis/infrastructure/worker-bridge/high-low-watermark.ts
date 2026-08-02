interface AdmissionWaiter {
  readonly resolve: () => void;
  readonly reject: (reason: unknown) => void;
  readonly signal?: AbortSignal;
  readonly onAbort?: () => void;
}

/**
 * Reserves capacity before resolving acquire(). Once the high watermark is
 * reached, new admissions remain paused until active work falls to low.
 */
export class HighLowWatermarkGate {
  readonly #high: number;
  readonly #low: number;
  readonly #waiters: AdmissionWaiter[] = [];
  #active = 0;
  #peak = 0;
  #paused = false;
  #closed: { readonly reason: unknown } | undefined;

  constructor(high: number, low: number) {
    if (!Number.isInteger(high) || high <= 0) {
      throw new Error("high watermark must be a positive integer");
    }
    if (!Number.isInteger(low) || low < 0 || low >= high) {
      throw new Error("low watermark must be an integer below high");
    }
    this.#high = high;
    this.#low = low;
  }

  get active(): number {
    return this.#active;
  }

  get peak(): number {
    return this.#peak;
  }

  acquire(signal?: AbortSignal): Promise<void> {
    if (this.#closed) return Promise.reject(this.#closed.reason);
    if (signal?.aborted) return Promise.reject(abortReason(signal));
    if (!this.#paused && this.#waiters.length === 0) {
      this.#reserve();
      return Promise.resolve();
    }

    return new Promise<void>((resolve, reject) => {
      const waiter: AdmissionWaiter = {
        resolve,
        reject,
        ...(signal ? { signal } : {}),
        ...(signal
          ? {
              onAbort: () => {
                const index = this.#waiters.indexOf(waiter);
                if (index < 0) return;
                this.#waiters.splice(index, 1);
                reject(abortReason(signal));
                this.#admitWaiters();
              },
            }
          : {}),
      };
      this.#waiters.push(waiter);
      signal?.addEventListener("abort", waiter.onAbort!, { once: true });
      if (signal?.aborted) waiter.onAbort?.();
    });
  }

  release(): void {
    if (this.#closed) return;
    if (this.#active <= 0) {
      throw new Error("watermark gate released without an active admission");
    }
    this.#active -= 1;
    this.#admitWaiters();
  }

  close(reason: unknown): void {
    if (this.#closed) return;
    this.#closed = { reason };
    this.#active = 0;
    for (const waiter of this.#waiters.splice(0)) {
      this.#removeAbortListener(waiter);
      waiter.reject(reason);
    }
  }

  #reserve(): void {
    this.#active += 1;
    this.#peak = Math.max(this.#peak, this.#active);
    if (this.#active >= this.#high) this.#paused = true;
  }

  #admitWaiters(): void {
    if (this.#closed) return;
    if (this.#paused) {
      if (this.#active > this.#low) return;
      this.#paused = false;
    }
    while (!this.#paused && this.#waiters.length > 0) {
      const waiter = this.#waiters.shift()!;
      this.#removeAbortListener(waiter);
      this.#reserve();
      waiter.resolve();
    }
  }

  #removeAbortListener(waiter: AdmissionWaiter): void {
    if (waiter.signal && waiter.onAbort) {
      waiter.signal.removeEventListener("abort", waiter.onAbort);
    }
  }
}

function abortReason(signal: AbortSignal): unknown {
  return signal.reason instanceof Error
    ? signal.reason
    : new Error("処理を中断しました");
}
