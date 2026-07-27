import type { Comp, SpecsOf } from "../common";
import { defineSpecs } from "../common";

export interface PublishedAnalysisStorageUsage {
  readonly bytes: number;
}

export namespace PublishedAnalysisStorageUsage {
  const specs = defineSpecs({
    // Stryker disable next-line ArrowFunction: An empty specification has no payload beyond the type added by defineSpecs.
    Current: () => ({}),
  });

  export const Current = specs.Current;
  export type SpecData = SpecsOf<typeof specs>;
  export type Spec = Comp<SpecData>;
}
