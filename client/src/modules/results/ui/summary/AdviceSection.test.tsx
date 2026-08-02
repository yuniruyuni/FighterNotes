import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import { AdviceSection } from "./AdviceSection.js";

describe("AdviceSection", () => {
  test("coverage不足で候補を抑制した場合は改善点なしと表示しない", () => {
    render(
      <AdviceSection
        report={syntheticAdviceReport({
          cards: [],
          suppressed_cards: [
            {
              id: "press_while_minus",
              title: "不利状況での最速暴れ",
              missing_requirements: ["own_input", "frame_meter"],
            },
          ],
        })}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(
      screen.queryByText("顕著な改善ポイントは検出されませんでした。"),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent(
      "1件の指摘候補を確認不能",
    );
    expect(screen.getByRole("note")).toHaveTextContent(
      "不利状況での最速暴れ: 自分の入力履歴・フレームメーター",
    );
  });

  test("候補自体を検出できないcoverage不足でも改善点なしと表示しない", () => {
    render(
      <AdviceSection
        report={syntheticAdviceReport({
          cards: [],
          suppressed_cards: [],
          coverage: {
            match_frames: 100,
            analyzed_match_frames: 100,
            input_segments: 0,
            analyzed_input_segments: 0,
            detector_match_frames: 100,
            availability: {
              own_hp: "unavailable",
              opponent_hp: "unavailable",
              own_drive: "available",
              opponent_drive: "available",
              own_super: "available",
              opponent_super: "available",
              own_input: "unavailable",
              opponent_input: "unavailable",
              own_meter: "available",
              opponent_meter: "available",
              contacts: "unavailable",
              punishes: "unavailable",
              spatial: "not_applicable",
              own_attack_info: "not_applicable",
              opponent_attack_info: "not_applicable",
            },
          },
        })}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(
      screen.queryByText("顕著な改善ポイントは検出されませんでした。"),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent(
      "改善ポイントを十分に判定できませんでした",
    );
    expect(screen.getByRole("note")).toHaveTextContent(
      "改善点がないという意味ではありません",
    );
  });
});
