import { describe, expect, test } from "bun:test";
import {
  buildAttackMoves,
  frameBundlePath,
  parseInputPatterns,
  parseOfficialFrameBundle,
} from "./frame-bundle-parser";

describe("frame bundle discovery", () => {
  test("HTML からハッシュ付きフレーム表 chunk を得る", () => {
    expect(
      frameBundlePath(
        '<script src="/6/_next/static/chunks/pages/character/%5Bname%5D/frame-deadbeef.js"></script>',
      ),
    ).toBe(
      "/6/_next/static/chunks/pages/character/%5Bname%5D/frame-deadbeef.js",
    );
  });

  test("JavaScript を実行せずキャラクター対応とJSON文字列を読む", () => {
    const row = (skill: string) =>
      JSON.stringify({
        frame: [
          {
            webId: "100",
            skill,
            type: "NORMAL",
            command: "2 + LK",
            command_modern: "2 + 弱",
            attribute: "下",
            startup_frame: "5",
          },
        ],
      }).replaceAll("'", "\\'");
    const source = `let a=JSON.parse('${row("A's low")}'),b=JSON.parse('${row("B")}');({ingrid:a,alex:b,cviper:a,sagat:a,elena:a,mai:a,terry:a,vega_mbison:a,gouki_akuma:a,ed:a})[name].frame`;

    const parsed = parseOfficialFrameBundle(source);
    expect(parsed.ingrid[0].skill).toBe("A's low");
    expect(parsed.alex[0].skill).toBe("B");
  });
});

describe("official input normalization", () => {
  test("Modern の AUTO 通常技と方向入力を構造化する", () => {
    expect(parseInputPatterns("AUTO + 弱", "modern")).toEqual([
      { direction: "standing", buttons: ["弱"], auto: true },
    ]);
    expect(parseInputPatterns("6 + 中", "modern")).toEqual([
      { direction: "horizontal", buttons: ["中"], auto: false },
    ]);
    expect(parseInputPatterns("2 + 弱", "modern")).toEqual([
      { direction: "down", buttons: ["弱"], auto: false },
    ]);
  });

  test("空中条件、ショートカットと手動入力の代替を扱う", () => {
    expect(
      parseInputPatterns("（ジャンプ中に）AUTO＋弱", "modern", true),
    ).toEqual([{ direction: "any", buttons: ["弱"], auto: true }]);
    expect(
      parseInputPatterns("6 + AUTO + SP/236 + 攻撃二つ", "modern"),
    ).toEqual([
      { direction: "horizontal", buttons: ["SP"], auto: true },
      { direction: "horizontal", buttons: ["弱", "中"], auto: false },
      { direction: "horizontal", buttons: ["弱", "強"], auto: false },
      { direction: "horizontal", buttons: ["中", "強"], auto: false },
    ]);
    expect(parseInputPatterns("2 + 弱 > 2 + 弱", "modern")).toEqual([]);
  });

  test("Classic の通常技とOD同時押しを入力履歴ラベルへ変換する", () => {
    expect(parseInputPatterns("2 + LK", "classic")).toEqual([
      { direction: "down", buttons: ["弱K"], auto: false },
    ]);
    expect(parseInputPatterns("236 + LPMPHP", "classic")).toEqual([
      { direction: "horizontal", buttons: ["弱P", "中P"], auto: false },
      { direction: "horizontal", buttons: ["弱P", "強P"], auto: false },
      { direction: "horizontal", buttons: ["中P", "強P"], auto: false },
    ]);
  });
});

describe("attack catalog", () => {
  test("上中下と空中を区別し、投げと弾を除外する", () => {
    const base = {
      webId: "100",
      type: "NORMAL",
      command: "LP",
      command_modern: "AUTO + 弱",
      startup_frame: "4",
    };
    const moves = buildAttackMoves([
      { ...base, skill: "High", attribute: "上" },
      { ...base, webId: "101", skill: "Overhead", attribute: "中" },
      { ...base, webId: "102", skill: "Low", attribute: "下下" },
      { ...base, webId: "103", skill: "Jump", type: "AIR", attribute: "中" },
      { ...base, webId: "104", skill: "Throw", attribute: "投" },
      { ...base, webId: "105", skill: "Shot", attribute: "上・弾" },
    ]);

    expect(moves.map((move) => move.kind)).toEqual([
      "high",
      "overhead",
      "low",
      "air",
    ]);
    expect(moves[0]).toEqual({
      startup: 4,
      kind: "high",
      classic_inputs: [
        { direction: "standing", buttons: ["弱P"], auto: false },
      ],
      modern_inputs: [{ direction: "standing", buttons: ["弱"], auto: true }],
    });
    expect(moves[0]).not.toHaveProperty("id");
    expect(moves[0]).not.toHaveProperty("name");
  });
});
