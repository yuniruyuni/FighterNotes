import { describe, expect, test } from "bun:test";
import { isCharacterId } from "~/modules/analysis/domain/character.js";
import { MATCHUP_GUIDES } from "./matchup-guide-data.js";

describe("matchup guide data", () => {
  test("キャラ id は実在し、項目 id は全体で一意", () => {
    const ids = new Set<string>();
    for (const [character, items] of Object.entries(MATCHUP_GUIDES)) {
      expect(isCharacterId(character)).toBe(true);
      expect(items.length).toBeGreaterThanOrEqual(3);
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
