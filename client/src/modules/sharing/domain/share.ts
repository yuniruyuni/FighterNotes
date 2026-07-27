export const SHARE_ID_PATTERN = /^[A-Za-z0-9_-]{22}$/;
export const MIN_DELETE_PASSWORD_LENGTH = 12;
export const MAX_DELETE_PASSWORD_LENGTH = 128;

export interface PublishedAnalysisShare {
  id: string;
  url: string;
  expiresAt: string;
}

export function isShareId(value: string): boolean {
  return SHARE_ID_PATTERN.test(value);
}

export function isValidDeletePassword(value: string): boolean {
  return (
    value.length >= MIN_DELETE_PASSWORD_LENGTH &&
    value.length <= MAX_DELETE_PASSWORD_LENGTH &&
    /\S/u.test(value)
  );
}

export function assertDeletePassword(value: string): void {
  if (!isValidDeletePassword(value)) {
    throw new Error(
      `削除用パスワードは${MIN_DELETE_PASSWORD_LENGTH}文字以上${MAX_DELETE_PASSWORD_LENGTH}文字以下で入力してください。`,
    );
  }
}

export function shareIdFromUrl(url: URL): string | undefined {
  if (
    !["https:", "http:"].includes(url.protocol) ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    return undefined;
  }
  const match = /^\/s\/([^/]+)$/.exec(url.pathname);
  return match?.[1] && isShareId(match[1]) ? match[1] : undefined;
}

export function shareIdFromPath(pathname: string): string | undefined {
  const match = /^\/s\/([^/]+)$/.exec(pathname);
  return match?.[1] && isShareId(match[1]) ? match[1] : undefined;
}
