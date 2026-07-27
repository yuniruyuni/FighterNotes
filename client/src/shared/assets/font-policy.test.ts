import { describe, expect, test } from "bun:test";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

function filesUnder(directory: string): string[] {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesUnder(path) : [path];
  });
}

const clientSourceFiles = filesUnder(join(import.meta.dir, "../.."));
const clientBuildFiles = filesUnder(join(import.meta.dir, "../../../static"));
const baseCss = readFileSync(
  join(import.meta.dir, "../styles/base.css"),
  "utf8",
);
const packageJson = JSON.parse(
  readFileSync(join(import.meta.dir, "../../../package.json"), "utf8"),
) as { scripts?: Record<string, string> };

describe("system font policy", () => {
  test("Web fontの定義・配信asset・外部requestを持たない", () => {
    const fontFiles = [...clientSourceFiles, ...clientBuildFiles].filter(
      (path) => /\.(?:woff2?|ttf|otf)$/i.test(path),
    );
    const inspectableFiles = [...clientSourceFiles, ...clientBuildFiles].filter(
      (path) =>
        /\.(?:css|html|js|jsx|ts|tsx)$/i.test(path) &&
        !path.endsWith(".test.ts") &&
        !path.endsWith(".test.tsx"),
    );
    const browserSource = inspectableFiles
      .map((path) => readFileSync(path, "utf8"))
      .join("\n");
    const fontBuildScripts = Object.entries(packageJson.scripts ?? {}).filter(
      ([name, command]) => /(?:font|woff|ttf|otf)/i.test(`${name}\n${command}`),
    );

    expect(fontFiles).toEqual([]);
    expect(browserSource).not.toMatch(/@font-face/i);
    expect(browserSource).not.toMatch(/\.(?:woff2?|ttf|otf)(?:["')?#]|$)/i);
    expect(browserSource).not.toMatch(/fonts\.(?:googleapis|gstatic)\.com/i);
    expect(fontBuildScripts).toEqual([]);
  });

  test("本文用と見出し用のsystem font stackを宣言する", () => {
    expect(baseCss).toContain("--font-body:");
    expect(baseCss).toContain("system-ui, -apple-system, BlinkMacSystemFont");
    expect(baseCss).toContain('"Hiragino Sans"');
    expect(baseCss).toContain('"Yu Gothic UI"');
    expect(baseCss).toContain("--font-head:");
    expect(baseCss).toContain('"Bahnschrift"');
    expect(baseCss).toContain('"DIN Condensed"');
    expect(baseCss).toContain('"DejaVu Sans Condensed"');
    expect(baseCss).toContain("font-stretch: condensed");
    expect(baseCss).toContain("font-family: var(--font-body)");
  });
});
