export interface BlobRangeReaderStats {
  readonly readCount: number;
  readonly totalBytesRead: number;
  readonly peakReadBytes: number;
}

export interface BlobSliceSource {
  readonly size: number;
  slice(start?: number, end?: number): Blob;
}

interface BlobRangeReaderDependencies {
  readonly readSlice?: (
    slice: Blob,
    signal: AbortSignal,
  ) => Promise<ArrayBuffer>;
}

/** Reads one bounded Blob range at a time and owns cancellation of that read. */
export class BlobRangeReader {
  readonly #source: BlobSliceSource;
  readonly #externalSignal: AbortSignal | undefined;
  readonly #readSlice: (
    slice: Blob,
    signal: AbortSignal,
  ) => Promise<ArrayBuffer>;
  readonly #controller = new AbortController();
  #reading = false;
  #readCount = 0;
  #totalBytesRead = 0;
  #peakReadBytes = 0;

  constructor(
    source: BlobSliceSource,
    signal?: AbortSignal,
    dependencies: BlobRangeReaderDependencies = {},
  ) {
    this.#source = source;
    this.#externalSignal = signal;
    this.#readSlice = dependencies.readSlice ?? readBlobSlice;
  }

  get statistics(): BlobRangeReaderStats {
    return {
      readCount: this.#readCount,
      totalBytesRead: this.#totalBytesRead,
      peakReadBytes: this.#peakReadBytes,
    };
  }

  async read(offset: number, size: number): Promise<ArrayBuffer> {
    assertRange(offset, size, this.#source.size);
    this.#throwIfStopped();
    if (this.#reading) {
      throw new Error("Blob range reader does not allow concurrent reads");
    }
    if (size === 0) return new ArrayBuffer(0);

    this.#reading = true;
    this.#peakReadBytes = Math.max(this.#peakReadBytes, size);
    const linked = linkAbortSignals(
      this.#controller.signal,
      this.#externalSignal,
    );
    try {
      const buffer = await this.#readSlice(
        this.#source.slice(offset, offset + size),
        linked.signal,
      );
      this.#throwIfStopped();
      if (buffer.byteLength !== size) {
        throw new Error(
          `Blob range read returned ${buffer.byteLength} bytes; expected ${size}`,
        );
      }
      this.#readCount += 1;
      this.#totalBytesRead += buffer.byteLength;
      return buffer;
    } finally {
      linked.dispose();
      this.#reading = false;
    }
  }

  stop(reason: unknown = new Error("Blob range reader stopped")): void {
    if (this.#controller.signal.aborted) return;
    this.#controller.abort(reason);
  }

  #throwIfStopped(): void {
    const signal = this.#controller.signal.aborted
      ? this.#controller.signal
      : this.#externalSignal;
    if (!signal?.aborted) return;
    throw abortReason(signal);
  }
}

function assertRange(offset: number, size: number, sourceSize: number): void {
  if (!Number.isSafeInteger(offset) || offset < 0) {
    throw new Error("Blob range offset must be a non-negative safe integer");
  }
  if (!Number.isSafeInteger(size) || size < 0) {
    throw new Error("Blob range size must be a non-negative safe integer");
  }
  if (offset > sourceSize || size > sourceSize - offset) {
    throw new Error("Blob range is outside the source");
  }
}

function linkAbortSignals(
  owned: AbortSignal,
  external: AbortSignal | undefined,
): { readonly signal: AbortSignal; readonly dispose: () => void } {
  const controller = new AbortController();
  const signals = external ? [owned, external] : [owned];
  const listeners = signals.map((signal) => {
    const onAbort = () => {
      if (!controller.signal.aborted) controller.abort(abortReason(signal));
    };
    signal.addEventListener("abort", onAbort, { once: true });
    if (signal.aborted) onAbort();
    return { signal, onAbort };
  });
  return {
    signal: controller.signal,
    dispose() {
      for (const { signal, onAbort } of listeners) {
        signal.removeEventListener("abort", onAbort);
      }
    },
  };
}

function readBlobSlice(slice: Blob, signal: AbortSignal): Promise<ArrayBuffer> {
  return new Promise<ArrayBuffer>((resolve, reject) => {
    const reader = new FileReader();
    let settled = false;
    const cleanup = () => {
      signal.removeEventListener("abort", onAbort);
      reader.onload = null;
      reader.onerror = null;
      reader.onabort = null;
    };
    const settle = (callback: () => void) => {
      if (settled) return;
      settled = true;
      cleanup();
      callback();
    };
    const onAbort = () => {
      if (reader.readyState === FileReader.LOADING) reader.abort();
      settle(() => reject(abortReason(signal)));
    };
    reader.onload = () => {
      const result = reader.result;
      settle(() => {
        if (result instanceof ArrayBuffer) resolve(result);
        else reject(new Error("Blob range could not be read"));
      });
    };
    reader.onerror = () =>
      settle(() =>
        reject(reader.error ?? new Error("Blob range could not be read")),
      );
    reader.onabort = onAbort;
    signal.addEventListener("abort", onAbort, { once: true });
    if (signal.aborted) {
      onAbort();
      return;
    }
    try {
      reader.readAsArrayBuffer(slice);
    } catch (error) {
      settle(() => reject(error));
    }
  });
}

function abortReason(signal: AbortSignal): unknown {
  return signal.reason instanceof Error
    ? signal.reason
    : new DOMException("Blob range read aborted", "AbortError");
}
