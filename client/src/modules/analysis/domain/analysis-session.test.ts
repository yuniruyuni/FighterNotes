import { describe, expect, test } from "bun:test";
import { AnalysisSession } from "./analysis-session.js";
import type { AdviceReport } from "./report.js";
import type { AnalysisResult } from "./result.js";

describe("analysis session reducer", () => {
  test("初期状態を毎回独立して生成する", () => {
    const first = AnalysisSession.initial();
    const second = AnalysisSession.initial();

    expect(first).toEqual({
      file: null,
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
    const ready = {
      ...AnalysisSession.initial(),
      file: new File(["video"], "replay.mp4"),
      side: "p1" as const,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    };

    expect(AnalysisSession.canStart(ready)).toBe(true);
    expect(AnalysisSession.canStart({ ...ready, file: null })).toBe(false);
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
      side: "p2" as const,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
      phase: "ready" as const,
      progress: 100,
    };

    expect(AnalysisSession.reduce(configured, { type: "reset" })).toEqual({
      ...AnalysisSession.initial(),
      file,
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
});
