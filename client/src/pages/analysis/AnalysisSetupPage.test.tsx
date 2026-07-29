import { describe, expect, mock, test } from "bun:test";
import { fireEvent, render, waitFor } from "@testing-library/react";
import type { AnalysisServices } from "~/modules/analysis/application/ports.js";
import { AnalysisSessionProvider } from "~/modules/analysis/index.js";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import { AnalysisSetupPage } from "./AnalysisSetupPage.js";

describe("AnalysisSetupPage", () => {
  test("共有サービスなしで動画解析だけを完了できる", async () => {
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] = async (
      _file,
      _side,
      onProgress,
      context,
    ) => {
      onProgress(1, "解析完了");
      return {
        ...syntheticAnalysisResult(),
        analysisContext: context,
      };
    };
    const analyze = mock(analyzeImplementation);
    const capture = mock(() => undefined);
    const services: AnalysisServices = {
      engine: {
        readiness: () => ({ available: true }),
        analyze,
      },
      debugSink: { capture },
    };
    render(
      <AnalysisSessionProvider services={services}>
        <AnalysisSetupPage />
      </AnalysisSessionProvider>,
    );

    const fileInput = document.querySelector<HTMLInputElement>("#file-input");
    expect(fileInput).not.toBeNull();
    fireEvent.change(fileInput!, {
      target: {
        files: [new File(["video"], "replay.mp4", { type: "video/mp4" })],
      },
    });
    const analyzeButton =
      document.querySelector<HTMLButtonElement>(".analyze-btn");
    if (!analyzeButton) throw new Error("analyze button not rendered");
    expect(analyzeButton.disabled).toBe(true);
    fireEvent.change(document.querySelector("#side-select")!, {
      target: { value: "p2" },
    });
    fireEvent.change(document.querySelector("#char-select")!, {
      target: { value: "JURI" },
    });
    fireEvent.change(document.querySelector("#opponent-char-select")!, {
      target: { value: "KEN" },
    });
    fireEvent.click(analyzeButton);

    await waitFor(() => expect(analyze).toHaveBeenCalledTimes(1));
    expect(capture).toHaveBeenCalledTimes(1);
    expect(analyzeButton.disabled).toBe(false);
  });
});
