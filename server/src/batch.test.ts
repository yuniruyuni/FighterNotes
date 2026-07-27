import { describe, expect, test } from "bun:test";
import { parseBatchCommand } from "./batch";

describe("parseBatchCommand", () => {
  test("引数がなければ通常のserver modeになる", () => {
    expect(parseBatchCommand(["/app/server"])).toBeNull();
  });

  test("compiled binaryとbun runのどちらでもcleanupを選べる", () => {
    expect(parseBatchCommand(["/app/server", "--batch=cleanup"])).toBe(
      "cleanup",
    );
    expect(
      parseBatchCommand([
        "/usr/local/bin/bun",
        "server/src/index.ts",
        "--batch=cleanup",
      ]),
    ).toBe("cleanup");
  });

  test("未知または重複したbatch指定は拒否する", () => {
    expect(() => parseBatchCommand(["server", "--batch=unknown"])).toThrow(
      "Unknown batch command",
    );
    expect(() =>
      parseBatchCommand(["server", "--batch=cleanup", "--batch=cleanup"]),
    ).toThrow("Only one --batch argument");
  });
});
