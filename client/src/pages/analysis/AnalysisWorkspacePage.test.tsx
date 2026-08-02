import { describe, expect, test } from "bun:test";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { DebugFrameInspector } from "~/modules/results/application/debug-frame-inspection.js";
import type { DebugFrameSource } from "~/modules/results/application/debug-frame-source.js";
import type { ResultsServices } from "~/modules/results/application/ports.js";
import { ResultsServicesProvider } from "~/modules/results/index.js";
import {
  syntheticAdviceReport,
  syntheticAnalysisResult,
} from "~/test-support/analysis.js";
import { AnalysisWorkspace } from "./AnalysisWorkspacePage.js";

describe("AnalysisWorkspace accessibility", () => {
  test("navigation・表示領域・scene遷移の現在位置とfocusを公開する", async () => {
    const user = userEvent.setup();
    const report = syntheticAdviceReport({
      rounds_detected: 1,
      cards: [
        {
          id: "anti_air",
          kind: "diagnosis",
          confidence: "high",
          title: "対空を見直す",
          severity: 2,
          description: "飛び込みを受けています。",
          practice: "対空を練習する。",
          evidence: [{ frame: 60, label: "対空確認" }],
        },
      ],
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
    });
    const result = {
      ...syntheticAnalysisResult(report),
      frameCount: 180,
      frameTimestamps: Array.from({ length: 180 }, (_, frame) => frame / 60),
      sampleData: null,
    };
    const restoreCanvas = installCanvasContext();

    try {
      render(
        <ResultsServicesProvider services={resultsServices()}>
          <AnalysisWorkspace
            file={new File(["video"], "replay.mp4", { type: "video/mp4" })}
            result={result}
            report={report}
            context={result.analysisContext}
            onBack={() => {}}
          />
        </ResultsServicesProvider>,
      );

      const navigation = screen.getByRole("navigation", { name: "解析結果" });
      const summaryButton = within(navigation).getByRole("button", {
        name: "解析サマリー",
      });
      const videoButton = within(navigation).getByRole("button", {
        name: "動画",
      });
      const cardButton = within(navigation).getByRole("button", {
        name: /対空を見直す/,
      });
      const debugButton = within(navigation).getByRole("button", {
        name: "認識デバッグ",
      });
      const summaryHeading = screen.getByRole("heading", {
        name: "解析結果サマリー",
      });
      await waitFor(() => expect(summaryHeading).toHaveFocus());
      expect(summaryButton).toHaveAttribute("aria-current", "page");
      expect(
        screen.getByRole("region", { name: "解析結果サマリー" }),
      ).not.toHaveAttribute("inert");
      expect(document.querySelector("#view-video")).toHaveAttribute("hidden");
      expect(document.querySelector("#view-video")).toHaveAttribute("inert");
      expect(document.querySelector("#view-debug")).toHaveAttribute("hidden");
      expect(
        screen.queryByRole("slider", { name: "動画の再生位置" }),
      ).toBeNull();

      summaryButton.focus();
      await user.keyboard("{Enter}");
      await waitFor(() => expect(summaryHeading).toHaveFocus());

      videoButton.focus();
      await user.keyboard("{Enter}");
      const progress = screen.getByRole("slider", { name: "動画の再生位置" });
      await waitFor(() => expect(progress).toHaveFocus());
      expect(videoButton).toHaveAttribute("aria-current", "page");
      expect(screen.getByRole("region", { name: "動画" })).not.toHaveAttribute(
        "hidden",
      );
      expect(document.querySelector("#view-summary")).toHaveAttribute("inert");
      expect(
        screen.queryByRole("region", { name: "解析結果サマリー" }),
      ).toBeNull();

      summaryButton.focus();
      await user.keyboard(" ");
      await waitFor(() => expect(summaryHeading).toHaveFocus());
      await user.click(
        screen.getByRole("button", { name: /対空確認 \(1\.0s\)/ }),
      );
      await waitFor(() => expect(progress).toHaveFocus());
      expect(cardButton).toHaveAttribute("aria-current", "page");
      expect(summaryHeading).not.toHaveFocus();

      summaryButton.focus();
      await user.keyboard("{Enter}");
      const roundButton = screen.getByRole("button", {
        name: "ラウンド 1 の開始場面を動画で開く",
      });
      roundButton.focus();
      await user.keyboard(" ");
      await waitFor(() => expect(progress).toHaveFocus());
      expect(videoButton).toHaveAttribute("aria-current", "page");

      debugButton.focus();
      await user.keyboard("{Enter}");
      const debugFirstControl = screen.getByRole("button", {
        name: "60フレーム戻る",
      });
      await waitFor(() => expect(debugFirstControl).toHaveFocus());
      expect(debugButton).toHaveAttribute("aria-current", "page");
      expect(
        screen.getByRole("region", { name: "認識デバッグ" }),
      ).not.toHaveAttribute("hidden");
      expect(document.querySelector("#view-video")).toHaveAttribute("inert");
      expect(
        screen.queryByRole("slider", { name: "動画の再生位置" }),
      ).toBeNull();
    } finally {
      restoreCanvas();
    }
  });
});

function resultsServices(): ResultsServices {
  const source: DebugFrameSource = {
    fallbackSource: document.createElement("canvas"),
    usesExactFrames: false,
    initialize: async () => {},
    decode: async () => null,
    seekFallback: () => {},
    destroy: () => {},
  };
  return {
    debugFrameInspector: debugInspector(),
    debugFrameSourceFactory: { create: () => source },
    history: { save: async () => {}, load: async () => [] },
  };
}

function debugInspector(): DebugFrameInspector {
  return {
    initialize: async () => ({
      p1: parallelogram(),
      p2: parallelogram(),
    }),
    inspectMeter: unexpectedInspection,
    inspectHp: unexpectedInspection,
    inspectDrive: unexpectedInspection,
    inspectSuper: unexpectedInspection,
    inspectInput: unexpectedInspection,
    inspectAttackInfo: unexpectedInspection,
  };
}

function parallelogram() {
  return {
    top_left: { x: 0, y: 0 },
    top_right: { x: 1, y: 0 },
    bottom_right: { x: 1, y: 1 },
    bottom_left: { x: 0, y: 1 },
  };
}

function unexpectedInspection(): never {
  throw new Error("inspection should not run in workspace interaction test");
}

function installCanvasContext(): () => void {
  const descriptor = Object.getOwnPropertyDescriptor(
    HTMLCanvasElement.prototype,
    "getContext",
  );
  Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
    configurable: true,
    value: () => ({}) as CanvasRenderingContext2D,
  });
  return () => {
    if (descriptor) {
      Object.defineProperty(
        HTMLCanvasElement.prototype,
        "getContext",
        descriptor,
      );
    } else {
      Reflect.deleteProperty(HTMLCanvasElement.prototype, "getContext");
    }
  };
}
