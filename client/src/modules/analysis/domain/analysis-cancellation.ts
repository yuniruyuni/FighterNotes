export class AnalysisCanceledError extends Error {
  constructor(message = "動画解析を中止しました") {
    super(message);
    this.name = "AnalysisCanceledError";
  }
}

export function isAnalysisCanceled(error: unknown): boolean {
  return error instanceof AnalysisCanceledError;
}
