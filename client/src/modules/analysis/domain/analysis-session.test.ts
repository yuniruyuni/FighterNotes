import { describe, expect, test } from "bun:test";
import { AnalysisSession } from "./analysis-session.js";
import type { AdviceReport } from "./report.js";
import type { AnalysisResult } from "./result.js";
import type { ValidatedVideoInput } from "./video-preflight.js";

function validatedVideo(file: File): ValidatedVideoInput {
  return {
    file,
    identity: {
      name: file.name,
      size: file.size,
      lastModified: file.lastModified,
      type: file.type,
    },
    metadataBytesRead: 1024,
    track: {
      trackId: 1,
      codec: "avc1.640028",
      codedWidth: 1920,
      codedHeight: 1080,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 0,
      framesPerSecond: 60,
      constantFrameRate: true,
      totalSamples: 600,
      maxSampleBytes: 1024,
      timescale: 60_000,
      duration: 600_000,
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

describe("analysis session reducer", () => {
  test("初期状態を毎回独立して生成する", () => {
    const first = AnalysisSession.initial();
    const second = AnalysisSession.initial();

    expect(first).toEqual({
      file: null,
      videoPreflight: { status: "idle" },
      side: "",
      ownCharacter: "",
      opponentCharacter: "",
      phase: "setup",
      progress: 0,
      status: "",
      error: "",
      result: null,
      report: null,
      context: null,
    });
    expect(first).not.toBe(second);
  });

  test("ファイルと両キャラクターが揃い、解析中でなければ開始できる", () => {
    const file = new File(["video"], "replay.mp4");
    const ready = {
      ...AnalysisSession.initial(),
      file,
      videoPreflight: {
        status: "valid" as const,
        video: validatedVideo(file),
      },
      side: "p1" as const,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    };

    expect(AnalysisSession.canStart(ready)).toBe(true);
    expect(AnalysisSession.canStart({ ...ready, file: null })).toBe(false);
    expect(
      AnalysisSession.canStart({
        ...ready,
        videoPreflight: { status: "checking" },
      }),
    ).toBe(false);
    expect(AnalysisSession.canStart({ ...ready, side: "" })).toBe(false);
    expect(AnalysisSession.canStart({ ...ready, ownCharacter: "" })).toBe(
      false,
    );
    expect(AnalysisSession.canStart({ ...ready, opponentCharacter: "" })).toBe(
      false,
    );
    expect(AnalysisSession.canStart({ ...ready, phase: "analyzing" })).toBe(
      false,
    );
  });

  test("設定actionを適用し、再入力時に以前のエラーを消す", () => {
    const initial = {
      ...AnalysisSession.initial(),
      side: "p1" as const,
      error: "old error",
    };
    const file = new File(["video"], "replay.mp4");
    const withFile = AnalysisSession.reduce(initial, { type: "file", file });
    expect(withFile.error).toBe("");
    expect(withFile.side).toBe("");
    expect(withFile.videoPreflight).toEqual({ status: "checking" });
    const withSide = AnalysisSession.reduce(withFile, {
      type: "side",
      side: "p2",
    });
    const withOwn = AnalysisSession.reduce(withSide, {
      type: "ownCharacter",
      character: "JURI",
    });
    expect(withOwn.error).toBe("");
    const withOpponent = AnalysisSession.reduce(withOwn, {
      type: "opponentCharacter",
      character: "KEN",
    });

    expect(withOpponent).toMatchObject({
      file,
      side: "p2",
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
      error: "",
    });
  });

  test("選択中の動画だけにpreflight結果を適用する", () => {
    const file = new File(["video"], "replay.mp4");
    const selected = AnalysisSession.reduce(AnalysisSession.initial(), {
      type: "file",
      file,
    });
    const valid = AnalysisSession.reduce(selected, {
      type: "videoPreflightValid",
      video: validatedVideo(file),
    });
    expect(valid.videoPreflight.status).toBe("valid");

    const other = new File(["other"], "other.mp4");
    expect(
      AnalysisSession.reduce(selected, {
        type: "videoPreflightValid",
        video: validatedVideo(other),
      }),
    ).toBe(selected);

    const failure = {
      status: "invalid" as const,
      code: "variable_frame_rate" as const,
      message: "VFRです",
    };
    expect(
      AnalysisSession.reduce(selected, {
        type: "videoPreflightInvalid",
        failure,
      }).videoPreflight,
    ).toBe(failure);
    expect(
      AnalysisSession.reduce(AnalysisSession.initial(), {
        type: "videoPreflightInvalid",
        failure,
      }),
    ).toEqual(AnalysisSession.initial());
    expect(
      AnalysisSession.reduce(selected, { type: "file", file: null })
        .videoPreflight,
    ).toEqual({ status: "idle" });
  });

  test("開始・完了・失敗で解析状態を遷移する", () => {
    const staleReport = {} as AdviceReport;
    const staleResult = { report: staleReport } as AnalysisResult;
    const configured = {
      ...AnalysisSession.initial(),
      error: "old error",
      result: staleResult,
      report: staleReport,
      context: { ownSide: "p1" as const, p1: {}, p2: {} },
    };
    const started = AnalysisSession.reduce(configured, { type: "start" });

    expect(started).toMatchObject({
      phase: "analyzing",
      progress: 0,
      status: "準備中…",
      error: "",
      result: null,
      report: null,
      context: null,
    });

    const report = {} as AdviceReport;
    const result = { report } as AnalysisResult;
    const context = { ownSide: "p2" as const, p1: {}, p2: {} };
    const completed = AnalysisSession.reduce(started, {
      type: "complete",
      result,
      report,
      context,
    });
    expect(completed).toMatchObject({
      phase: "ready",
      progress: 100,
      status: "",
      error: "",
      result,
      report,
      context,
    });

    expect(
      AnalysisSession.reduce(completed, { type: "fail", error: "failed" }),
    ).toMatchObject({ phase: "setup", status: "", error: "failed" });
  });

  test("中止中と中止済みを通常エラーと分け、遅延した進捗と完了を無視する", () => {
    const started = AnalysisSession.reduce(AnalysisSession.initial(), {
      type: "start",
    });
    const canceling = AnalysisSession.reduce(started, { type: "cancel" });

    expect(canceling).toMatchObject({
      phase: "canceling",
      status: "解析を中止しています…",
      error: "",
    });
    expect(AnalysisSession.reduce(canceling, { type: "cancel" })).toBe(
      canceling,
    );
    expect(
      AnalysisSession.reduce(canceling, {
        type: "progress",
        progress: 0.9,
        status: "遅延した進捗",
      }),
    ).toBe(canceling);

    const report = {} as AdviceReport;
    const result = { report } as AnalysisResult;
    expect(
      AnalysisSession.reduce(canceling, {
        type: "complete",
        result,
        report,
        context: { ownSide: "p1", p1: {}, p2: {} },
      }),
    ).toBe(canceling);

    const canceled = AnalysisSession.reduce(canceling, { type: "canceled" });
    expect(canceled).toMatchObject({
      phase: "canceled",
      progress: 0,
      status: "解析を中止しました。設定を確認して再試行できます。",
      error: "",
    });
  });

  test("動画とキャラクターを保ち、サイドの再確認を要求してリセットする", () => {
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const configured = {
      ...AnalysisSession.initial(),
      file,
      videoPreflight: {
        status: "valid" as const,
        video: validatedVideo(file),
      },
      side: "p2" as const,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
      phase: "ready" as const,
      progress: 100,
    };

    expect(AnalysisSession.reduce(configured, { type: "reset" })).toEqual({
      ...AnalysisSession.initial(),
      file,
      videoPreflight: configured.videoPreflight,
      side: "",
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    });
  });

  test("進捗を小数百分率へ変換し、遅延通知でも後退させない", () => {
    const started = AnalysisSession.reduce(AnalysisSession.initial(), {
      type: "start",
    });
    const progressed = AnalysisSession.reduce(started, {
      type: "progress",
      progress: 0.426,
      status: "HUDを解析中",
    });

    expect(progressed.phase).toBe("analyzing");
    expect(progressed.progress).toBe(42.6);
    expect(progressed.status).toBe("HUDを解析中");
    expect(
      AnalysisSession.reduce(progressed, {
        type: "progress",
        progress: 0.4,
        status: "遅れて届いた進捗",
      }).progress,
    ).toBe(42.6);
  });

  /**
   * 中止処理の最中に開始できると、走っている解析を残したまま二重に始まる。
   * 「解析中」と同じく開始不可でなければならない。
   */
  test("中止処理中は開始できない", () => {
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const ready = {
      ...AnalysisSession.initial(),
      file,
      videoPreflight: {
        status: "valid" as const,
        video: validatedVideo(file),
      },
      side: "p1" as const,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    };

    expect(AnalysisSession.canStart(ready)).toBe(true);
    expect(AnalysisSession.canStart({ ...ready, phase: "canceling" })).toBe(
      false,
    );
    expect(AnalysisSession.canStart({ ...ready, phase: "analyzing" })).toBe(
      false,
    );
    // 設定をやり直せる段階では、もう一度開始できる。
    for (const phase of ["setup", "canceled", "ready"] as const) {
      expect(AnalysisSession.canStart({ ...ready, phase })).toBe(true);
    }
  });

  /**
   * preflightは非同期に遅れて届く。設定段階を抜けた後の結果を適用すると、
   * 解析中の状態を後から書き換えてしまう。
   */
  test("設定段階を抜けた後に届いたpreflight結果を適用しない", () => {
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const selected = AnalysisSession.reduce(AnalysisSession.initial(), {
      type: "file",
      file,
    });
    const failure = {
      status: "invalid" as const,
      code: "variable_frame_rate" as const,
      message: "VFRです",
    };

    for (const phase of ["analyzing", "canceling", "canceled"] as const) {
      const running = { ...selected, phase };
      expect(
        AnalysisSession.reduce(running, {
          type: "videoPreflightValid",
          video: validatedVideo(file),
        }),
      ).toBe(running);
      expect(
        AnalysisSession.reduce(running, {
          type: "videoPreflightInvalid",
          failure,
        }),
      ).toBe(running);
    }
  });

  /**
   * 中止完了は、中止要求済みと解析中の両方から届きうる。それ以外の段階で
   * 受け取ると、設定中や完了済みの画面を中止表示へ戻してしまう。
   */
  test("中止完了を中止中と解析中からだけ受け付ける", () => {
    const base = AnalysisSession.initial();

    for (const phase of ["canceling", "analyzing"] as const) {
      expect(
        AnalysisSession.reduce({ ...base, phase }, { type: "canceled" }),
      ).toMatchObject({
        phase: "canceled",
        status: "解析を中止しました。設定を確認して再試行できます。",
      });
    }

    for (const phase of ["setup", "canceled", "ready"] as const) {
      const untouched = { ...base, phase };
      expect(AnalysisSession.reduce(untouched, { type: "canceled" })).toBe(
        untouched,
      );
    }
  });
});
