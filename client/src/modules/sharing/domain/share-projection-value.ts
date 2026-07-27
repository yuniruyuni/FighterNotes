export const MAX_COUNT = 65_535;
export const MAX_ROUNDS = 255;
export const MAX_SEVERITY_BP = 1_000_000;
export const MAX_DURATION_DECISECONDS = 864_000;
export const MAX_HP_BP = 1_000_000;

export class ShareProjectionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ShareProjectionError";
  }
}

export function boundedInteger(
  value: number,
  max: number,
  field: string,
): number {
  assertNonNegativeFinite(value, field);
  return Math.min(max, Math.round(value));
}

export function scaledInteger(
  value: number,
  scale: number,
  max: number,
  field: string,
): number {
  assertNonNegativeFinite(value, field);
  return Math.min(max, Math.round(value * scale));
}

function assertNonNegativeFinite(value: number, field: string): void {
  if (!Number.isFinite(value) || value < 0) {
    throw new ShareProjectionError(`${field} が不正です。`);
  }
}
