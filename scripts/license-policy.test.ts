import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  canonicalizeSpdxExpression,
  extractCopyrightNotices,
  licensePolicy,
  parseSpdxExpression,
  validateLicenseExpression,
} from "./license-policy";

describe("SPDX license policy", () => {
  test("single、OR、AND、WITHを含む式を構文解析する", () => {
    expect([...parseSpdxExpression("MIT").licenses]).toEqual(["MIT"]);
    expect([
      ...parseSpdxExpression("(MIT OR Apache-2.0) AND Unicode-3.0").licenses,
    ]).toEqual(["MIT", "Apache-2.0", "Unicode-3.0"]);
    expect([
      ...parseSpdxExpression("Apache-2.0 WITH LLVM-exception").exceptions,
    ]).toEqual(["LLVM-exception"]);
    expect(canonicalizeSpdxExpression("MIT OR Apache-2.0")).toBe(
      canonicalizeSpdxExpression("(Apache-2.0 OR MIT)"),
    );
    expect(canonicalizeSpdxExpression("MIT AND Apache-2.0")).not.toBe(
      canonicalizeSpdxExpression("MIT OR Apache-2.0"),
    );
  });

  test("不完全な式と未対応の文字を拒否する", () => {
    expect(() => parseSpdxExpression("MIT OR")).toThrow(/Invalid SPDX/);
    expect(() => parseSpdxExpression("(MIT AND Apache-2.0")).toThrow(
      /Invalid SPDX/,
    );
    expect(() => parseSpdxExpression("MIT / Apache-2.0")).toThrow(
      /Invalid SPDX/,
    );
    expect(() => parseSpdxExpression("(MIT) WITH LLVM-exception")).toThrow(
      /Invalid SPDX/,
    );
  });

  test("未審査・禁止・license file参照だけの宣言を拒否する", () => {
    expect(() => validateLicenseExpression("MPL-2.0", "example@1")).toThrow(
      /outside the reviewed policy/,
    );
    expect(() =>
      validateLicenseExpression("AGPL-3.0-only", "example@1"),
    ).toThrow(/prohibited licenses/);
    expect(() =>
      validateLicenseExpression("SEE LICENSE IN LICENSE.txt", "example@1"),
    ).toThrow(/unsupported license/);
  });

  test("npm/Cargoで同じ許可license一覧を使用する", () => {
    const cargoPolicy = Bun.TOML.parse(
      readFileSync(join(import.meta.dir, "../about.toml"), "utf8"),
    ) as { accepted: string[] };
    expect(cargoPolicy.accepted).toEqual(
      licensePolicy.allowedLicenseIdentifiers,
    );
  });
});

describe("copyright notice extraction", () => {
  test("明示されたcopyright行だけを重複なく抽出する", () => {
    expect(
      extractCopyrightNotices([
        {
          text: [
            "Copyright (c) 2026 Example",
            "Copyright (c) 2026 Example",
            "THE COPYRIGHT HOLDERS SHALL NOT BE LIABLE",
          ].join("\n"),
        },
      ]),
    ).toEqual(["Copyright (c) 2026 Example"]);
  });

  test("個別表記がなければ完全な通知文を参照させる", () => {
    expect(extractCopyrightNotices([{ text: "MIT License" }])).toEqual([
      "Not separately stated in the distributed files; see the complete license and notice text.",
    ]);
  });
});
