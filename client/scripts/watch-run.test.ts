import { describe, expect, test } from "bun:test";
import { createMatcher, parseArgs, watchRoot } from "./watch-run";

describe("parseArgs", () => {
  test("splits globs and the command at --", () => {
    expect(
      parseArgs(["a/*.css", "b/*.css", "--", "bun", "run", "build"]),
    ).toEqual({
      patterns: ["a/*.css", "b/*.css"],
      command: ["bun", "run", "build"],
    });
  });

  test("rejects a missing separator, glob or command", () => {
    expect(() => parseArgs(["a/*.css"])).toThrow("usage");
    expect(() => parseArgs(["--", "bun"])).toThrow("usage");
    expect(() => parseArgs(["a/*.css", "--"])).toThrow("usage");
  });
});

describe("watchRoot", () => {
  test("stops at the first segment with a wildcard", () => {
    expect(watchRoot("/repo/client/src/**/*.css")).toBe("/repo/client/src");
    expect(watchRoot("/repo/client/src/images/*.jpg")).toBe(
      "/repo/client/src/images",
    );
  });

  test("watches the parent of a plain file", () => {
    expect(watchRoot("/repo/DATA_NOTICE.md")).toBe("/repo");
  });
});

describe("createMatcher", () => {
  const matches = createMatcher(
    ["src/**/*.css", "../DATA_NOTICE.md"],
    "/repo/client",
  );

  test("matches files the globs describe, relative to cwd", () => {
    expect(matches("/repo/client/src/shared/styles/base.css")).toBe(true);
    expect(matches("/repo/DATA_NOTICE.md")).toBe(true);
  });

  test("ignores everything else", () => {
    expect(matches("/repo/client/src/shared/styles/base.ts")).toBe(false);
    expect(matches("/repo/client/static/styles/base.css")).toBe(false);
    expect(matches("/repo/README.md")).toBe(false);
  });
});
