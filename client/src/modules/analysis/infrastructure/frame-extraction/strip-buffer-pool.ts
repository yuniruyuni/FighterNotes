import { ANALYSIS_STRIPS } from "./layout.js";

export interface AnalysisTransferBuffers {
  readonly hud: ArrayBuffer;
  readonly meter: ArrayBuffer;
  readonly input: ArrayBuffer;
}

export class TransferBufferPool {
  readonly #slots: AnalysisTransferBuffers[];
  readonly #freeSlots: number[];
  readonly #waiters: Array<(slot: number) => void> = [];

  constructor(slotCount = 2) {
    this.#slots = Array.from({ length: slotCount }, createBuffers);
    this.#freeSlots = Array.from({ length: slotCount }, (_, index) => index);
  }

  acquire(): Promise<number> {
    const slot = this.#freeSlots.pop();
    if (slot !== undefined) return Promise.resolve(slot);
    return new Promise((resolve) => this.#waiters.push(resolve));
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
    if (waiter) waiter(slot);
    else this.#freeSlots.push(slot);
  }
}

function createBuffers(): AnalysisTransferBuffers {
  return {
    hud: new ArrayBuffer(ANALYSIS_STRIPS.hud.byteLength),
    meter: new ArrayBuffer(ANALYSIS_STRIPS.meter.byteLength),
    input: new ArrayBuffer(ANALYSIS_STRIPS.input.byteLength),
  };
}
