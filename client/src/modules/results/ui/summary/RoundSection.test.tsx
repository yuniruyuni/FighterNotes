import { describe, expect, test } from "bun:test";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { RoundSection } from "./RoundSection.js";

describe("RoundSection", () => {
  test("native buttonのSpace操作と行のpointer操作で開始場面を開く", async () => {
    const user = userEvent.setup();
    const scenes: Array<Omit<SceneSelection, "key">> = [];
    render(
      <RoundSection
        report={syntheticAdviceReport({
          round_summaries: [
            {
              round_no: 1,
              start_frame: 120,
              end_frame: 1_200,
              won: true,
              own_hp_end: 0.4,
              opp_hp_end: 0,
              own_hp_lost: 0.6,
              opp_hp_lost: 1,
              own_hits_taken: 4,
              early_hit: false,
              own_burnouts: 0,
              detection_confidence: "high",
            },
          ],
        })}
        onSceneChange={(scene) => scenes.push(scene)}
      />,
    );

    const button = screen.getByRole("button", {
      name: "ラウンド 1 の開始場面を動画で開く",
    });
    const row = button.closest("tr");
    expect(row).not.toBeNull();
    expect(row).not.toHaveAttribute("tabindex");

    button.focus();
    await user.keyboard(" ");
    expect(scenes).toEqual([
      { frame: 120, card: null, label: "ラウンド 1 開始" },
    ]);

    fireEvent.click(row!);
    expect(scenes).toHaveLength(2);
  });
});
