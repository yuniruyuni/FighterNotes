import { describe, expect, test } from "bun:test";
import { needsTimeoutRetry, type Result, report } from "./run-mutest.ts";

function result(overrides: Partial<Result> = {}): Result {
  return {
    crate: "hud-vision",
    total: 1744,
    detected: 1744,
    undetected: 0,
    timedOut: 0,
    crashed: 0,
    seconds: 109,
    ...overrides,
  };
}

describe("needsTimeoutRetry", () => {
  test("時間切れだけが問題なら測り直す", () => {
    expect(needsTimeoutRetry(result({ timedOut: 1 }))).toBe(true);
  });

  test("問題が無ければ測り直さない", () => {
    expect(needsTimeoutRetry(result())).toBe(false);
  });

  test("未検出・異常終了・失敗は測り直さない", () => {
    expect(needsTimeoutRetry(result({ timedOut: 1, undetected: 1 }))).toBe(
      false,
    );
    expect(needsTimeoutRetry(result({ timedOut: 1, crashed: 1 }))).toBe(false);
    expect(
      needsTimeoutRetry(
        result({ timedOut: 1, failure: "集計行が見つからない" }),
      ),
    ).toBe(false);
  });
});

function collect(results: Result[]): { problems: string[]; lines: string[] } {
  const lines: string[] = [];
  const original = console.log;
  console.log = (...args: unknown[]) => lines.push(args.join(" "));
  try {
    return { problems: report(results), lines };
  } finally {
    console.log = original;
  }
}

describe("report", () => {
  test("測り直しても時間切れなら失敗として挙げる", () => {
    const { problems, lines } = collect([
      result({ timedOut: 1, retried: true, seconds: 218 }),
    ]);

    expect(problems).toEqual(["hud-vision: 1 変異が測り直しても時間切れ"]);
    expect(lines[0]).toContain("測り直し済み");
  });

  test("未検出と異常終了と変異なしを区別して挙げる", () => {
    const { problems } = collect([
      result({ crate: "a", undetected: 2, detected: 1742 }),
      result({ crate: "b", crashed: 3 }),
      result({ crate: "c", total: 0, detected: 0 }),
      result({ crate: "d", failure: "集計行が見つからない" }),
    ]);

    expect(problems).toEqual([
      "a: 2 変異が未検出",
      "b: 3 変異でハーネスが異常終了",
      "c: テストからたどれるコードが無い",
      "d: 集計行が見つからない",
    ]);
  });

  test("問題が無ければ何も挙げない", () => {
    expect(collect([result()]).problems).toEqual([]);
  });
});
