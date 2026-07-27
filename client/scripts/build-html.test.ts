import { describe, expect, test } from "bun:test";
import { renderIndexHtml, stylesheetVersion } from "./build-html";

describe("stylesheet cache version", () => {
  test("all stylesheets contribute to a stable content hash", () => {
    const version = stylesheetVersion(["base", "legal"]);

    expect(version).toMatch(/^[a-f0-9]{16}$/);
    expect(stylesheetVersion(["base", "legal"])).toBe(version);
    expect(stylesheetVersion(["base changed", "legal"])).not.toBe(version);
    expect(stylesheetVersion(["base", "legal changed"])).not.toBe(version);
  });

  test("every stylesheet URL receives the generated version", () => {
    const output = renderIndexHtml(
      [
        '<link rel="stylesheet" href="/styles/base.css?v=__STYLES_VERSION__" />',
        '<link rel="stylesheet" href="/styles/legal.css?v=__STYLES_VERSION__" />',
      ].join("\n"),
      "1234abcd",
    );

    expect(output).not.toContain("__STYLES_VERSION__");
    expect(output.match(/\?v=1234abcd/g)).toHaveLength(2);
  });

  test("fails instead of silently shipping unversioned stylesheets", () => {
    expect(() => renderIndexHtml("<html></html>", "1234abcd")).toThrow(
      "stylesheet version placeholder",
    );
  });
});
