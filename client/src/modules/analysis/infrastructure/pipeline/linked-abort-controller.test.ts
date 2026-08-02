import { describe, expect, test } from "bun:test";
import { AnalysisCanceledError } from "../../domain/analysis-cancellation.js";
import { LinkedAbortController } from "./linked-abort-controller.js";

describe("LinkedAbortController", () => {
  test("利用者の中止が先なら後続のstall理由で上書きしない", () => {
    const caller = new AbortController();
    const linked = new LinkedAbortController(caller.signal);
    const canceled = new AnalysisCanceledError();

    caller.abort(canceled);
    linked.abort(new Error("stall"));

    expect(linked.signal.aborted).toBe(true);
    expect(linked.signal.reason).toBe(canceled);
    linked.dispose();
  });

  test("stallが先なら後続の利用者中止で理由を上書きしない", () => {
    const caller = new AbortController();
    const linked = new LinkedAbortController(caller.signal);
    const stalled = new Error("stall");

    linked.abort(stalled);
    caller.abort(new AnalysisCanceledError());

    expect(linked.signal.reason).toBe(stalled);
    linked.dispose();
    linked.dispose();
  });
});
