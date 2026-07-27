export const OFFICIAL_HASHTAG = "FighterNotes";
export const NATIVE_SHARE_TEXT = `SF6の対戦分析結果 #${OFFICIAL_HASHTAG}`;

export function buildXIntentUrl(shareUrl: string): string {
  const intent = new URL("https://x.com/intent/tweet");
  intent.searchParams.set("text", "SF6の対戦分析結果 | Fighter Notes");
  intent.searchParams.set("url", shareUrl);
  intent.searchParams.set("hashtags", OFFICIAL_HASHTAG);
  return intent.toString();
}
