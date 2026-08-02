import { describe, expect, mock, test } from "bun:test";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { AnalysisServices } from "~/modules/analysis/application/ports.js";
import { AnalysisCanceledError } from "~/modules/analysis/domain/analysis-cancellation.js";
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
    expect(screen.getByRole("status")).toHaveTextContent(
      "動画解析が完了しました。",
    );
  });

  test("小数の可視進捗と名前付きprogressbar、工程live statusを表示する", async () => {
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] = (
      _file,
      _side,
      onProgress,
      _context,
      signal,
    ) => {
      onProgress(0.426, "フレーム 426 / 1000");
      return new Promise((_resolve, reject) => {
        signal.addEventListener("abort", () => reject(signal.reason), {
          once: true,
        });
      });
    };
    const rendered = renderSetup(analysisServices(analyzeImplementation));
    configureAnalysis();

    fireEvent.click(screen.getByRole("button", { name: "解析する" }));

    const progress = await screen.findByRole("progressbar", {
      name: "動画解析の進捗",
    });
    expect(progress).toHaveAttribute("value", "42.6");
    expect(
      screen.getByText("42.6%", { selector: "span.analysis-progress-percent" }),
    ).toBeTruthy();
    expect(screen.getByRole("status")).toHaveTextContent(
      "動画フレームを解析中です。",
    );
    rendered.unmount();
  });

  test("解析を一度だけ中止し、同じ設定ですぐ再試行できる", async () => {
    let abortEvents = 0;
    let cancellationReason: unknown;
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] = (
      _file,
      _side,
      _onProgress,
      _context,
      signal,
    ) =>
      new Promise((_resolve, reject) => {
        signal.addEventListener(
          "abort",
          () => {
            abortEvents += 1;
            cancellationReason = signal.reason;
            reject(signal.reason);
          },
          { once: true },
        );
      });
    const services = analysisServices(analyzeImplementation);
    renderSetup(services);
    configureAnalysis();

    fireEvent.click(screen.getByRole("button", { name: "解析する" }));
    const cancel = await screen.findByRole("button", { name: "解析を中止" });
    fireEvent.click(cancel);
    fireEvent.click(cancel);

    await screen.findByText("解析を中止しました");
    expect(abortEvents).toBe(1);
    expect(cancellationReason).toBeInstanceOf(AnalysisCanceledError);
    expect(screen.queryByText("解析エラー")).toBeNull();
    expect(
      (document.querySelector("#side-select") as HTMLSelectElement).value,
    ).toBe("p2");
    expect(
      (document.querySelector("#char-select") as HTMLSelectElement).value,
    ).toBe("JURI");
    expect(
      (document.querySelector("#opponent-char-select") as HTMLSelectElement)
        .value,
    ).toBe("KEN");
    expect(screen.getByRole("button", { name: "解析する" })).toBeEnabled();
    expect(screen.getByRole("status")).toHaveTextContent(
      "動画解析を中止しました。",
    );
  });

  test("解析失敗をassertive live regionへ即時通知する", async () => {
    const services = analysisServices(async () => {
      throw new Error("decoder failure");
    });
    renderSetup(services);
    configureAnalysis();

    fireEvent.click(screen.getByRole("button", { name: "解析する" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "エラー: decoder failure",
    );
  });

  test("中止後に遅れて完了しても結果として採用しない", async () => {
    let resolveAnalysis!: (
      value: ReturnType<typeof syntheticAnalysisResult>,
    ) => void;
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] = () =>
      new Promise((resolve) => {
        resolveAnalysis = resolve;
      });
    const services = analysisServices(analyzeImplementation);
    const capture = services.debugSink.capture;
    renderSetup(services);
    configureAnalysis();

    fireEvent.click(screen.getByRole("button", { name: "解析する" }));
    fireEvent.click(await screen.findByRole("button", { name: "解析を中止" }));
    resolveAnalysis(syntheticAnalysisResult());

    await screen.findByText("解析を中止しました");
    expect(capture).not.toHaveBeenCalled();
  });

  test("解析中に画面を破棄すると処理を中止する", async () => {
    let signal: AbortSignal | undefined;
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] = (
      _file,
      _side,
      _onProgress,
      _context,
      currentSignal,
    ) => {
      signal = currentSignal;
      return new Promise((_resolve, reject) => {
        currentSignal.addEventListener(
          "abort",
          () => reject(currentSignal.reason),
          { once: true },
        );
      });
    };
    const rendered = renderSetup(analysisServices(analyzeImplementation));
    configureAnalysis();
    fireEvent.click(screen.getByRole("button", { name: "解析する" }));
    await screen.findByRole("button", { name: "解析を中止" });

    rendered.unmount();

    expect(signal?.aborted).toBe(true);
    expect(signal?.reason).toBeInstanceOf(AnalysisCanceledError);
  });
});

function analysisServices(
  analyze: AnalysisServices["engine"]["analyze"],
): AnalysisServices {
  return {
    engine: { readiness: () => ({ available: true }), analyze },
    debugSink: { capture: mock(() => undefined) },
  };
}

function renderSetup(services: AnalysisServices): ReturnType<typeof render> {
  return render(
    <AnalysisSessionProvider services={services}>
      <AnalysisSetupPage />
    </AnalysisSessionProvider>,
  );
}

function configureAnalysis(): void {
  const fileInput = document.querySelector<HTMLInputElement>("#file-input");
  if (!fileInput) throw new Error("file input not rendered");
  fireEvent.change(fileInput, {
    target: {
      files: [new File(["video"], "replay.mp4", { type: "video/mp4" })],
    },
  });
  fireEvent.change(document.querySelector("#side-select")!, {
    target: { value: "p2" },
  });
  fireEvent.change(document.querySelector("#char-select")!, {
    target: { value: "JURI" },
  });
  fireEvent.change(document.querySelector("#opponent-char-select")!, {
    target: { value: "KEN" },
  });
}
