export interface AnalysisTiming {
  readonly frameIndex: number;
  readonly tDraw: number;
  readonly tCopy: number;
  readonly tMeter: number;
  readonly tHud: number;
}

export function logPerformance(timing: AnalysisTiming): void {
  if (timing.frameIndex <= 0) return;
  const ms = (value: number) => `${value.toFixed(0)}ms`;
  console.log(
    `[perf] ${timing.frameIndex}f total:` +
      ` draw+get=${ms(timing.tDraw)} (${ms(timing.tDraw / timing.frameIndex)}/f)` +
      ` worker_copy=${ms(timing.tCopy)} (${ms(timing.tCopy / timing.frameIndex)}/f)` +
      ` meter=${ms(timing.tMeter)} (${ms(timing.tMeter / timing.frameIndex)}/f)` +
      ` hud=${ms(timing.tHud)} (${ms(timing.tHud / timing.frameIndex)}/f)`,
  );
}
