export function mp4TimestampUs(cts: number, timescale: number): number {
  if (!Number.isFinite(cts) || !Number.isFinite(timescale) || timescale <= 0) {
    throw new RangeError(
      "cts and timescale must be finite, and timescale must be positive",
    );
  }
  return Math.trunc((cts * 1_000_000) / timescale);
}

export class SampleTimestampIndex {
  private readonly sampleByTimestamp = new Map<number, number>();

  add(timestampUs: number, sampleIndex: number): void {
    this.sampleByTimestamp.set(timestampUs, sampleIndex);
  }

  resolve(timestampUs: number): number {
    return this.sampleByTimestamp.get(timestampUs) ?? -1;
  }
}
