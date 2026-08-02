import { abortReason } from "./abort.js";

/**
 * Links a caller-owned signal with an internal abort source such as a stall
 * watchdog. AbortController keeps the first reason, which makes races
 * deterministic.
 */
export class LinkedAbortController {
  readonly #controller = new AbortController();
  readonly #source: AbortSignal;
  readonly #forwardSourceAbort: () => void;
  #disposed = false;

  constructor(source: AbortSignal) {
    this.#source = source;
    this.#forwardSourceAbort = () => {
      this.abort(abortReason(source));
    };
    source.addEventListener("abort", this.#forwardSourceAbort, { once: true });
    if (source.aborted) this.#forwardSourceAbort();
  }

  get signal(): AbortSignal {
    return this.#controller.signal;
  }

  abort(reason: Error): void {
    this.#controller.abort(reason);
  }

  dispose(): void {
    if (this.#disposed) return;
    this.#disposed = true;
    this.#source.removeEventListener("abort", this.#forwardSourceAbort);
  }
}
