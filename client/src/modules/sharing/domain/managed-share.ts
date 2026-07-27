import { isGeneratedDeleteCode } from "./delete-code.js";
import { isShareId } from "./share.js";

export const MANAGED_SHARE_RECORD_VERSION = 1;
const MAX_LABEL_LENGTH = 80;

export interface ManagedShare {
  id: string;
  deleteCode: string;
  createdAt: string;
  expiresAt: string;
  label: string;
}

export interface StoredManagedShare extends ManagedShare {
  version: typeof MANAGED_SHARE_RECORD_VERSION;
}

export interface ManagedShareSnapshot {
  available: boolean;
  shares: ManagedShare[];
}

export const ManagedShare = {
  store(share: ManagedShare): StoredManagedShare | undefined {
    return ManagedShare.parse({
      ...share,
      version: MANAGED_SHARE_RECORD_VERSION,
    });
  },

  parse(value: unknown): StoredManagedShare | undefined {
    if (
      !isRecord(value) ||
      value.version !== MANAGED_SHARE_RECORD_VERSION ||
      typeof value.id !== "string" ||
      !isShareId(value.id) ||
      typeof value.deleteCode !== "string" ||
      !isGeneratedDeleteCode(value.deleteCode) ||
      typeof value.label !== "string" ||
      value.label.length === 0 ||
      value.label.length > MAX_LABEL_LENGTH ||
      !isIsoDate(value.createdAt) ||
      !isIsoDate(value.expiresAt)
    ) {
      return undefined;
    }

    return {
      version: MANAGED_SHARE_RECORD_VERSION,
      id: value.id,
      deleteCode: value.deleteCode,
      label: value.label,
      createdAt: value.createdAt,
      expiresAt: value.expiresAt,
    };
  },

  isExpired(share: ManagedShare, now: Date): boolean {
    return new Date(share.expiresAt).getTime() <= now.getTime();
  },
};

function isIsoDate(value: unknown): value is string {
  if (typeof value !== "string") return false;
  const date = new Date(value);
  return Number.isFinite(date.getTime()) && date.toISOString() === value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
