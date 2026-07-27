import { describe, expect, test } from "bun:test";
import { existsSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const html = readFileSync(
  join(import.meta.dir, "../../entrypoints/index.html"),
  "utf8",
);
const publicUrl = "https://fighter.yuniruyuni.net/";
const imageUrl = `${publicUrl}images/fighter-notes-ogp.jpg`;

describe("OGP / Twitter Card", () => {
  test("公開URLとlarge image cardを絶対URLで宣言する", () => {
    expect(html).toContain(`<link rel="canonical" href="${publicUrl}" />`);
    expect(html).toContain(`<meta property="og:url" content="${publicUrl}" />`);
    expect(html).toContain(`content="${imageUrl}"`);
    expect(html).toContain(
      '<meta name="twitter:card" content="summary_large_image" />',
    );
  });

  test("画像の形式・寸法・代替テキストを宣言する", () => {
    expect(html).toContain(
      '<meta property="og:image:type" content="image/jpeg" />',
    );
    expect(html).toContain('<meta property="og:image:width" content="1200" />');
    expect(html).toContain('<meta property="og:image:height" content="630" />');
    expect(html).toContain('property="og:image:alt"');
    expect(html).toContain('name="twitter:image:alt"');
  });

  test("OGP画像のソースファイルが存在する", () => {
    const imagePath = join(import.meta.dir, "images", "fighter-notes-ogp.jpg");
    expect(existsSync(imagePath)).toBe(true);
    expect(statSync(imagePath).size).toBeGreaterThan(0);
  });
});
