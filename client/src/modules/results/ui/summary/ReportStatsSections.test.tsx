import { describe, expect, test } from "bun:test";
import { render, screen, within } from "@testing-library/react";
import type {
  AnalysisAvailability,
  AnalysisCoverage,
} from "~/modules/analysis/contracts.js";
import { syntheticTacticStats } from "~/test-support/analysis.js";
import {
  InputStatsSection,
  TacticStatsSection,
} from "./ReportStatsSections.js";

const superStats = syntheticTacticStats({
  sa1_used: 0,
  sa2_used: 0,
  sa3_used: 0,
  ca_used: 0,
  opponent_sa1_used: 0,
  opponent_sa2_used: 0,
  opponent_sa3_used: 0,
  opponent_ca_used: 0,
  super_gauge_end: 0,
  opponent_super_gauge_end: 0,
});

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

function explicitCoverage(
  overrides: Partial<AnalysisAvailability> = {},
): AnalysisCoverage {
  return {
    match_frames: 100,
    analyzed_match_frames: 100,
    input_segments: 1,
    analyzed_input_segments: 1,
    detector_match_frames: 100,
    own_super_end_reliable: true,
    opponent_super_end_reliable: true,
    availability: availability(overrides),
  };
}

describe("TacticStatsSection detector coverage", () => {
  test("SAが全区間uncertainなら使用0回と表示しない", () => {
    render(
      <TacticStatsSection
        stats={superStats}
        coverage={{
          match_frames: 100,
          analyzed_match_frames: 100,
          input_segments: 1,
          analyzed_input_segments: 1,
          detector_match_frames: 100,
          own_hp_reliable_frames: 100,
          opponent_hp_reliable_frames: 100,
          own_drive_reliable_frames: 100,
          own_input_observed_frames: 100,
          opponent_input_observed_frames: 100,
          own_meter_mapped_frames: 100,
          opponent_meter_mapped_frames: 100,
          own_super_reliable_frames: 0,
          opponent_super_reliable_frames: 0,
          own_super_end_reliable: false,
          opponent_super_end_reliable: false,
        }}
      />,
    );

    const ownSuper = screen.getByText("自分のSA / CA").closest(".stat-item");
    expect(ownSuper).not.toBeNull();
    expect(
      within(ownSuper as HTMLElement).getByText("確認不能"),
    ).toBeInTheDocument();
    expect(
      within(ownSuper as HTMLElement).queryByText("0 回"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText("自分 確認不能 / 相手 確認不能"),
    ).toBeInTheDocument();
  });

  test("legacy reportはcoverageなしでも従来の集計を表示する", () => {
    render(<TacticStatsSection stats={superStats} />);

    const ownSuper = screen.getByText("自分のSA / CA").closest(".stat-item");
    expect(ownSuper).not.toBeNull();
    expect(
      within(ownSuper as HTMLElement).getByText("0 回"),
    ).toBeInTheDocument();
  });

  test("HP認識率が不足した戦術行は0件と断定しない", () => {
    render(
      <TacticStatsSection
        stats={syntheticTacticStats({
          anti_air_opportunities: 0,
          anti_air_successes: 0,
        })}
        coverage={{
          match_frames: 100,
          analyzed_match_frames: 100,
          input_segments: 1,
          analyzed_input_segments: 1,
          detector_match_frames: 100,
          own_hp_reliable_frames: 0,
          opponent_hp_reliable_frames: 100,
          own_input_observed_frames: 100,
          opponent_input_observed_frames: 100,
        }}
      />,
    );

    const antiAir = screen.getByText("対空 成功 / 機会").closest(".stat-item");
    expect(antiAir).not.toBeNull();
    expect(
      within(antiAir as HTMLElement).getByText("確認不能"),
    ).toBeInTheDocument();
    expect(
      within(antiAir as HTMLElement).queryByText("0 / 0"),
    ).not.toBeInTheDocument();
  });

  test("中央攻撃表示の帰属不足ではSA表示ダメージを確認不能にする", () => {
    render(
      <TacticStatsSection
        stats={syntheticTacticStats({
          sa1_used: 1,
          sa2_used: 0,
          sa3_used: 0,
          ca_used: 0,
          super_damage_samples: 0,
        })}
        coverage={{
          match_frames: 100,
          analyzed_match_frames: 100,
          input_segments: 1,
          analyzed_input_segments: 1,
          detector_match_frames: 100,
          own_hp_reliable_frames: 100,
          opponent_hp_reliable_frames: 100,
          own_super_reliable_frames: 100,
          opponent_super_reliable_frames: 100,
          attack_damage_events: 3,
          attack_damage_linked: 0,
        }}
      />,
    );

    const damage = screen
      .getByText("SA投入後の表示ダメージ")
      .closest(".stat-item");
    expect(damage).not.toBeNull();
    expect(
      within(damage as HTMLElement).getByText("確認不能"),
    ).toBeInTheDocument();
  });

  test("新レポートは分母0を利用可能扱いせず、旧レポートだけ互換表示する", () => {
    render(
      <TacticStatsSection
        stats={superStats}
        coverage={{
          match_frames: 0,
          analyzed_match_frames: 0,
          input_segments: 0,
          analyzed_input_segments: 0,
          detector_match_frames: 0,
          availability: availability({
            own_hp: "unavailable",
            opponent_hp: "unavailable",
            own_drive: "unavailable",
            opponent_drive: "unavailable",
            own_super: "unavailable",
            opponent_super: "unavailable",
            own_input: "unavailable",
            opponent_input: "unavailable",
            own_meter: "unavailable",
            opponent_meter: "unavailable",
            contacts: "unavailable",
            punishes: "unavailable",
            spatial: "not_applicable",
            own_attack_info: "not_applicable",
            opponent_attack_info: "not_applicable",
          }),
        }}
      />,
    );

    const antiAir = screen.getByText("対空 成功 / 機会").closest(".stat-item");
    const ownSuper = screen.getByText("自分のSA / CA").closest(".stat-item");
    expect(
      within(antiAir as HTMLElement).getByText("確認不能"),
    ).toBeInTheDocument();
    expect(
      within(ownSuper as HTMLElement).getByText("確認不能"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("自分 確認不能 / 相手 確認不能"),
    ).toBeInTheDocument();
  });

  test("空間解析が機会なしなら生ラッシュと前ステップ投げを欠測扱いしない", () => {
    render(
      <TacticStatsSection
        stats={syntheticTacticStats()}
        coverage={explicitCoverage({ spatial: "not_applicable" })}
      />,
    );

    const driveRush = screen
      .getByText("生ラッシュ対処 / 相手の生ラッシュ")
      .closest(".stat-item");
    const dashThrow = screen
      .getByText("前ステップ投げを受けた")
      .closest(".stat-item");
    expect(
      within(driveRush as HTMLElement).getByText("確認なし"),
    ).toBeInTheDocument();
    expect(
      within(driveRush as HTMLElement).queryByText("確認不能"),
    ).not.toBeInTheDocument();
    expect(
      within(dashThrow as HTMLElement).getByText("0 回"),
    ).toBeInTheDocument();
  });

  test("DI・生ラッシュ・SA文脈の固有依存関係を可用性契約から使う", () => {
    render(
      <TacticStatsSection
        stats={syntheticTacticStats({
          di_faced: 1,
          raw_drive_rushes_faced: 1,
          sa1_used: 1,
          sa2_used: 0,
          sa3_used: 0,
          ca_used: 0,
        })}
        coverage={explicitCoverage({
          own_input: "unavailable",
          opponent_drive: "unavailable",
          punishes: "unavailable",
        })}
      />,
    );

    for (const label of [
      "DI返し / 相手DI",
      "生ラッシュ対処 / 相手の生ラッシュ",
      "SAを使った文脈",
    ]) {
      const item = screen.getByText(label).closest(".stat-item");
      expect(item).not.toBeNull();
      expect(
        within(item as HTMLElement).getByText("少なくとも 1 件"),
      ).toBeInTheDocument();
    }
    const ownSuper = screen.getByText("自分のSA / CA").closest(".stat-item");
    expect(
      within(ownSuper as HTMLElement).getByText("1 回"),
    ).toBeInTheDocument();
  });
});

describe("InputStatsSection detector coverage", () => {
  test("入力自体は読めてもHP欠測ならジャンプ結果を0回と断定しない", () => {
    render(
      <InputStatsSection
        stats={{
          total_inputs: 10,
          minutes: 1,
          jumps: 2,
          jumps_per_min: 2,
          jump_got_hit: 0,
          jump_landed: 0,
          throw_attempts: 1,
          throw_hits: 0,
          button_presses: 5,
          auto_presses: 0,
          auto_ratio: 0,
          di_presses: 0,
          crouch_ratio: 0.2,
        }}
        coverage={{
          match_frames: 100,
          analyzed_match_frames: 100,
          input_segments: 10,
          analyzed_input_segments: 10,
          detector_match_frames: 100,
          own_input_observed_frames: 100,
          own_hp_reliable_frames: 0,
          opponent_hp_reliable_frames: 0,
          own_meter_mapped_frames: 100,
          opponent_meter_mapped_frames: 100,
        }}
      />,
    );

    for (const label of [
      "ジャンプを落とされた",
      "飛びを通した",
      "投げ 成功/試行",
    ]) {
      const item = screen.getByText(label).closest(".stat-item");
      expect(item).not.toBeNull();
      expect(
        within(item as HTMLElement).getByText("確認不能"),
      ).toBeInTheDocument();
    }
  });
});

describe("TacticStatsSection SA/CA availability", () => {
  test("ruleset v9の集計不能フラグを使用0回として表示しない", () => {
    render(
      <TacticStatsSection
        stats={syntheticTacticStats({
          super_art_stats_available: false,
          opponent_super_art_stats_available: false,
          sa1_used: 0,
          opponent_sa1_used: 0,
        })}
      />,
    );

    for (const label of ["自分のSA / CA", "相手のSA / CA"]) {
      const item = screen.getByText(label).closest(".stat-item");
      expect(item).not.toBeNull();
      expect(
        within(item as HTMLElement).getByText("確認不能"),
      ).toBeInTheDocument();
      expect(
        within(item as HTMLElement).queryByText("0 回"),
      ).not.toBeInTheDocument();
    }
    expect(screen.queryByText(/SA1 0 \/ SA2/)).not.toBeInTheDocument();
    expect(
      screen.getByText(/自分 確認不能 \/ 相手 確認不能/),
    ).toBeInTheDocument();
  });
});
