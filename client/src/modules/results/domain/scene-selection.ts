import type { AdviceCard } from "~/modules/analysis/contracts.js";

export interface SceneSelection {
  key: number;
  frame: number;
  card: AdviceCard | null;
  label?: string;
  endFrame?: number;
}
