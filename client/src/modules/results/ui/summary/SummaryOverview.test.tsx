import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import { SummaryOverview } from "./SummaryOverview.js";

describe("SummaryOverview", () => {
  test("解析結果が映像からの推定であることを常に表示する", () => {
    render(
      <SummaryOverview
        context={{
          ownSide: "p2",
          p1: { character: "JURI" },
          p2: { character: "KEN" },
        }}
        report={syntheticAdviceReport({ summary: "解析結果の要約" })}
      />,
    );

    expect(screen.getByText("解析結果の要約")).toBeInTheDocument();
    expect(
      screen.getByText(
        "解析結果は映像からの推定です。正確な記録ではなく、見直しのための参考情報として利用してください。",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("2P（右）・KEN")).toBeInTheDocument();
    expect(screen.getByText(/相手: 1P（左）・JURI/)).toBeInTheDocument();
  });

  test("ラウンド割当率と検出器別の認識率を分けて表示する", () => {
    render(
      <SummaryOverview
        context={{ ownSide: "p1", p1: {}, p2: {} }}
        report={syntheticAdviceReport({
          coverage: {
            match_frames: 100,
            analyzed_match_frames: 80,
            input_segments: 1,
            analyzed_input_segments: 1,
            detector_match_frames: 80,
            own_hp_reliable_frames: 72,
            opponent_hp_reliable_frames: 64,
            own_drive_reliable_frames: 80,
            opponent_drive_reliable_frames: 40,
            own_super_reliable_frames: 0,
            opponent_super_reliable_frames: 0,
            own_input_observed_frames: 70,
            opponent_input_observed_frames: 60,
            own_meter_mapped_frames: 75,
            opponent_meter_mapped_frames: 70,
          },
        })}
      />,
    );

    expect(screen.getByText("ラウンド割当 80%")).toBeInTheDocument();
    expect(screen.getByText("HP認識 自分 90% / 相手 80%")).toBeInTheDocument();
    expect(screen.queryByText(/解析範囲/)).not.toBeInTheDocument();
  });
});
