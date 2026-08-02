import { describe, expect, test } from "bun:test";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { AnalysisAvailability } from "~/modules/analysis/contracts.js";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { RoundSection } from "./RoundSection.js";

function availability(
  overrides: Partial<AnalysisAvailability> = {},
): AnalysisAvailability {
  return {
    own_hp: "available",
    opponent_hp: "available",
    own_drive: "available",
    opponent_drive: "available",
    own_super: "available",
    opponent_super: "available",
    own_input: "available",
    opponent_input: "available",
    own_meter: "available",
    opponent_meter: "available",
    contacts: "available",
    punishes: "available",
    spatial: "available",
    own_attack_info: "available",
    opponent_attack_info: "available",
    ...overrides,
  };
}

const round = {
  round_no: 1,
  start_frame: 100,
  end_frame: 1000,
  won: true,
  own_hp_end: 0,
  opp_hp_end: 0.42,
  own_hp_lost: 1,
  opp_hp_lost: 0.58,
  own_hits_taken: 5,
  early_hit: false,
  own_burnouts: 2,
  detection_confidence: "high" as const,
};

describe("RoundSection", () => {
  test("native buttonのSpace操作と行のpointer操作で開始場面を開く", async () => {
    const user = userEvent.setup();
    const scenes: Array<Omit<SceneSelection, "key">> = [];
    render(
      <RoundSection
        report={syntheticAdviceReport({
          round_summaries: [
            {
              ...round,
              start_frame: 120,
              end_frame: 1_200,
              own_hp_end: 0.4,
              opp_hp_end: 0,
              own_hp_lost: 0.6,
              opp_hp_lost: 1,
              own_hits_taken: 4,
              own_burnouts: 0,
            },
          ],
        })}
        onSceneChange={(scene) => scenes.push(scene)}
      />,
    );

    const button = screen.getByRole("button", {
      name: "ラウンド 1 の開始場面を動画で開く",
    });
    const rowElement = button.closest("tr");
    expect(rowElement).not.toBeNull();
    expect(rowElement).not.toHaveAttribute("tabindex");

    button.focus();
    await user.keyboard(" ");
    expect(scenes).toEqual([
      { frame: 120, card: null, label: "ラウンド 1 開始" },
    ]);

    fireEvent.click(rowElement!);
    expect(scenes).toHaveLength(2);
  });

  test("HP欠測は勝敗とHP由来値だけを隠し、Drive由来の値を残す", () => {
    render(
      <RoundSection
        report={syntheticAdviceReport({
          round_summaries: [round],
          coverage: {
            match_frames: 100,
            analyzed_match_frames: 100,
            input_segments: 1,
            analyzed_input_segments: 1,
            availability: availability({
              own_hp: "unavailable",
            }),
          },
        })}
        onSceneChange={() => undefined}
      />,
    );

    const rowElement = screen.getByRole("row", { name: /1/ });
    expect(within(rowElement).getAllByText("確認不能")).toHaveLength(5);
    expect(within(rowElement).getByText("42%")).toBeInTheDocument();
    expect(within(rowElement).getByText("🔥🔥")).toBeInTheDocument();
    expect(within(rowElement).queryByText("WIN")).not.toBeInTheDocument();
    expect(within(rowElement).queryByText("0%")).not.toBeInTheDocument();
  });

  test("Drive欠測だけなら勝敗とHP値を残しバーンアウトを確認不能にする", () => {
    render(
      <RoundSection
        report={syntheticAdviceReport({
          round_summaries: [round],
          coverage: {
            match_frames: 100,
            analyzed_match_frames: 100,
            input_segments: 1,
            analyzed_input_segments: 1,
            availability: availability({
              own_drive: "unavailable",
            }),
          },
        })}
        onSceneChange={() => undefined}
      />,
    );

    const rowElement = screen.getByRole("row", { name: /1/ });
    expect(within(rowElement).getByText("WIN")).toBeInTheDocument();
    expect(within(rowElement).getByText("0%")).toBeInTheDocument();
    expect(within(rowElement).getByText("42%")).toBeInTheDocument();
    expect(within(rowElement).getByText("確認不能")).toBeInTheDocument();
    expect(within(rowElement).queryByText("🔥🔥")).not.toBeInTheDocument();
  });
});
