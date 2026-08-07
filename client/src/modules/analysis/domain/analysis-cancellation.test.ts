import { describe, expect, test } from "bun:test";
import {
  AnalysisCanceledError,
  isAnalysisCanceled,
} from "./analysis-cancellation.js";

describe("analysis cancellation", () => {
  test("中止を利用者向けの既定文言で表す", () => {
    const error = new AnalysisCanceledError();

    expect(error).toBeInstanceOf(Error);
    expect(error.message).toBe("動画解析を中止しました");
    expect(error.name).toBe("AnalysisCanceledError");
  });

  test("呼び出し側が文脈を足した文言を使える", () => {
    const error = new AnalysisCanceledError("別のファイルを選び直しました");

    expect(error.message).toBe("別のファイルを選び直しました");
    expect(error.name).toBe("AnalysisCanceledError");
  });

  /**
   * 中止は失敗表示に載せない正常系なので、他の例外と取り違えると
   * 利用者が操作しただけの場面をエラーとして見せてしまう。
   */
  test("中止だけを中止として判定する", () => {
    expect(isAnalysisCanceled(new AnalysisCanceledError())).toBe(true);
    expect(isAnalysisCanceled(new AnalysisCanceledError("任意の文言"))).toBe(
      true,
    );

    for (const other of [
      new Error("動画解析を中止しました"),
      new TypeError("boom"),
      { name: "AnalysisCanceledError", message: "動画解析を中止しました" },
      "動画解析を中止しました",
      null,
      undefined,
    ]) {
      expect(isAnalysisCanceled(other)).toBe(false);
    }
  });
});
