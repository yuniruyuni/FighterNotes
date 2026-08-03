import { describe, expect, test } from "bun:test";
import {
  frameCharacterSlugs,
  moveName,
  parseCharacterPage,
} from "./frame-page-parser";

const classic = (inner: string) =>
  `<span class="frame_arts__x">Dummy</span><p class="frame_classic__x">${inner}</p>`;
const img = (token: string) =>
  `<img src="/6/assets/images/common/controller/${token}.png" alt="" />`;
const sectionRow = (heading: string) => `<tr><th>${heading}</th></tr>`;
const moveRow = (name: string, startup: string, damage: string) =>
  `<tr><td>${name}</td><td>${startup}</td>${"<td>-</td>".repeat(5)}<td>${damage}</td></tr>`;

describe("frameCharacterSlugs", () => {
  test("locale付きlinkを重複排除して公開済みcharacterを列挙する", () => {
    const html = [
      '<a href="/6/character/ryu/frame">Ryu</a>',
      '<a href="/6/ja-jp/character/yasmine/frame">Yasmine</a>',
      '<a href="/6/en-asia/character/yasmine/frame">Yasmine</a>',
      '<a href="/6/character/ryu">profile only</a>',
    ].join("");

    expect(frameCharacterSlugs(html)).toEqual(["ryu", "yasmine"]);
  });
});

describe("moveName", () => {
  test("立ち通常技はボタンのみ", () => {
    expect(moveName(classic(`${img("icon_punch_l")}L`))).toBe("LP");
  });

  test("しゃがみ技は方向 + ボタン", () => {
    expect(
      moveName(
        classic(`${img("key-d")} ${img("key-plus")} ${img("icon_kick_m")}M`),
      ),
    ).toBe("2MK");
  });

  test("波動コマンドは 236 + ボタン", () => {
    expect(
      moveName(
        classic(
          `${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-plus")}${img("icon_punch_l")}L`,
        ),
      ),
    ).toBe("236LP");
  });

  test("SA は 236236 + 全ボタン表記", () => {
    expect(
      moveName(
        classic(
          `${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-plus")}${img("icon_kick")}`,
        ),
      ),
    ).toBe("236236K");
  });

  test("タメコマンドは [4]6 形式", () => {
    expect(
      moveName(
        classic(
          `${img("key-lc")}${img("key-r")}${img("key-plus")}${img("icon_punch_l")}L`,
        ),
      ),
    ).toBe("[4]6LP");
  });

  test("回転コマンドは 360 / 720 形式", () => {
    expect(
      moveName(
        classic(
          `${img("key-circle")}${img("key-plus")}${img("icon_punch_l")}L`,
        ),
      ),
    ).toBe("360LP");
    expect(
      moveName(
        classic(
          `${img("key-circle")}${img("key-circle")}${img("key-plus")}${img("icon_punch")}`,
        ),
      ),
    ).toBe("720P");
  });

  test("空中技は null（地上技のみ収録）", () => {
    expect(
      moveName(classic(`(During a jump) ${img("icon_punch_l")}L`)),
    ).toBeNull();
  });

  test("派生技・状態限定技は null（中立から出せない）", () => {
    // OD 技後の追撃（LUKE の DDT 等）
    expect(
      moveName(
        classic(
          `(After an OD Flash Knuckle)${img("icon_punch")}${img("icon_punch")}`,
        ),
      ),
    ).toBeNull();
    // スタンス派生（CAMMY のフーリガン派生等）
    expect(
      moveName(classic(`(During Hooligan Combination)${img("icon_kick")}`)),
    ).toBeNull();
    // ドライブリバーサル（ガード中限定）
    expect(
      moveName(
        classic(
          `(When blocking or during a successful Drive Parry)${img("icon_punch_h")}H${img("icon_kick_h")}H`,
        ),
      ),
    ).toBeNull();
  });

  test("投げの近距離条件やリソース条件は収録対象", () => {
    // 投げ（密着条件は中立から歩いて満たせる）
    expect(
      moveName(
        classic(
          `(When near opponent)${img("icon_punch_l")}L${img("icon_kick_l")}L`,
        ),
      ),
    ).toBe("LPLK");
    // JAMIE の飲酒レベル条件（リソース依存だが中立から出せる）
    expect(
      moveName(
        classic(
          `(Drink level 1 or higher)${img("key-d")}${img("key-dl")}${img("key-l")}${img("key-plus")}${img("icon_kick_m")}M`,
        ),
      ),
    ).toBe("214MK");
  });

  test("直接入力tokenがない自動発動・当身成立後の技は除外する", () => {
    expect(
      moveName(
        `<span class="frame_arts__x">Arc Step</span><p class="frame_classic__x">(Automatically activates after getting close with Sprint)</p>`,
      ),
    ).toBeNull();
    expect(
      moveName(
        `<span class="frame_arts__x">Scutum (Physical counter version)</span><p class="frame_classic__x">&#42;Take an attack during Scutum</p>`,
      ),
    ).toBeNull();
  });

  test("コマンドがdashの多段技後半は確反候補から除外する", () => {
    expect(
      moveName(
        `<span class="frame_arts__x">L Alon(2)</span><p class="frame_classic__x">-</p>`,
      ),
    ).toBeNull();
    expect(
      moveName(
        `<span class="frame_arts__x">Follow-up</span><p class="frame_classic__x">ー</p>`,
      ),
    ).toBeNull();
  });

  test("Classicコマンド欄が消えた構造変更は例外にする", () => {
    expect(() =>
      moveName(`<span class="frame_arts__x">Changed Markup</span>`),
    ).toThrow("Classicコマンドが見つかりません");
  });

  test("未知のコマンド画像は例外（変換漏れ検知）", () => {
    expect(() => moveName(classic(img("key-unknown")))).toThrow();
  });
});

describe("parseCharacterPage", () => {
  test("合成ページからカテゴリ、数値、地上技を抽出する", () => {
    const named = (name: string, command: string) =>
      `<span class="frame_arts__x">${name}</span><p class="frame_classic__x">${command}</p>`;
    const html = [
      "<table><tbody>",
      sectionRow("Normal Moves"),
      moveRow("Move", "Startup", "Damage"),
      moveRow(classic(`${img("icon_punch_l")}L`), "4", "300"),
      moveRow(
        classic(`${img("key-d")}${img("key-plus")}${img("icon_kick_m")}M`),
        "8",
        "500",
      ),
      moveRow(classic(`(During a jump) ${img("icon_punch_l")}L`), "5", "300"),
      sectionRow("Unique Attacks"),
      moveRow(
        classic(`${img("key-r")}${img("key-plus")}${img("icon_punch_m")}M`),
        "7",
        "600",
      ),
      sectionRow("Special Moves"),
      moveRow(
        classic(
          `${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-plus")}${img("icon_punch_l")}L`,
        ),
        "17",
        "1,000 (500x2)",
      ),
      sectionRow("Super Arts"),
      moveRow(
        classic(
          `${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-d")}${img("key-dr")}${img("key-r")}${img("key-plus")}${img("icon_kick")}`,
        ),
        "9",
        "2,000",
      ),
      sectionRow("Throws"),
      moveRow(
        classic(`${img("icon_punch_l")}L${img("icon_kick_l")}L`),
        "5",
        "1,200",
      ),
      sectionRow("Common Moves"),
      moveRow(named("Automatic Follow-up", ""), "26", "800"),
      "</tbody></table>",
    ].join("");

    expect(parseCharacterPage(html)).toEqual([
      { name: "LP", startup: 4, damage: 300, category: "normal" },
      { name: "2MK", startup: 8, damage: 500, category: "normal" },
      { name: "6MP", startup: 7, damage: 600, category: "unique" },
      { name: "236LP", startup: 17, damage: 1000, category: "special" },
      { name: "236236K", startup: 9, damage: 2000, category: "super" },
      { name: "LPLK", startup: 5, damage: 1200, category: "throw" },
    ]);
  });

  test("セクション見出しより先にデータ行があれば失敗する", () => {
    const html = moveRow(classic(`${img("icon_punch_l")}L`), "4", "300");
    expect(() => parseCharacterPage(html)).toThrow(
      "セクション見出しが見つからないままデータ行が出現: LP",
    );
  });
});
