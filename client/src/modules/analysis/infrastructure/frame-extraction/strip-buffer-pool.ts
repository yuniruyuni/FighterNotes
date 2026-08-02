import { ANALYSIS_STRIPS } from "./layout.js";

export interface AnalysisTransferBuffers {
  readonly hud: ArrayBuffer;
  readonly meter: ArrayBuffer;
  readonly input: ArrayBuffer;
}

interface BufferWaiter {
  readonly resolve: (slot: number) => void;
  readonly reject: (reason: unknown) => void;
  readonly signal?: AbortSignal;
  readonly onAbort?: () => void;
}

export class TransferBufferPool {
  readonly #slots: AnalysisTransferBuffers[];
  readonly #freeSlots: number[];
  readonly #waiters: BufferWaiter[] = [];

  constructor(slotCount = 2) {
    this.#slots = Array.from({ length: slotCount }, createBuffers);
    this.#freeSlots = Array.from({ length: slotCount }, (_, index) => index);
  }

  acquire(signal?: AbortSignal): Promise<number> {
    if (signal?.aborted) return Promise.reject(abortReason(signal));
    const slot = this.#freeSlots.pop();
    if (slot !== undefined) return Promise.resolve(slot);
    return new Promise((resolve, reject) => {
      let waiter: BufferWaiter;
      if (signal) {
        const onAbort = () => {
          const index = this.#waiters.indexOf(waiter);
          if (index >= 0) this.#waiters.splice(index, 1);
          reject(abortReason(signal));
        };
        waiter = { resolve, reject, signal, onAbort };
        signal.addEventListener("abort", onAbort, { once: true });
      } else {
        waiter = { resolve, reject };
      }
      this.#waiters.push(waiter);
    });
  }

  get(slot: number): AnalysisTransferBuffers {
    const buffers = this.#slots[slot];
    if (!buffers) throw new Error(`Unknown transfer buffer slot: ${slot}`);
    return buffers;
  }

  release(slot: number, buffers: AnalysisTransferBuffers): void {
    if (!this.#slots[slot]) {
      throw new Error(`Unknown transfer buffer slot: ${slot}`);
    }
    this.#slots[slot] = buffers;
    const waiter = this.#waiters.shift();
    if (waiter) {
      if (waiter.signal && waiter.onAbort) {
        waiter.signal.removeEventListener("abort", waiter.onAbort);
      }
      waiter.resolve(slot);
    } else this.#freeSlots.push(slot);
  }
}

function abortReason(signal: AbortSignal): Error {
  return signal.reason instanceof Error
    ? signal.reason
    : new Error("転送バッファーの待機を中止しました");
}

function createBuffers(): AnalysisTransferBuffers {
  return {
    hud: new ArrayBuffer(ANALYSIS_STRIPS.hud.byteLength),
    meter: new ArrayBuffer(ANALYSIS_STRIPS.meter.byteLength),
    input: new ArrayBuffer(ANALYSIS_STRIPS.input.byteLength),
  };
}
