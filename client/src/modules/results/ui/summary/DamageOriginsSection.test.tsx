import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type {
  AnalysisAvailability,
  AttributedDamageEvent,
  DamageApproach,
  DamageAttackEvidence,
  DamageBreakdown,
  DamageContact,
  DamageOrigin,
  RoundSummary,
  StrikeKind,
} from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { DamageOriginsSection } from "./DamageOriginsSection.js";

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

function damage(
  sequence: number,
  round: number,
  hpDrop: number,
  origin: DamageOrigin,
  strikeKind?: StrikeKind,
  approach?: DamageApproach,
  contact?: DamageContact,
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
    ...(approach ? { approach } : {}),
    ...(contact ? { contact, contact_confidence: "high" as const } : {}),
    contexts: origin === "throw" ? ["burnout"] : [],
  };
}

function attackEvidence(
  overrides: Partial<DamageAttackEvidence> = {},
): DamageAttackEvidence {
  return {
    victim: 1,
    attacker: 2,
    damage_start_frame: 90,
    sequence_start_frame: 100,
    sequence_end_frame: 140,
    combo_damage: 1_200,
    sequence_count: 1,
    final_scaling_percent: 100,
    starter_attribute: "throw",
    final_attribute: "throw",
    complete: true,
    recovered_from_max: false,
    confidence: "high",
    hp_consistency: "consistent",
    ...overrides,
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

  test("接近手段と接触種別を同時に表示する", () => {
    render(
      <DamageOriginsSection
        breakdown={{
          attribution_version: 5,
          total_hp_lost: 0.2,
          classified_hp_lost: 0.2,
          events: [
            damage(
              1,
              1,
              0.2,
              "raw_drive_rush",
              undefined,
              "raw_drive_rush",
              "throw",
            ),
          ],
        }}
        rounds={[round(1)]}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(screen.getByText("生ドライブラッシュ→投げ")).toBeInTheDocument();
  });

  test("中央攻撃情報の4状態と根拠を折りたたみ表示し、scene labelにも含める", async () => {
    const user = userEvent.setup();
    const evidenceEvents = [
      {
        ...damage(1, 1, 0.12, "throw"),
        attack_evidence: attackEvidence(),
      },
      {
        ...damage(2, 1, 0.2, "strike", "low"),
        attack_evidence: attackEvidence({
          damage_start_frame: 190,
          sequence_start_frame: 200,
          sequence_end_frame: 240,
          combo_damage: 1_800,
          sequence_count: 2,
          final_scaling_percent: 80,
          starter_attribute: "lower",
          final_attribute: "middle",
          hp_consistency: "mismatch",
        }),
      },
      {
        ...damage(3, 1, 0.1, "strike", "high"),
        attack_evidence: attackEvidence({
          damage_start_frame: 290,
          sequence_start_frame: 300,
          sequence_end_frame: 340,
          combo_damage: 0,
          hp_consistency: "unverified",
        }),
      },
      {
        ...damage(4, 1, 0.08, "unclassified"),
        attack_evidence: attackEvidence({
          damage_start_frame: 390,
          sequence_start_frame: 400,
          sequence_end_frame: 440,
          combo_damage: 800,
          complete: false,
          recovered_from_max: true,
          confidence: "medium",
        }),
      },
    ];
    const { container } = render(
      <DamageOriginsSection
        breakdown={{
          attribution_version: 5,
          total_hp_lost: 0.5,
          classified_hp_lost: 0.42,
          events: evidenceEvents,
        }}
        rounds={[round(1)]}
        frameTimestamps={Array.from({ length: 500 }, (_, frame) => frame / 60)}
        onSceneChange={() => undefined}
      />,
    );

    expect(
      container.querySelector(
        '.attack-evidence-status[data-status="consistent"]',
      ),
    ).toHaveTextContent("HPバーと整合");
    expect(
      container.querySelector(
        '.attack-evidence-status[data-status="mismatch"]',
      ),
    ).toHaveTextContent("HPバーと不一致");
    expect(
      container.querySelector(
        '.attack-evidence-status[data-status="unverified"]',
      ),
    ).toHaveTextContent("HP未照合");
    expect(
      container.querySelector(
        '.attack-evidence-status[data-status="incomplete"]',
      ),
    ).toHaveTextContent("表示認識が不完全");
    expect(screen.getByText("ゲーム内表示 0")).toBeInTheDocument();

    const consistentDetails = container.querySelector(
      'details.attack-evidence-details[data-status="consistent"]',
    );
    expect(consistentDetails).not.toHaveAttribute("open");
    await user.click(consistentDetails!.querySelector("summary")!);
    expect(consistentDetails).toHaveAttribute("open");
    expect(consistentDetails).toHaveTextContent("HPバーの標準10,000換算1,200");
    expect(consistentDetails).toHaveTextContent("表示damageとの差0");
    expect(consistentDetails).toHaveTextContent("始動 投げ → 最終 投げ");

    expect(
      screen.getByRole("button", {
        name: /ゲーム内表示 累積ダメージ 1,200、1 hit、最終補正 100%、始動 投げ、最終 投げ、HPバーと整合、認識確度 高/,
      }),
    ).toBeInTheDocument();
  });

  test("HP不一致warningから該当sceneへ移動し、根拠なしeventは従来表示を保つ", async () => {
    const user = userEvent.setup();
    const selected: Array<Omit<SceneSelection, "key">> = [];
    const mismatch = {
      ...damage(2, 1, 0.2, "strike", "low"),
      attack_evidence: attackEvidence({
        damage_start_frame: 190,
        sequence_start_frame: 200,
        sequence_end_frame: 240,
        combo_damage: 1_500,
        hp_consistency: "mismatch" as const,
      }),
    };
    const { container } = render(
      <DamageOriginsSection
        breakdown={{
          attribution_version: 5,
          total_hp_lost: 0.8,
          classified_hp_lost: 0.8,
          events: [damage(1, 1, 0.6, "throw"), mismatch],
        }}
        rounds={[round(1)]}
        frameTimestamps={Array.from({ length: 300 }, (_, frame) => frame / 60)}
        onSceneChange={(scene) => selected.push(scene)}
      />,
    );

    expect(
      screen.getByText(/HPバー推定が一致しない場面が1件/),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole("button", {
        name: "HP表示不一致・R1・20%。3.2秒を動画で確認",
      }),
    );
    expect(selected).toEqual([
      {
        frame: 190,
        endFrame: 240,
        card: null,
        label: "HP表示不一致・R1・20%",
      },
    ]);
    expect(
      container.querySelectorAll("details.attack-evidence-details"),
    ).toHaveLength(1);
    expect(
      screen.getByRole("button", {
        name: "投げ・R1・60%。判定確度 高。状況 バーンアウト中。動画で確認",
      }),
    ).toBeInTheDocument();
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

  test("自分HPが利用不能なら既存の値を0件や0%として表示しない", () => {
    render(
      <DamageOriginsSection
        breakdown={breakdown}
        coverage={{
          match_frames: 100,
          analyzed_match_frames: 100,
          input_segments: 1,
          analyzed_input_segments: 1,
          availability: availability({
            own_hp: "unavailable",
          }),
        }}
        rounds={[round(1), round(2)]}
        frameTimestamps={[]}
        onSceneChange={() => undefined}
      />,
    );

    expect(
      screen.getByText(/被ダメージ量・件数・起点は確認不能/),
    ).toBeInTheDocument();
    expect(screen.queryByText("150%")).not.toBeInTheDocument();
    expect(screen.queryByText("未分類（要確認）")).not.toBeInTheDocument();
  });
});
