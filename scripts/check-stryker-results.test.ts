import { describe, expect, test } from "bun:test";
import { summarizeMutationReport } from "./check-stryker-results";

const mutant = (status: string, id: string) => ({
  id,
  status,
  mutatorName: "ConditionalExpression",
  location: { start: { line: 3, column: 5 } },
});

describe("Stryker result policy", () => {
  test("killed mutants pass while compile errors remain inconclusive", () => {
    expect(
      summarizeMutationReport({
        files: {
          "src/model.ts": {
            mutants: [mutant("Killed", "1"), mutant("CompileError", "2")],
          },
        },
      } as Parameters<typeof summarizeMutationReport>[0]),
    ).toEqual({
      total: 2,
      counts: { Killed: 1, CompileError: 1 },
      blocking: [],
    });
  });

  test("survivors, timeouts and undocumented ignores require review", () => {
    const report = summarizeMutationReport({
      files: {
        "src/model.ts": {
          mutants: [
            mutant("Survived", "1"),
            mutant("Timeout", "2"),
            mutant("Ignored", "3"),
          ],
        },
      },
    } as Parameters<typeof summarizeMutationReport>[0]);

    expect(report.blocking.map(({ mutant }) => mutant.status)).toEqual([
      "Survived",
      "Timeout",
      "Ignored",
    ]);
  });

  test("reason付きの同値mutant除外を許可する", () => {
    const ignored = {
      ...mutant("Ignored", "1"),
      statusReason: "Equivalent for an empty specification payload",
    };
    const report = summarizeMutationReport({
      files: { "src/model.ts": { mutants: [ignored] } },
    } as Parameters<typeof summarizeMutationReport>[0]);

    expect(report.blocking).toEqual([]);
  });
});
