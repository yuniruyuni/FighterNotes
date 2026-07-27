import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

type JsonObject = Record<string, unknown>;

interface DataStats {
  characterCount: number;
  moveCount: number;
}

interface FileManifest {
  sha256: string;
  characterCount: number;
  moveCount: number;
}

interface DataManifest {
  schemaVersion: number;
  dataVersion: string;
  files: Record<DataFileName, FileManifest>;
}

const DATA_DIRECTORY = fileURLToPath(
  new URL("../crates/video-analyzer/data/", import.meta.url),
);

const DATA_FILE_NAMES = ["frame_data.json", "attack_data.json"] as const;
type DataFileName = (typeof DATA_FILE_NAMES)[number];

const EXPECTED_CHARACTER_IDS = [
  "A_K_I",
  "AKUMA",
  "ALEX",
  "BLANKA",
  "CAMMY",
  "CHUN_LI",
  "C_VIPER",
  "DEE_JAY",
  "DHALSIM",
  "ED",
  "E_HONDA",
  "ELENA",
  "GUILE",
  "INGRID",
  "JAMIE",
  "JP",
  "JURI",
  "KEN",
  "KIMBERLY",
  "LILY",
  "LUKE",
  "MAI",
  "MANON",
  "MARISA",
  "M_BISON",
  "RASHID",
  "RYU",
  "SAGAT",
  "TERRY",
  "ZANGIEF",
] as const;

const MOVE_CATEGORIES = new Set([
  "normal",
  "unique",
  "special",
  "super",
  "throw",
  "common",
]);
const STRIKE_KINDS = new Set(["high", "overhead", "low", "air"]);
const INPUT_DIRECTIONS = new Set([
  "any",
  "standing",
  "neutral",
  "down",
  "horizontal",
  "down_diagonal",
]);
const CLASSIC_BUTTONS = new Set(["弱P", "中P", "強P", "弱K", "中K", "強K"]);
const MODERN_BUTTONS = new Set(["弱", "中", "強", "SP"]);

function fail(path: string, message: string): never {
  throw new Error(`${path}: ${message}`);
}

function expectObject(value: unknown, path: string): JsonObject {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    fail(path, "object ではありません");
  }
  return value as JsonObject;
}

function expectArray(value: unknown, path: string): unknown[] {
  if (!Array.isArray(value)) {
    fail(path, "array ではありません");
  }
  return value;
}

function expectString(value: unknown, path: string): string {
  if (typeof value !== "string") {
    fail(path, "string ではありません");
  }
  return value;
}

function expectBoolean(value: unknown, path: string): boolean {
  if (typeof value !== "boolean") {
    fail(path, "boolean ではありません");
  }
  return value;
}

function expectInteger(
  value: unknown,
  path: string,
  minimum: number,
  maximum: number,
): number {
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < minimum ||
    value > maximum
  ) {
    fail(path, `${minimum}..${maximum} の整数ではありません`);
  }
  return value;
}

function expectExactKeys(
  value: JsonObject,
  expected: readonly string[],
  path: string,
): void {
  const actual = Object.keys(value);
  if (
    actual.length !== expected.length ||
    actual.some((key, index) => key !== expected[index])
  ) {
    fail(
      path,
      `field が contract と一致しません (expected: ${expected.join(", ")}, actual: ${actual.join(", ")})`,
    );
  }
}

async function readJsonDocument(fileName: string): Promise<{
  bytes: Buffer;
  value: unknown;
}> {
  const filePath = join(DATA_DIRECTORY, fileName);
  const bytes = await readFile(filePath);
  const text = bytes.toString("utf8");
  if (!text.endsWith("\n") || text.includes("\r")) {
    fail(fileName, "LF終端のcanonical JSONではありません");
  }

  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch (error) {
    fail(fileName, `JSONをparseできません: ${String(error)}`);
  }

  const canonical = `${JSON.stringify(value, null, 1)}\n`;
  if (text !== canonical) {
    fail(fileName, "1-space indentのcanonical JSONではありません");
  }
  return { bytes, value };
}

function validateManifest(value: unknown): DataManifest {
  const manifest = expectObject(value, "manifest.json");
  expectExactKeys(
    manifest,
    ["schema_version", "data_version", "files"],
    "manifest.json",
  );
  const schemaVersion = expectInteger(
    manifest.schema_version,
    "manifest.json.schema_version",
    1,
    1,
  );
  const dataVersion = expectString(
    manifest.data_version,
    "manifest.json.data_version",
  );
  if (!/^\d{4}-\d{2}-\d{2}\.\d+$/.test(dataVersion)) {
    fail("manifest.json.data_version", "YYYY-MM-DD.N のversionではありません");
  }

  const files = expectObject(manifest.files, "manifest.json.files");
  expectExactKeys(files, DATA_FILE_NAMES, "manifest.json.files");

  const parsedFiles = Object.fromEntries(
    DATA_FILE_NAMES.map((fileName) => {
      const path = `manifest.json.files.${fileName}`;
      const file = expectObject(files[fileName], path);
      expectExactKeys(file, ["sha256", "character_count", "move_count"], path);
      const sha256 = expectString(file.sha256, `${path}.sha256`);
      if (!/^[a-f0-9]{64}$/.test(sha256)) {
        fail(`${path}.sha256`, "lowercase SHA-256ではありません");
      }
      return [
        fileName,
        {
          sha256,
          characterCount: expectInteger(
            file.character_count,
            `${path}.character_count`,
            1,
            1000,
          ),
          moveCount: expectInteger(
            file.move_count,
            `${path}.move_count`,
            1,
            1_000_000,
          ),
        },
      ];
    }),
  ) as Record<DataFileName, FileManifest>;

  return { schemaVersion, dataVersion, files: parsedFiles };
}

function validateCharacterKeys(value: JsonObject, path: string): void {
  expectExactKeys(value, EXPECTED_CHARACTER_IDS, path);
}

function hasControlCharacter(value: string): boolean {
  return [...value].some((character) => {
    const codePoint = character.codePointAt(0);
    return codePoint !== undefined && (codePoint <= 0x1f || codePoint === 0x7f);
  });
}

function validateMoveName(value: unknown, path: string): void {
  const name = expectString(value, path);
  if (
    name.length === 0 ||
    name.length > 128 ||
    name !== name.trim() ||
    hasControlCharacter(name)
  ) {
    fail(path, "空、前後空白、制御文字、または128文字超過を含みます");
  }
  if (/[<>]/.test(name) || /https?:\/\//i.test(name)) {
    fail(path, "HTMLまたはURLを含みます");
  }
}

function validateFrameData(value: unknown): DataStats {
  const table = expectObject(value, "frame_data.json");
  validateCharacterKeys(table, "frame_data.json");
  let moveCount = 0;

  for (const character of EXPECTED_CHARACTER_IDS) {
    const moves = expectArray(table[character], `frame_data.json.${character}`);
    if (moves.length === 0) {
      fail(`frame_data.json.${character}`, "技がありません");
    }
    moveCount += moves.length;

    moves.forEach((rawMove, index) => {
      const path = `frame_data.json.${character}[${index}]`;
      const move = expectObject(rawMove, path);
      expectExactKeys(move, ["name", "startup", "damage", "category"], path);
      validateMoveName(move.name, `${path}.name`);
      expectInteger(move.startup, `${path}.startup`, 1, 1000);
      expectInteger(move.damage, `${path}.damage`, 0, 100_000);
      const category = expectString(move.category, `${path}.category`);
      if (!MOVE_CATEGORIES.has(category)) {
        fail(`${path}.category`, `未知のcategoryです: ${category}`);
      }
    });
  }

  return {
    characterCount: EXPECTED_CHARACTER_IDS.length,
    moveCount,
  };
}

function validateInputPatterns(
  value: unknown,
  path: string,
  validButtons: ReadonlySet<string>,
): number {
  const patterns = expectArray(value, path);
  const signatures = new Set<string>();

  patterns.forEach((rawPattern, index) => {
    const patternPath = `${path}[${index}]`;
    const pattern = expectObject(rawPattern, patternPath);
    expectExactKeys(pattern, ["direction", "buttons", "auto"], patternPath);

    const direction = expectString(
      pattern.direction,
      `${patternPath}.direction`,
    );
    if (!INPUT_DIRECTIONS.has(direction)) {
      fail(`${patternPath}.direction`, `未知のdirectionです: ${direction}`);
    }

    const buttons = expectArray(pattern.buttons, `${patternPath}.buttons`).map(
      (button, buttonIndex) => {
        const parsed = expectString(
          button,
          `${patternPath}.buttons[${buttonIndex}]`,
        );
        if (!validButtons.has(parsed)) {
          fail(
            `${patternPath}.buttons[${buttonIndex}]`,
            `未知のbuttonです: ${parsed}`,
          );
        }
        return parsed;
      },
    );
    if (buttons.length === 0 || buttons.length > 3) {
      fail(`${patternPath}.buttons`, "1..3個のbuttonではありません");
    }
    if (new Set(buttons).size !== buttons.length) {
      fail(`${patternPath}.buttons`, "同じbuttonが重複しています");
    }

    const auto = expectBoolean(pattern.auto, `${patternPath}.auto`);
    const signature = `${direction}:${auto}:${buttons.join("+")}`;
    if (signatures.has(signature)) {
      fail(patternPath, "同じ入力patternが重複しています");
    }
    signatures.add(signature);
  });

  return patterns.length;
}

function validateAttackData(value: unknown): DataStats {
  const table = expectObject(value, "attack_data.json");
  validateCharacterKeys(table, "attack_data.json");
  let moveCount = 0;

  for (const character of EXPECTED_CHARACTER_IDS) {
    const moves = expectArray(
      table[character],
      `attack_data.json.${character}`,
    );
    if (moves.length === 0) {
      fail(`attack_data.json.${character}`, "攻撃候補がありません");
    }
    moveCount += moves.length;

    moves.forEach((rawMove, index) => {
      const path = `attack_data.json.${character}[${index}]`;
      const move = expectObject(rawMove, path);
      expectExactKeys(
        move,
        ["startup", "kind", "classic_inputs", "modern_inputs"],
        path,
      );
      expectInteger(move.startup, `${path}.startup`, 1, 1000);
      const kind = expectString(move.kind, `${path}.kind`);
      if (!STRIKE_KINDS.has(kind)) {
        fail(`${path}.kind`, `未知のstrike kindです: ${kind}`);
      }

      const classicCount = validateInputPatterns(
        move.classic_inputs,
        `${path}.classic_inputs`,
        CLASSIC_BUTTONS,
      );
      const modernCount = validateInputPatterns(
        move.modern_inputs,
        `${path}.modern_inputs`,
        MODERN_BUTTONS,
      );
      if (classicCount + modernCount === 0) {
        fail(path, "Classic/Modernの入力patternがありません");
      }
    });
  }

  return {
    characterCount: EXPECTED_CHARACTER_IDS.length,
    moveCount,
  };
}

function verifyManifestEntry(
  fileName: DataFileName,
  manifest: FileManifest,
  bytes: Buffer,
  stats: DataStats,
): void {
  const actualHash = createHash("sha256").update(bytes).digest("hex");
  if (actualHash !== manifest.sha256) {
    fail(
      `manifest.json.files.${fileName}.sha256`,
      `checksumが一致しません (expected: ${manifest.sha256}, actual: ${actualHash})`,
    );
  }
  if (
    stats.characterCount !== manifest.characterCount ||
    stats.moveCount !== manifest.moveCount
  ) {
    fail(
      `manifest.json.files.${fileName}`,
      `件数が一致しません (expected: ${manifest.characterCount} characters / ${manifest.moveCount} moves, actual: ${stats.characterCount} / ${stats.moveCount})`,
    );
  }
}

const [manifestDocument, frameDocument, attackDocument] = await Promise.all([
  readJsonDocument("manifest.json"),
  readJsonDocument("frame_data.json"),
  readJsonDocument("attack_data.json"),
]);

const manifest = validateManifest(manifestDocument.value);
const frameStats = validateFrameData(frameDocument.value);
const attackStats = validateAttackData(attackDocument.value);
verifyManifestEntry(
  "frame_data.json",
  manifest.files["frame_data.json"],
  frameDocument.bytes,
  frameStats,
);
verifyManifestEntry(
  "attack_data.json",
  manifest.files["attack_data.json"],
  attackDocument.bytes,
  attackStats,
);

console.log(
  `frame data ${manifest.dataVersion} validated: ` +
    `${frameStats.characterCount} characters, ` +
    `${frameStats.moveCount} frame moves, ` +
    `${attackStats.moveCount} attack moves`,
);
