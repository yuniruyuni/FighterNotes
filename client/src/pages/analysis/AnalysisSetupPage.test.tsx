import { describe, expect, mock, test } from "bun:test";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
    const user = userEvent.setup();

    render(
      <AnalysisSessionProvider services={services}>
        <AnalysisSetupPage />
      </AnalysisSessionProvider>,
    );

    const fileInput = document.querySelector<HTMLInputElement>("#file-input");
    expect(fileInput).not.toBeNull();
    await user.upload(
      fileInput!,
      new File(["video"], "replay.mp4", { type: "video/mp4" }),
    );
    await user.selectOptions(
      screen.getByLabelText(/自分のキャラクター/),
      "JURI",
    );
    await user.selectOptions(
      screen.getByLabelText(/相手のキャラクター/),
      "KEN",
    );
    await user.click(screen.getByRole("button", { name: "解析する" }));

    await waitFor(() => expect(analyze).toHaveBeenCalledTimes(1));
    expect(capture).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "解析する" })).toBeEnabled();
  });
});
