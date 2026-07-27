import type { Comp, SpecsOf } from "../common";
import { defineSpecs } from "../common";
import type { DeletePasswordHash, ShareId } from "./model";

export interface PublishedAnalysisLifecycle {
  readonly id: ShareId;
  readonly deletePasswordHash: DeletePasswordHash | null;
  readonly createdAt: Date;
  readonly expiresAt: Date;
}

export namespace PublishedAnalysisLifecycle {
  export type SortKey = "createdAt" | "expiresAt" | "id";

  const specs = defineSpecs({
    ById: (id: ShareId) => ({ id }),
    ByIds: (...ids: ShareId[]) => ({ ids }),
    ActiveAt: (at: Date) => ({ at }),
    ExpiredAt: (at: Date) => ({ at }),
    CreatedAtOrBefore: (cutoff: Date) => ({ cutoff }),
  });

  export const ById = specs.ById;
  export const ByIds = specs.ByIds;
  export const ActiveAt = specs.ActiveAt;
  export const ExpiredAt = specs.ExpiredAt;
  export const CreatedAtOrBefore = specs.CreatedAtOrBefore;
  export type SpecData = SpecsOf<typeof specs>;
  export type Spec = Comp<SpecData>;

  export const defaultSort = {
    keys: ["expiresAt", "id"] as const,
    order: "asc" as const,
  };

  export function cursor(
    lifecycle: PublishedAnalysisLifecycle,
    keys: readonly SortKey[],
  ): Record<string, string> {
    const result: Record<string, string> = {};
    for (const key of keys) {
      const value = lifecycle[key];
      result[key] = value instanceof Date ? value.toISOString() : String(value);
    }
    return result;
  }
}
