import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join, sep } from "node:path";
import {
  CHARACTER_IDS,
  createPersistablePublishedAnalysis,
  createPublishedAnalysisContent,
  type DeletePasswordHash,
  FINDING_ASSESSMENTS,
  FINDING_KINDS,
  MAX_COUNT,
  MAX_DURATION_DECISECONDS,
  MAX_HP_BP,
  MAX_SEVERITY_BP,
  PublishedAnalysis,
  parseDeletePassword,
  parseShareId,
  serializedByteLength,
  TARGET_PUBLISHED_ANALYSIS_BYTES,
} from ".";

function candidate() {
  return {
    rulesetVersion: 3,
    ownCharacter: "LUKE",
    opponentCharacter: "CHUN_LI",
    rounds: { detected: 3, won: 2, lost: 1, unresolved: 0 },
    findings: [
      { kind: "big_hits", occurrences: 2, severityBp: 3100 },
      { kind: "anti_air", occurrences: 1, severityBp: 1200 },
    ],
    tactics: {
      antiAir: { opportunities: 3, successes: 2, jumpInsAllowed: 1 },
      driveImpact: {
        faced: 2,
        returned: 1,
        blocked: 0,
        parried: 0,
        hit: 1,
        avoided: 0,
        unconfirmed: 0,
      },
      rawDriveRush: { faced: 1, defended: 1, hit: 0, unconfirmed: 0 },
      dashThrow: { faced: 1 },
      throwWhiff: { count: 0 },
      fastestChallenge: {
        opportunities: 4,
        strikeAttempts: 2,
        strikeLosses: 1,
        throwAttempts: 1,
        throwLosses: 0,
      },
      burnout: {
        count: 1,
        durationDeciseconds: 123,
        hpLostBp: 2100,
        hpDealtBp: 800,
        selfInitiated: 1,
        forced: 0,
        mixed: 0,
        unknown: 0,
      },
    },
  };
}

function superArts(
  count: number,
  availability: "complete" | "partial" = "complete",
) {
  return {
    own: {
      availability,
      levels: { sa1: count, sa2: count, sa3: count, ca: count },
      outcomes: {
        hit: count,
        block: count,
        noImmediateContact: count,
        punished: count,
        ko: count,
      },
      contexts: {
        combo: count,
        punish: count,
        reversal: count,
        neutral: count,
      },
    },
    opponent: {
      availability,
      levels: { sa1: count, sa2: count, sa3: count, ca: count },
      outcomes: {
        hit: count,
        block: count,
        noImmediateContact: count,
        punished: count,
        ko: count,
      },
    },
  };
}

describe("PublishedAnalysis model", () => {
  test("Rustの全カードIDとrulesetを共有カタログが網羅する", () => {
    const adviceDir = join(
      import.meta.dir,
      "../../../../crates/video-analyzer/src/advice",
    );
    const detectorDir = join(adviceDir, "detectors");
    const detectors = [
      ...new Bun.Glob("**/*.rs").scanSync({
        cwd: detectorDir,
        absolute: true,
      }),
    ]
      .sort()
      .filter((path) => !path.split(sep).includes("tests"))
      .map((path) => readFileSync(path, "utf8"))
      .join("\n");
    const adviceParameters = readFileSync(
      join(adviceDir, "parameters.rs"),
      "utf8",
    );
    const rustFindingKinds = [...detectors.matchAll(/id: "([^"]+)"/g)]
      .map((match) => match[1])
      .sort();
    expect(rustFindingKinds).toEqual([...FINDING_KINDS].sort());
    expect(adviceParameters).toContain("pub const RULESET_VERSION: u32 = 9;");
  });

  test("clientとDB migrationの閉じたIDがサーバーカタログと一致する", () => {
    const clientShareSource = readFileSync(
      join(
        import.meta.dir,
        "../../../../client/src/modules/sharing/domain/published-analysis-contract.ts",
      ),
      "utf8",
    );
    const clientCharacterSource = readFileSync(
      join(
        import.meta.dir,
        "../../../../client/src/modules/analysis/domain/character.ts",
      ),
      "utf8",
    );
    expect(tsObjectIds(clientCharacterSource, "CHARACTER_CATALOG")).toEqual([
      ...CHARACTER_IDS,
    ]);
    expect(tsArray(clientShareSource, "SHAREABLE_FINDING_KINDS")).toEqual([
      ...FINDING_KINDS,
    ]);
    expect(tsArray(clientShareSource, "SHAREABLE_ASSESSMENTS")).toEqual([
      ...FINDING_ASSESSMENTS,
    ]);

    const migration = readFileSync(
      join(import.meta.dir, "../../../../schema/tables/published_analyses.sql"),
      "utf8",
    );
    expect(sqlEnum(migration, "own_character")).toEqual([...CHARACTER_IDS]);
    expect(sqlEnum(migration, "opponent_character")).toEqual([
      ...CHARACTER_IDS,
    ]);
    expect(sqlEnum(migration, "kind")).toEqual([...FINDING_KINDS]);
    expect(sqlEnum(migration, "assessment")).toEqual([...FINDING_ASSESSMENTS]);
    expect(migration).toContain(
      "minus_defense_opportunities INTEGER NOT NULL DEFAULT 0",
    );
  });

  test("strict inputを正規化しfindingを標準順へ並べる", () => {
    const result = createPublishedAnalysisContent(candidate());
    const repeated = createPublishedAnalysisContent(candidate());
    expect(result.ok).toBe(true);
    expect(repeated).toEqual(result);
    if (!result.ok) return;
    expect(result.value.schemaVersion).toBe(1);
    expect(result.value.presentationRevision).toBe(1);
    expect(result.value.findings.map((item) => item.kind)).toEqual([
      "anti_air",
      "big_hits",
    ]);
    expect(result.value.findings.map((item) => item.assessment)).toEqual([
      "diagnosis",
      "observation",
    ]);
    expect(Object.isFrozen(result.value)).toBe(true);
    expect(Object.isFrozen(result.value.tactics.burnout)).toBe(true);
  });

  test("v9のSA/CA availabilityと公開集計を閉じたモデルへ保存する", () => {
    const result = createPublishedAnalysisContent({
      ...candidate(),
      rulesetVersion: 9,
      findings: candidate().findings.map((finding) => ({
        ...finding,
        assessment: "diagnosis" as const,
      })),
      superArts: superArts(2),
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.superArts).toEqual(superArts(2));
    expect(Object.isFrozen(result.value.superArts?.own)).toBe(true);

    const partial = createPublishedAnalysisContent({
      ...candidate(),
      rulesetVersion: 9,
      findings: candidate().findings.map((finding) => ({
        ...finding,
        assessment: "diagnosis" as const,
      })),
      superArts: superArts(1, "partial"),
    });
    expect(partial.ok).toBe(true);
    if (partial.ok) {
      expect(partial.value.superArts?.own.availability).toBe("partial");
    }
  });

  test("全キャラクターと全findingを閉じたIDとして受理する", () => {
    for (const character of CHARACTER_IDS) {
      const value = candidate();
      value.ownCharacter = character;
      expect(createPublishedAnalysisContent(value).ok).toBe(true);
    }
    const value = candidate();
    value.findings = FINDING_KINDS.map((kind) => ({
      kind,
      occurrences: 1,
      severityBp: 1,
    }));
    expect(createPublishedAnalysisContent(value).ok).toBe(true);
  });

  test("未知キー、未知ID、重複、非整数、ラウンド不整合を拒否する", () => {
    const invalidValues: unknown[] = [
      { ...candidate(), comment: "<script>alert(1)</script>" },
      { ...candidate(), ownCharacter: "<img onerror=alert(1)>" },
      { ...candidate(), rulesetVersion: 999 },
      { ...candidate(), rulesetVersion: 6 },
      {
        ...candidate(),
        rounds: { detected: 3, won: 3, lost: 1, unresolved: 0 },
      },
      {
        ...candidate(),
        findings: [
          { kind: "anti_air", occurrences: 1, severityBp: 1 },
          { kind: "anti_air", occurrences: 2, severityBp: 2 },
        ],
      },
      {
        ...candidate(),
        tactics: {
          ...candidate().tactics,
          dashThrow: { faced: 1.5 },
        },
      },
      {
        ...candidate(),
        rounds: { detected: 256, won: 256, lost: 0, unresolved: 0 },
      },
      {
        ...candidate(),
        findings: [{ kind: "anti_air", occurrences: 0, severityBp: 1 }],
      },
      {
        ...candidate(),
        findings: [
          {
            kind: "anti_air",
            occurrences: 1,
            severityBp: Number.POSITIVE_INFINITY,
          },
        ],
      },
      {
        ...candidate(),
        tactics: {
          ...candidate().tactics,
          dashThrow: { faced: -1 },
        },
      },
    ];
    for (const value of invalidValues) {
      expect(createPublishedAnalysisContent(value).ok).toBe(false);
    }
  });

  test("不正入力を安定したfailure codeとfield pathへ変換する", () => {
    const result = createPublishedAnalysisContent({
      ...candidate(),
      rounds: { detected: 3, won: 3, lost: 1, unresolved: 0 },
      findings: [
        { kind: "anti_air", occurrences: 1, severityBp: 1 },
        { kind: "anti_air", occurrences: 2, severityBp: 2 },
      ],
    });

    expect(result).toMatchObject({
      ok: false,
      error: {
        code: "INVALID_INPUT",
        message: "Invalid published analysis",
        details: { paths: ["rounds", "findings.1.kind"] },
      },
    });
  });

  test("全findingをcatalog順へ並べる", () => {
    const value = candidate();
    value.findings = FINDING_KINDS.map((kind) => ({
      kind,
      occurrences: 1,
      severityBp: 1,
    }));

    const result = createPublishedAnalysisContent(value);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.findings.map(({ kind }) => kind)).toEqual([
      ...FINDING_KINDS,
    ]);
  });

  test("最大値を入れたモデルも4KiB目標内に収まる", () => {
    const value = candidate();
    value.rulesetVersion = 9;
    value.rounds = { detected: 255, won: 85, lost: 85, unresolved: 85 };
    value.findings = FINDING_KINDS.map((kind) => ({
      kind,
      occurrences: MAX_COUNT,
      severityBp: MAX_SEVERITY_BP,
    }));
    value.tactics = {
      antiAir: {
        opportunities: MAX_COUNT,
        successes: MAX_COUNT,
        jumpInsAllowed: MAX_COUNT,
      },
      driveImpact: {
        faced: MAX_COUNT,
        returned: MAX_COUNT,
        blocked: MAX_COUNT,
        parried: MAX_COUNT,
        hit: MAX_COUNT,
        avoided: MAX_COUNT,
        unconfirmed: MAX_COUNT,
      },
      rawDriveRush: {
        faced: MAX_COUNT,
        defended: MAX_COUNT,
        hit: MAX_COUNT,
        unconfirmed: MAX_COUNT,
      },
      dashThrow: { faced: MAX_COUNT },
      throwWhiff: { count: MAX_COUNT },
      fastestChallenge: {
        opportunities: MAX_COUNT,
        strikeAttempts: MAX_COUNT,
        strikeLosses: MAX_COUNT,
        throwAttempts: MAX_COUNT,
        throwLosses: MAX_COUNT,
      },
      burnout: {
        count: MAX_COUNT,
        durationDeciseconds: MAX_DURATION_DECISECONDS,
        hpLostBp: MAX_HP_BP,
        hpDealtBp: MAX_HP_BP,
        selfInitiated: MAX_COUNT,
        forced: MAX_COUNT,
        mixed: MAX_COUNT,
        unknown: MAX_COUNT,
      },
    };
    const result = createPublishedAnalysisContent({
      ...value,
      findings: value.findings.map((finding) => ({
        ...finding,
        assessment: "diagnosis" as const,
      })),
      superArts: superArts(MAX_COUNT),
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(serializedByteLength(result.value)).toBeLessThan(
      TARGET_PUBLISHED_ANALYSIS_BYTES,
    );
    const id = parseShareId("Abcdefghijklmnopqrstu_");
    if (!id) throw new Error("invalid fixture");
    const persisted = createPersistablePublishedAnalysis({
      id,
      content: result.value,
      deletePasswordHash: "x".repeat(512) as DeletePasswordHash,
      now: new Date("2026-07-13T00:00:00.000Z"),
      retentionDays: 365,
    });
    expect(persisted.analysis.logicalSizeBytes).toBeLessThanOrEqual(8 * 1024);
  });

  test("明示されたIDとハッシュから有効期限付きの保存モデルを作る", () => {
    const content = createPublishedAnalysisContent(candidate());
    expect(content.ok).toBe(true);
    if (!content.ok) return;
    const id = parseShareId("Abcdefghijklmnopqrstu_");
    if (!id) throw new Error("invalid fixture");
    const deletePasswordHash = "fixture-hash" as DeletePasswordHash;
    const created = createPersistablePublishedAnalysis({
      id,
      content: content.value,
      deletePasswordHash,
      now: new Date("2026-07-13T00:00:00.000Z"),
      retentionDays: 365,
    });
    expect(parseShareId(created.analysis.id)).toBe(created.analysis.id);
    expect(created.analysis.expiresAt.toISOString()).toBe(
      "2027-07-13T00:00:00.000Z",
    );
    expect(created.analysis.deletePasswordHash).toBe(deletePasswordHash);
    expect(created.analysis.logicalSizeBytes).toBeGreaterThan(0);
    expect(created.analysis.logicalSizeBytes).toBeLessThanOrEqual(8 * 1024);
    expect(parseDeletePassword("too-short")).toBeNull();
  });

  test("共有IDと削除passwordを境界値で検証する", () => {
    const id = "Abcdefghijklmnopqrstu_";
    expect(String(parseShareId(id))).toBe(id);
    expect(parseShareId(`${id}x`)).toBeNull();
    expect(parseShareId("../not-a-share-id")).toBeNull();
    expect(String(parseDeletePassword("x".repeat(12)))).toBe("x".repeat(12));
    expect(String(parseDeletePassword("x".repeat(128)))).toBe("x".repeat(128));
    expect(parseDeletePassword("x".repeat(11))).toBeNull();
    expect(parseDeletePassword("x".repeat(129))).toBeNull();
    expect(parseDeletePassword(" ".repeat(12))).toBeNull();
  });

  test("全query specificationとcursorを公開モデルから生成する", () => {
    const content = createPublishedAnalysisContent(candidate());
    if (!content.ok) throw new Error("invalid fixture");
    const id = parseShareId("Abcdefghijklmnopqrstu_");
    if (!id) throw new Error("invalid fixture");
    const createdAt = new Date("2026-07-22T00:00:00.000Z");
    const expiresAt = new Date("2026-08-22T00:00:00.000Z");
    const analysis: PublishedAnalysis = {
      id,
      content: content.value,
      createdAt,
      expiresAt,
    };

    expect(PublishedAnalysis.ById(id)).toMatchObject({ type: "ById", id });
    expect(PublishedAnalysis.ActiveAt(createdAt)).toMatchObject({
      type: "ActiveAt",
      at: createdAt,
    });
    expect(
      PublishedAnalysis.cursor(analysis, ["createdAt", "expiresAt", "id"]),
    ).toEqual({
      createdAt: "2026-07-22T00:00:00.000Z",
      expiresAt: "2026-08-22T00:00:00.000Z",
      id,
    });
    expect(PublishedAnalysis.defaultSort).toEqual({
      keys: ["createdAt", "id"],
      order: "desc",
    });
  });
});

function tsArray(source: string, name: string): string[] {
  const match = new RegExp(
    `export const ${name} = \\[([\\s\\S]*?)\\] as const;`,
  ).exec(source);
  if (!match) throw new Error(`${name} not found`);
  return quotedValues(match[1]);
}

function tsObjectIds(source: string, name: string): string[] {
  const match = new RegExp(
    `export const ${name} = \\[([\\s\\S]*?)\\] as const;`,
  ).exec(source);
  if (!match) throw new Error(`${name} not found`);
  return quotedValues(match[1], /\{ id: "([^"]+)"/g);
}

function sqlEnum(source: string, column: string): string[] {
  const match = new RegExp(
    `${column} TEXT(?: NOT NULL)?[\\s\\S]*?CHECK \\(${column} IN \\(([\\s\\S]*?)\\)\\)`,
  ).exec(source);
  if (!match) throw new Error(`${column} check not found`);
  return quotedValues(match[1], /'([^']+)'/g);
}

function quotedValues(
  source: string,
  pattern: RegExp = /"([^"]+)"/g,
): string[] {
  return [...source.matchAll(pattern)].map((match) => match[1]);
}
