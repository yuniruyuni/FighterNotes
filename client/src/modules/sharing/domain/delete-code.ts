const DELETE_CODE_ALPHABET = "23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const DELETE_CODE_GROUP_SIZE = 4;
const DELETE_CODE_GROUP_COUNT = 3;
const DELETE_CODE_PATTERN =
  /^[23456789A-HJ-NP-Z]{4}(?:-[23456789A-HJ-NP-Z]{4}){2}$/iu;

export type FillRandomValues = (values: Uint8Array) => Uint8Array;

/**
 * 60 bit の暗号学的乱数から、人が転記しやすい削除コードを作る。
 * 紛らわしい 0/O/1/I を除いた32文字を使うため、byteの下位5 bitを偏りなく利用できる。
 */
export function generateDeleteCode(fillRandomValues: FillRandomValues): string {
  const values = new Uint8Array(
    DELETE_CODE_GROUP_SIZE * DELETE_CODE_GROUP_COUNT,
  );
  fillRandomValues(values);

  const characters = Array.from(
    values,
    (value) => DELETE_CODE_ALPHABET[value & 31],
  );
  return Array.from({ length: DELETE_CODE_GROUP_COUNT }, (_, index) =>
    characters
      .slice(
        index * DELETE_CODE_GROUP_SIZE,
        (index + 1) * DELETE_CODE_GROUP_SIZE,
      )
      .join(""),
  ).join("-");
}

/** 発行済みコード形式だけを大文字へ正規化する。旧パスワードより先に適用してはならない。 */
export function normalizeDeleteCredential(value: string): string {
  return DELETE_CODE_PATTERN.test(value) ? value.toUpperCase() : value;
}

/** 旧パスワードを正確に保ちつつ、発行コードの小文字入力だけをfallback候補にする。 */
export function deleteCredentialCandidates(value: string): string[] {
  const normalized = normalizeDeleteCredential(value);
  return normalized === value ? [value] : [value, normalized];
}

export function isGeneratedDeleteCode(value: string): boolean {
  return DELETE_CODE_PATTERN.test(value);
}
