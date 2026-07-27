import { describe, expect, test } from "bun:test";
import { isGeneratedDeleteCode } from "../domain/delete-code.js";
import { browserSharingServices } from "./browser-sharing-services.js";

describe("browser sharing services", () => {
  test("暗号学的乱数から有効な削除コードを生成する", () => {
    expect(
      isGeneratedDeleteCode(browserSharingServices.generateDeleteCode()),
    ).toBe(true);
  });
});
