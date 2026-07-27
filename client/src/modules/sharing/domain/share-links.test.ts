import { describe, expect, test } from "bun:test";
import {
  buildXIntentUrl,
  NATIVE_SHARE_TEXT,
  OFFICIAL_HASHTAG,
} from "./share-links";

describe("share link helpers", () => {
  test("X Intentへ定型文と共有URLだけを渡す", () => {
    const shareUrl = "https://fighter.example/s/Abcdefghijklmnopqrstu_";
    const result = new URL(buildXIntentUrl(shareUrl));
    expect(result.origin).toBe("https://x.com");
    expect(result.pathname).toBe("/intent/tweet");
    expect(result.searchParams.get("text")).toBe(
      "SF6の対戦分析結果 | Fighter Notes",
    );
    expect(result.searchParams.get("url")).toBe(shareUrl);
    expect(OFFICIAL_HASHTAG).toBe("FighterNotes");
    expect(result.searchParams.get("hashtags")).toBe("FighterNotes");
    expect(NATIVE_SHARE_TEXT).toBe("SF6の対戦分析結果 #FighterNotes");
  });
});
