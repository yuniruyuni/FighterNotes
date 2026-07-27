import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

const STYLE_VERSION_PLACEHOLDER = "__STYLES_VERSION__";
const stylesheetPaths = [
  "../src/shared/styles/base.css",
  "../src/shared/styles/legal.css",
  "../src/modules/analysis/ui/setup/setup.css",
  "../src/modules/results/ui/workspace/workspace.css",
  "../src/modules/results/ui/summary/summary.css",
  "../src/modules/results/ui/player/player-debug.css",
] as const;

export function stylesheetVersion(
  stylesheets: readonly (string | Uint8Array)[],
): string {
  const hash = createHash("sha256");
  for (const stylesheet of stylesheets) {
    hash.update(stylesheet);
    hash.update("\0");
  }
  return hash.digest("hex").slice(0, 16);
}

export function renderIndexHtml(template: string, version: string): string {
  if (!template.includes(STYLE_VERSION_PLACEHOLDER)) {
    throw new Error("index.html is missing the stylesheet version placeholder");
  }
  return template.replaceAll(STYLE_VERSION_PLACEHOLDER, version);
}

async function buildHtml(): Promise<void> {
  const templatePath = join(import.meta.dir, "../src/entrypoints/index.html");
  const outputDirectory = join(import.meta.dir, "../static");
  const stylesheets = await Promise.all(
    stylesheetPaths.map((relativePath) =>
      readFile(join(import.meta.dir, relativePath)),
    ),
  );
  const template = await readFile(templatePath, "utf8");
  const output = renderIndexHtml(template, stylesheetVersion(stylesheets));

  await mkdir(outputDirectory, { recursive: true });
  await writeFile(join(outputDirectory, "index.html"), output);
}

if (import.meta.main) {
  await buildHtml();
}
