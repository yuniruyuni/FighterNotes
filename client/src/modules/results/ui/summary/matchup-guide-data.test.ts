import { describe, expect, test } from "bun:test";
import { isCharacterId } from "~/modules/analysis/domain/character.js";
import { MATCHUP_GUIDES } from "./matchup-guide-data.js";

describe("matchup guide data", () => {
  test("キャラ id は実在し、項目 id は全体で一意", () => {
    const ids = new Set<string>();
    for (const [character, items] of Object.entries(MATCHUP_GUIDES)) {
      expect(isCharacterId(character)).toBe(true);
      expect(items.length).toBeGreaterThanOrEqual(2);
      for (const item of items) {
        expect(ids.has(item.id)).toBe(false);
        ids.add(item.id);
      }
    }
  });

  test("各項目は観測条件と本文を持つ", () => {
    for (const items of Object.values(MATCHUP_GUIDES)) {
      for (const item of items) {
        expect(item.triggers.length).toBeGreaterThanOrEqual(1);
        expect(item.text.length).toBeGreaterThanOrEqual(30);
        for (const trigger of item.triggers) {
          if (trigger.kind === "move") {
            expect(trigger.notations.length).toBeGreaterThanOrEqual(1);
          }
        }
      }
    }
  });
});

// 記譜のタイポは「絶対に発火しない項目」として静かに死ぬので、
// frame_data と突き合わせて実在を固定する。
import frameData from "../../../../../../crates/analysis-context/data/frame_data.json";

test("move 条件の記譜は frame_data に実在する", () => {
  const table = frameData as Record<string, Array<{ name: string }>>;
  for (const [character, items] of Object.entries(MATCHUP_GUIDES)) {
    const names = new Set((table[character] ?? []).map((move) => move.name));
    expect(names.size).toBeGreaterThan(0);
    for (const item of items) {
      for (const trigger of item.triggers) {
        if (trigger.kind !== "move") continue;
        for (const notation of trigger.notations) {
          expect(
            names.has(notation),
            `${character} ${item.id} の記譜 ${notation} が frame_data に無い`,
          ).toBe(true);
        }
      }
    }
  }
});
