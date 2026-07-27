import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type {
  AttributedDamageEvent,
  DamageBreakdown,
  DamageOrigin,
  RoundSummary,
  StrikeKind,
} from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { DamageOriginsSection } from "./DamageOriginsSection.js";

function damage(
  sequence: number,
  round: number,
  hpDrop: number,
  origin: DamageOrigin,
  strikeKind?: StrikeKind,
): AttributedDamageEvent {
  const start = sequence * 100;
  return {
    sequence_no: sequence,
    round_no: round,
    start_frame: start,
    end_frame: start + 40,
    scene_frame: start - 10,
    hp_before: 1,
    hp_after: 1 - hpDrop,
    hp_drop: hpDrop,
    origin,
    confidence: origin === "unclassified" ? "low" : "high",
    ...(strikeKind
      ? { strike_kind: strikeKind, strike_kind_confidence: "high" as const }
      : {}),
    contexts: origin === "throw" ? ["burnout"] : [],
  };
}

const events = [
  damage(1, 1, 0.6, "throw"),
  damage(2, 1, 0.4, "strike", "low"),
  damage(3, 2, 0.5, "unclassified"),
];

function round(roundNo: number): RoundSummary {
  return {
    round_no: roundNo,
    start_frame: (roundNo - 1) * 1000,
    end_frame: roundNo * 1000 - 1,
    won: null,
    own_hp_end: 0,
    opp_hp_end: 1,
    own_hp_lost: roundNo === 1 ? 1 : 0.5,
    opp_hp_lost: 0,
    own_hits_taken: roundNo === 1 ? 2 : 1,
    early_hit: false,
    own_burnouts: 0,
    detection_confidence: "medium",
  };
}

const breakdown: DamageBreakdown = {
  attribution_version: 2,
  total_hp_lost: 1.5,
  classified_hp_lost: 1,
  events,
};

describe("DamageOriginsSection", () => {
  test("100%超の全体値、R別値、未分類を表示する", async () => {
    const user = userEvent.setup();
    render(
      <DamageOriginsSection
        breakdown={breakdown}
        rounds={[round(1), round(2)]}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(screen.getByText("150%")).toBeInTheDocument();
    expect(screen.getByText("15,000")).toBeInTheDocument();
    expect(screen.getByText("66.7%")).toBeInTheDocument();
    expect(screen.getByText("未分類（要確認）")).toBeInTheDocument();
    expect(screen.getByText("下段")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: /全2R合計の被ダメージ構成/ }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "R1" }));
    expect(screen.getByText("最大体力比・R1").parentElement).toHaveTextContent(
      "100%",
    );
    expect(screen.getByText("10,000")).toBeInTheDocument();
    expect(screen.queryByText("未分類（要確認）")).not.toBeInTheDocument();
  });

  test("被弾シーンを既存プレイヤー用の区間として渡す", async () => {
    const user = userEvent.setup();
    const selected: Array<Omit<SceneSelection, "key">> = [];
    render(
      <DamageOriginsSection
        breakdown={breakdown}
        rounds={[round(1), round(2)]}
        frameTimestamps={Array.from({ length: 400 }, (_, frame) => frame / 60)}
        onSceneChange={(scene) => selected.push(scene)}
      />,
    );

    await user.click(
      screen.getByRole("button", {
        name: /投げ・R1・60%。判定確度 高。状況 バーンアウト中。動画で確認/,
      }),
    );
    expect(selected).toEqual([
      {
        frame: 90,
        endFrame: 140,
        card: null,
        label: "投げ・R1・60%",
      },
    ]);
  });

  test("旧レポートには空の分析セクションを追加しない", () => {
    const { container } = render(
      <DamageOriginsSection
        breakdown={undefined}
        rounds={[]}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(container).toBeEmptyDOMElement();
  });
});
