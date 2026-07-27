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
      side: "p1",
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
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    };

    expect(AnalysisSession.canStart(ready)).toBe(true);
    expect(AnalysisSession.canStart({ ...ready, file: null })).toBe(false);
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
    const initial = { ...AnalysisSession.initial(), error: "old error" };
    const file = new File(["video"], "replay.mp4");
    const withFile = AnalysisSession.reduce(initial, { type: "file", file });
    expect(withFile.error).toBe("");
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

  test("設定を保ったまま解析状態をリセットする", () => {
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
      side: "p2",
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
    });
  });

  test("進捗を百分率に変換する", () => {
    const started = AnalysisSession.reduce(AnalysisSession.initial(), {
      type: "start",
    });
    const progressed = AnalysisSession.reduce(started, {
      type: "progress",
      progress: 0.426,
      status: "HUDを解析中",
    });

    expect(progressed.phase).toBe("analyzing");
    expect(progressed.progress).toBe(43);
    expect(progressed.status).toBe("HUDを解析中");
  });
});
