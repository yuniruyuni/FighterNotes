import { describe, expect, mock, test } from "bun:test";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import type { AnalysisServices } from "../application/ports.js";
import type {
  ValidatedVideoInput,
  VideoPreflightResult,
} from "../domain/video-preflight.js";
import {
  AnalysisSessionProvider,
  useAnalysisSession,
} from "./AnalysisSessionProvider.js";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolveValue, rejectValue) => {
    resolve = resolveValue;
    reject = rejectValue;
  });
  return { promise, resolve, reject };
}

function validatedVideo(file: File): ValidatedVideoInput {
  return {
    file,
    identity: {
      name: file.name,
      size: file.size,
      lastModified: file.lastModified,
      type: file.type,
    },
    metadataBytesRead: 2048,
    track: {
      trackId: 1,
      codec: "avc1.640028",
      codedWidth: 1920,
      codedHeight: 1080,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 0,
      framesPerSecond: 59.94,
      constantFrameRate: true,
      totalSamples: 600,
      maxSampleBytes: 1024,
      timescale: 60_000,
      duration: 600_600,
      decoderConfig: {
        codec: "avc1.640028",
        codedWidth: 1920,
        codedHeight: 1080,
      },
      codecConfig: {
        codec: "avc1.640028",
        width: 1920,
        height: 1080,
      },
    },
  };
}

function services(
  preflight: AnalysisServices["engine"]["preflight"],
  analyze: AnalysisServices["engine"]["analyze"] = async () =>
    syntheticAnalysisResult(),
): AnalysisServices {
  return {
    engine: {
      readiness: () => ({ available: true }),
      preflight,
      analyze,
    },
    debugSink: { capture: () => undefined },
  };
}

function Harness({ first, second }: { first: File; second?: File }) {
  const session = useAnalysisSession();
  return (
    <>
      <output data-testid="file">{session.state.file?.name ?? "none"}</output>
      <output data-testid="preflight">
        {session.state.videoPreflight.status}
      </output>
      {session.state.videoPreflight.status === "invalid" && (
        <output data-testid="message">
          {session.state.videoPreflight.message}
        </output>
      )}
      <button type="button" onClick={() => session.setFile(first)}>
        first
      </button>
      {second && (
        <button type="button" onClick={() => session.setFile(second)}>
          second
        </button>
      )}
      <button type="button" onClick={() => session.setFile(null)}>
        clear
      </button>
      <button
        type="button"
        onClick={() => {
          session.setSide("p1");
          session.setOwnCharacter("JURI");
          session.setOpponentCharacter("KEN");
        }}
      >
        configure
      </button>
      <button type="button" onClick={() => void session.analyze()}>
        analyze
      </button>
    </>
  );
}

describe("AnalysisSessionProvider video preflight", () => {
  test("検証済みmetadataを同じFileの解析へ渡す", async () => {
    const file = new File(["video"], "valid.mp4", { type: "video/mp4" });
    const validated = validatedVideo(file);
    const preflight = deferred<VideoPreflightResult>();
    const analyzeImplementation: AnalysisServices["engine"]["analyze"] =
      async () => syntheticAnalysisResult();
    const analyze = mock(analyzeImplementation);
    render(
      <AnalysisSessionProvider
        services={services(async () => preflight.promise, analyze)}
      >
        <Harness first={file} />
      </AnalysisSessionProvider>,
    );
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "first" }));
    expect(screen.getByTestId("preflight")).toHaveTextContent("checking");
    await act(async () => {
      preflight.resolve({ status: "valid", video: validated });
      await Promise.resolve();
    });
    await waitFor(() =>
      expect(screen.getByTestId("preflight")).toHaveTextContent("valid"),
    );
    await user.click(screen.getByRole("button", { name: "configure" }));
    await user.click(screen.getByRole("button", { name: "analyze" }));
    await waitFor(() => expect(analyze).toHaveBeenCalledTimes(1));
    expect(analyze.mock.calls[0]?.[0]).toBe(file);
    expect(analyze.mock.calls[0]?.[1]).toBe(validated);
  });

  test("旧selectionをabortし、遅延結果を新しい動画へ混入させない", async () => {
    const first = new File(["first"], "first.mp4", { type: "video/mp4" });
    const second = new File(["second"], "second.mp4", {
      type: "video/mp4",
    });
    const firstResult = deferred<VideoPreflightResult>();
    const secondResult = deferred<VideoPreflightResult>();
    const signals: AbortSignal[] = [];
    render(
      <AnalysisSessionProvider
        services={services((file, signal) => {
          signals.push(signal);
          return file === first ? firstResult.promise : secondResult.promise;
        })}
      >
        <Harness first={first} second={second} />
      </AnalysisSessionProvider>,
    );
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "first" }));
    await user.click(screen.getByRole("button", { name: "second" }));
    expect(signals[0]?.aborted).toBe(true);
    expect(signals[1]?.aborted).toBe(false);

    await act(async () => {
      firstResult.resolve({
        status: "valid",
        video: validatedVideo(first),
      });
      await Promise.resolve();
    });
    expect(screen.getByTestId("file")).toHaveTextContent("second.mp4");
    expect(screen.getByTestId("preflight")).toHaveTextContent("checking");

    await act(async () => {
      secondResult.resolve({
        status: "invalid",
        code: "variable_frame_rate",
        message: "second is VFR",
      });
      await Promise.resolve();
    });
    expect(screen.getByTestId("preflight")).toHaveTextContent("invalid");
    expect(screen.getByTestId("message")).toHaveTextContent("second is VFR");
  });

  test("clear・unmount・unexpected failureを安全に処理する", async () => {
    const file = new File(["video"], "pending.mp4", { type: "video/mp4" });
    const pending = deferred<VideoPreflightResult>();
    let signal: AbortSignal | undefined;
    const view = render(
      <AnalysisSessionProvider
        services={services((_file, currentSignal) => {
          signal = currentSignal;
          return pending.promise;
        })}
      >
        <Harness first={file} />
      </AnalysisSessionProvider>,
    );
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "first" }));
    await user.click(screen.getByRole("button", { name: "clear" }));
    expect(signal?.aborted).toBe(true);
    expect(screen.getByTestId("preflight")).toHaveTextContent("idle");
    await user.click(screen.getByRole("button", { name: "first" }));
    const unmountSignal = signal;
    view.unmount();
    expect(unmountSignal?.aborted).toBe(true);

    render(
      <AnalysisSessionProvider
        services={services(async () => {
          throw new Error("unexpected parser failure");
        })}
      >
        <Harness first={file} />
      </AnalysisSessionProvider>,
    );
    await user.click(screen.getByRole("button", { name: "first" }));
    await waitFor(() =>
      expect(screen.getByTestId("preflight")).toHaveTextContent("invalid"),
    );
    expect(screen.getByTestId("message")).toHaveTextContent(
      "unexpected parser failure",
    );
  });
});
