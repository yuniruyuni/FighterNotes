// Capcom 公式フレーム表の Next.js バンドルから、入力照合用データを抽出する。
// リモート JavaScript 自体は実行せず、埋め込まれた JSON.parse の文字列だけを読む。

export type StrikeKind = "high" | "overhead" | "low" | "air";

export type InputDirection =
  | "any"
  | "standing"
  | "neutral"
  | "down"
  | "horizontal"
  | "down_diagonal";

export interface AttackInputPattern {
  direction: InputDirection;
  buttons: string[];
  auto: boolean;
}

export interface AttackMoveData {
  startup: number;
  kind: StrikeKind;
  classic_inputs: AttackInputPattern[];
  modern_inputs: AttackInputPattern[];
}

interface OfficialFrameRow {
  webId?: string | number | null;
  skill?: string | null;
  type?: string | null;
  command?: string | null;
  command_modern?: string | null;
  attribute?: string | null;
  startup_frame?: string | null;
}

/** HTML から、キャラクターフレーム表を含むハッシュ付き chunk URL を得る。 */
export function frameBundlePath(html: string): string {
  const matches = [
    ...html.matchAll(
      /["'](\/6\/_next\/static\/chunks\/pages\/character\/[^"']+\/frame-[^"']+\.js)["']/g,
    ),
  ].map((match) => match[1]);
  const unique = [...new Set(matches)];
  if (unique.length !== 1) {
    throw new Error(`フレーム表 chunk を一意に特定できない (${unique.length})`);
  }
  return unique[0];
}

/** バンドル内の slug -> JSON データ変数の対応を使い、全キャラの行を返す。 */
export function parseOfficialFrameBundle(
  source: string,
): Record<string, OfficialFrameRow[]> {
  const assignments = parseJsonAssignments(source);
  const mapping = source.match(
    /\(\{((?:[a-z][a-z0-9_]*:[A-Za-z_$][\w$]*,?){10,})\}\)\[[^\]]+\]\.frame/,
  );
  if (!mapping)
    throw new Error("キャラクターとフレームデータの対応表が見つからない");

  const result: Record<string, OfficialFrameRow[]> = {};
  for (const pair of mapping[1].split(",")) {
    const [slug, variable] = pair.split(":");
    const value = assignments.get(variable) as
      | { frame?: OfficialFrameRow[] }
      | undefined;
    if (!slug || !value || !Array.isArray(value.frame)) {
      throw new Error(`フレームデータ変数を解決できない: ${pair}`);
    }
    result[slug] = value.frame;
  }
  return result;
}

/** 公式の生行を、解析器が直接照合できる最小カタログへ変換する。 */
export function buildAttackMoves(rows: OfficialFrameRow[]): AttackMoveData[] {
  const moves: AttackMoveData[] = [];
  for (const row of rows) {
    const startup = leadingNumber(row.startup_frame);
    const kind = strikeKind(row);
    if (startup === null || kind === null) continue;

    const airborne = kind === "air";
    const classicInputs = parseInputPatterns(row.command, "classic", airborne);
    const modernInputs = parseInputPatterns(
      row.command_modern,
      "modern",
      airborne,
    );
    if (classicInputs.length === 0 && modernInputs.length === 0) continue;

    moves.push({
      startup,
      kind,
      classic_inputs: classicInputs,
      modern_inputs: modernInputs,
    });
  }
  return moves;
}

export function parseInputPatterns(
  command: string | null | undefined,
  scheme: "classic" | "modern",
  airborne = false,
): AttackInputPattern[] {
  if (!command) return [];
  let normalized = command.normalize("NFKC").replace(/\s+/g, "");
  // 条件文は入力履歴の1行には現れない。入れ子を考慮して内側から除去する。
  let previous = "";
  while (previous !== normalized) {
    previous = normalized;
    normalized = normalized.replace(/\([^()]*\)/g, "");
  }
  // `A > B` は後続派生の行。ここでは被ダメージ列の最初の接触だけを
  // 照合するため、先頭入力を後続技の入力として流用しない。
  if (normalized.includes(">")) return [];
  normalized = normalized.replaceAll("ホールド", "");
  if (!normalized || normalized === "-" || normalized.includes("入力なし")) {
    return [];
  }

  const patterns: AttackInputPattern[] = [];
  for (const alternative of normalized.split("/")) {
    const parsed =
      scheme === "classic"
        ? classicButtons(alternative)
        : modernButtons(alternative);
    if (!parsed || parsed.variants.length === 0) continue;

    const directions = directionsFor(
      alternative.slice(0, parsed.start),
      airborne,
    );
    const auto = alternative.includes("AUTO");
    for (const direction of directions) {
      for (const buttons of parsed.variants) {
        patterns.push({ direction, buttons, auto });
      }
    }
  }
  return deduplicatePatterns(patterns);
}

function parseJsonAssignments(source: string): Map<string, unknown> {
  const values = new Map<string, unknown>();
  const assignment =
    /([A-Za-z_$][\w$]*)=JSON\.parse\(('(?:\\[\s\S]|[^'\\])*')\)/g;
  for (const match of source.matchAll(assignment)) {
    const decoded = decodeSingleQuoted(match[2].slice(1, -1));
    values.set(match[1], JSON.parse(decoded));
  }
  return values;
}

function decodeSingleQuoted(value: string): string {
  let decoded = "";
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];
    if (char !== "\\") {
      decoded += char;
      continue;
    }
    index += 1;
    if (index >= value.length) throw new Error("不正な JavaScript 文字列");
    const escapeCode = value[index];
    const simple: Record<string, string> = {
      "'": "'",
      '"': '"',
      "\\": "\\",
      "/": "/",
      b: "\b",
      f: "\f",
      n: "\n",
      r: "\r",
      t: "\t",
      v: "\v",
      "0": "\0",
    };
    if (simple[escapeCode] !== undefined) {
      decoded += simple[escapeCode];
      continue;
    }
    if (escapeCode === "u") {
      const hex = value.slice(index + 1, index + 5);
      if (!/^[0-9a-f]{4}$/i.test(hex)) throw new Error("不正な Unicode escape");
      decoded += String.fromCharCode(Number.parseInt(hex, 16));
      index += 4;
      continue;
    }
    if (escapeCode === "x") {
      const hex = value.slice(index + 1, index + 3);
      if (!/^[0-9a-f]{2}$/i.test(hex)) throw new Error("不正な hex escape");
      decoded += String.fromCharCode(Number.parseInt(hex, 16));
      index += 2;
      continue;
    }
    if (escapeCode === "\n") continue;
    decoded += escapeCode;
  }
  return decoded;
}

function leadingNumber(value: string | null | undefined): number | null {
  const match = value?.replaceAll(",", "").match(/^(\d+)/);
  return match ? Number(match[1]) : null;
}

function strikeKind(row: OfficialFrameRow): StrikeKind | null {
  const attribute = row.attribute?.replace(/[※\s]/g, "") ?? "";
  if (!attribute || attribute.includes("投") || attribute.includes("弾")) {
    return null;
  }
  if (row.type === "AIR") return "air";
  const first = [...attribute].find((value) => "上中下".includes(value));
  if (first === "上") return "high";
  if (first === "中") return "overhead";
  if (first === "下") return "low";
  return null;
}

interface ParsedButtons {
  start: number;
  variants: string[][];
}

const CLASSIC_BUTTON = /LP|MP|HP|LK|MK|HK|P|K/;

function classicButtons(command: string): ParsedButtons | null {
  const start = command.search(CLASSIC_BUTTON);
  if (start < 0) return null;
  const expression = command.slice(start).replaceAll("+", "");
  const variants: string[][] = [];
  for (const alternative of expression.split("or")) {
    if (alternative === "LP|MP|HP" || alternative === "P") {
      variants.push(["弱P"], ["中P"], ["強P"]);
      continue;
    }
    if (alternative === "LK|MK|HK" || alternative === "K") {
      variants.push(["弱K"], ["中K"], ["強K"]);
      continue;
    }
    if (alternative === "LPMPHP" || alternative === "PP") {
      variants.push(["弱P", "中P"], ["弱P", "強P"], ["中P", "強P"]);
      continue;
    }
    if (alternative === "LKMKHK" || alternative === "KK") {
      variants.push(["弱K", "中K"], ["弱K", "強K"], ["中K", "強K"]);
      continue;
    }
    for (const choice of alternative.split("|")) {
      const tokens = choice.match(/LP|MP|HP|LK|MK|HK/g) ?? [];
      if (
        tokens.join("") !== choice ||
        tokens.length === 0 ||
        tokens.length > 3
      ) {
        continue;
      }
      variants.push(tokens.map(classicButtonLabel));
    }
  }
  return { start, variants: deduplicateButtons(variants) };
}

function classicButtonLabel(button: string): string {
  const strength = { L: "弱", M: "中", H: "強" }[button[0]];
  return `${strength}${button[1]}`;
}

const MODERN_BUTTON = /攻撃三つ|攻撃二つ|攻撃|弱|中|強|SP/;

function modernButtons(command: string): ParsedButtons | null {
  if (
    command.includes("DI") ||
    command.includes("DP") ||
    command.includes("投")
  ) {
    return null;
  }
  const start = command.search(MODERN_BUTTON);
  if (start < 0) return null;
  const expression = command.slice(start).replaceAll("+", "");
  const variants: string[][] = [];
  for (const alternative of expression.split("or")) {
    if (alternative === "攻撃") {
      variants.push(["弱"], ["中"], ["強"]);
      continue;
    }
    if (alternative === "攻撃二つ") {
      variants.push(["弱", "中"], ["弱", "強"], ["中", "強"]);
      continue;
    }
    if (alternative === "攻撃三つ") {
      variants.push(["弱", "中", "強"]);
      continue;
    }
    for (const choice of alternative.split("|")) {
      const tokens = choice.match(/弱|中|強|SP/g) ?? [];
      if (
        tokens.join("") !== choice ||
        tokens.length === 0 ||
        tokens.length > 3
      ) {
        continue;
      }
      variants.push(tokens);
    }
  }
  return { start, variants: deduplicateButtons(variants) };
}

function directionsFor(prefix: string, airborne: boolean): InputDirection[] {
  if (airborne) return ["any"];
  const cleaned = prefix.replaceAll("AUTO", "").replaceAll("+", "");
  if (!cleaned) return ["standing"];
  if (cleaned.includes("回転")) return ["any"];

  const parts = cleaned.includes("or") ? cleaned.split("or") : [cleaned];
  const directions: InputDirection[] = [];
  for (const part of parts) {
    if (part.includes("N")) directions.push("neutral");
    const numbers = part.match(/\d+/g) ?? [];
    const finalDigit = numbers.at(-1)?.at(-1);
    const direction = directionForDigit(finalDigit);
    if (direction) directions.push(direction);
  }
  return [
    ...new Set(directions.length > 0 ? directions : (["standing"] as const)),
  ];
}

function directionForDigit(digit: string | undefined): InputDirection | null {
  if (digit === "5") return "neutral";
  if (digit === "2") return "down";
  if (digit === "1" || digit === "3") return "down_diagonal";
  if (digit === "4" || digit === "6") return "horizontal";
  if (digit === "7" || digit === "8" || digit === "9") return "any";
  return null;
}

function deduplicateButtons(values: string[][]): string[][] {
  const seen = new Set<string>();
  return values.filter((buttons) => {
    const key = [...buttons].sort().join("+");
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function deduplicatePatterns(
  values: AttackInputPattern[],
): AttackInputPattern[] {
  const seen = new Set<string>();
  return values.filter((pattern) => {
    const key = `${pattern.direction}:${pattern.auto}:${[...pattern.buttons].sort().join("+")}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
