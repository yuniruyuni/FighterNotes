import { deepFreeze } from "../common/deep-freeze";
import type { Fail } from "../common/fail";
import { fail } from "../common/fail";
import type { Result } from "../common/result";
import type { Comp, SpecsOf } from "../common/spec";
import { defineSpecs } from "../common/spec";
import {
  FINDING_KINDS,
  PRESENTATION_REVISION,
  SCHEMA_VERSION,
} from "./catalog";
import {
  type PublishedAnalysisCandidate,
  publishedAnalysisCandidateSchema,
} from "./schema";

declare const SHARE_ID_BRAND: unique symbol;
declare const DELETE_PASSWORD_BRAND: unique symbol;
declare const DELETE_PASSWORD_HASH_BRAND: unique symbol;

export type ShareId = string & { readonly [SHARE_ID_BRAND]: true };
export type DeletePassword = string & {
  readonly [DELETE_PASSWORD_BRAND]: true;
};
export type DeletePasswordHash = string & {
  readonly [DELETE_PASSWORD_HASH_BRAND]: true;
};

export const MIN_DELETE_PASSWORD_LENGTH = 12;
export const MAX_DELETE_PASSWORD_LENGTH = 128;

export interface PublishedAnalysisContent extends PublishedAnalysisCandidate {
  readonly schemaVersion: typeof SCHEMA_VERSION;
  readonly presentationRevision: typeof PRESENTATION_REVISION;
}

export type PublishedFinding = PublishedAnalysisContent["findings"][number];

export interface PublishedAnalysis {
  readonly id: ShareId;
  readonly content: PublishedAnalysisContent;
  readonly createdAt: Date;
  readonly expiresAt: Date;
}

export namespace PublishedAnalysis {
  export type SortKey = "createdAt" | "expiresAt" | "id";

  const specs = defineSpecs({
    ById: (id: ShareId) => ({ id }),
    ActiveAt: (at: Date) => ({ at }),
  });

  export const ById = specs.ById;
  export const ActiveAt = specs.ActiveAt;
  export type SpecData = SpecsOf<typeof specs>;
  export type Spec = Comp<SpecData>;

  export const defaultSort = {
    keys: ["createdAt", "id"] as const,
    order: "desc" as const,
  };

  export function cursor(
    analysis: PublishedAnalysis,
    keys: readonly SortKey[],
  ): Record<string, string> {
    return cursorValues(analysis, keys);
  }
}

export interface PersistablePublishedAnalysis extends PublishedAnalysis {
  readonly deletePasswordHash: DeletePasswordHash;
  readonly logicalSizeBytes: number;
}

export interface CreatedPublishedAnalysis {
  readonly analysis: PersistablePublishedAnalysis;
}

const SHARE_ID_PATTERN = /^[A-Za-z0-9_-]{22}$/;

export function createPublishedAnalysisContent(
  input: unknown,
): Result<PublishedAnalysisContent, Fail> {
  const parsed = publishedAnalysisCandidateSchema.safeParse(input);
  if (!parsed.success) {
    return {
      ok: false,
      error: fail("INVALID_INPUT", "Invalid published analysis", {
        paths: parsed.error.issues.map((issue) => issue.path.join(".")),
      }),
    };
  }

  const order = new Map(FINDING_KINDS.map((kind, index) => [kind, index]));
  const findings = [...parsed.data.findings].sort(
    (left, right) =>
      (order.get(left.kind) ?? Number.MAX_SAFE_INTEGER) -
      (order.get(right.kind) ?? Number.MAX_SAFE_INTEGER),
  );

  return {
    ok: true,
    value: deepFreeze({
      schemaVersion: SCHEMA_VERSION,
      presentationRevision: PRESENTATION_REVISION,
      ...parsed.data,
      findings,
    }),
  };
}

export function createPersistablePublishedAnalysis(options: {
  id: ShareId;
  content: PublishedAnalysisContent;
  deletePasswordHash: DeletePasswordHash;
  now: Date;
  retentionDays: number;
}): CreatedPublishedAnalysis {
  const createdAt = new Date(options.now);
  const expiresAt = new Date(createdAt);
  expiresAt.setUTCDate(expiresAt.getUTCDate() + options.retentionDays);
  const logicalSizeBytes = new TextEncoder().encode(
    JSON.stringify({
      id: options.id,
      content: options.content,
      deletePasswordHash: options.deletePasswordHash,
      createdAt: createdAt.toISOString(),
      expiresAt: expiresAt.toISOString(),
    }),
  ).byteLength;
  if (logicalSizeBytes > MAX_ANALYSIS_LOGICAL_SIZE_BYTES) {
    throw new Error("Published analysis exceeds logical storage limit");
  }
  return {
    analysis: {
      id: options.id,
      content: options.content,
      deletePasswordHash: options.deletePasswordHash,
      createdAt,
      expiresAt,
      logicalSizeBytes,
    },
  };
}

export const MAX_ANALYSIS_LOGICAL_SIZE_BYTES = 8 * 1024;

export function parseShareId(value: string): ShareId | null {
  return SHARE_ID_PATTERN.test(value) ? (value as ShareId) : null;
}

export function parseDeletePassword(value: string): DeletePassword | null {
  return value.length >= MIN_DELETE_PASSWORD_LENGTH &&
    value.length <= MAX_DELETE_PASSWORD_LENGTH &&
    /\S/u.test(value)
    ? (value as DeletePassword)
    : null;
}

function cursorValues<T extends object, K extends keyof T>(
  value: T,
  keys: readonly K[],
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const key of keys) {
    const item = value[key];
    result[String(key)] =
      item instanceof Date ? item.toISOString() : String(item);
  }
  return result;
}
