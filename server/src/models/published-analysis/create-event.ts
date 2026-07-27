import type { Comp, SpecsOf } from "../common";
import { defineSpecs } from "../common";
import type { ShareId } from "./model";

export interface PublishedAnalysisCreateEvent {
  readonly analysisId: ShareId;
  readonly createdAt: Date;
}

export namespace PublishedAnalysisCreateEvent {
  const specs = defineSpecs({
    ByAnalysisId: (analysisId: ShareId) => ({ analysisId }),
    CreatedAtOrAfter: (start: Date) => ({ start }),
    CreatedBefore: (cutoff: Date) => ({ cutoff }),
  });

  export const ByAnalysisId = specs.ByAnalysisId;
  export const CreatedAtOrAfter = specs.CreatedAtOrAfter;
  export const CreatedBefore = specs.CreatedBefore;
  export type SpecData = SpecsOf<typeof specs>;
  export type Spec = Comp<SpecData>;
}

export function startOfUtcDay(value: Date): Date {
  return new Date(
    Date.UTC(value.getUTCFullYear(), value.getUTCMonth(), value.getUTCDate()),
  );
}
