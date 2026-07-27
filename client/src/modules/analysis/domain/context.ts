export type AnalysisSide = "p1" | "p2";

export interface PlayerAnalysisContext {
  character?: string;
  controlType?: string;
}

/** Session metadata that cannot be inferred reliably from HUD strips. */
export interface AnalysisContext {
  ownSide: AnalysisSide;
  p1: PlayerAnalysisContext;
  p2: PlayerAnalysisContext;
  battleVersion?: string;
}

export interface AnalysisContextInput {
  ownSide?: string;
  p1?: PlayerAnalysisContext;
  p2?: PlayerAnalysisContext;
  battleVersion?: string;
}

export interface AnalysisContextOptions {
  ownControlType?: string;
  opponentControlType?: string;
  battleVersion?: string;
}

export function createAnalysisContext(
  ownSide: string,
  ownCharacter = "",
  opponentCharacter = "",
  options: AnalysisContextOptions = {},
): AnalysisContext {
  const side = normalizeSide(ownSide);
  const own = player(ownCharacter, options.ownControlType);
  const opponent = player(opponentCharacter, options.opponentControlType);
  return {
    ownSide: side,
    p1: side === "p1" ? own : opponent,
    p2: side === "p2" ? own : opponent,
    ...optional("battleVersion", options.battleVersion),
  };
}

/** Accepts the previous fourth argument (`ownChar`) as well as full P1/P2 metadata. */
export function resolveAnalysisContext(
  ownSide: string,
  input: string | AnalysisContextInput = "",
): AnalysisContext {
  if (typeof input === "string") {
    return createAnalysisContext(ownSide, input);
  }

  return {
    ownSide: normalizeSide(ownSide),
    p1: normalizePlayer(input.p1),
    p2: normalizePlayer(input.p2),
    ...optional("battleVersion", input.battleVersion),
  };
}

function normalizeSide(side: string): AnalysisSide {
  return side.toLowerCase() === "p2" ? "p2" : "p1";
}

function normalizePlayer(
  value: PlayerAnalysisContext | undefined,
): PlayerAnalysisContext {
  return player(value?.character, value?.controlType);
}

function player(
  character: string | undefined,
  controlType: string | undefined,
): PlayerAnalysisContext {
  return {
    ...optional("character", character),
    ...optional("controlType", controlType),
  };
}

function optional<K extends string>(
  key: K,
  value: string | undefined,
): Partial<Record<K, string>> {
  const normalized = value?.trim();
  return normalized ? ({ [key]: normalized } as Record<K, string>) : {};
}
