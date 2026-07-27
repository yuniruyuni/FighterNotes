// Capcom 公式フレームデータページ（SSR HTML）のパーサ。
// コマンドは画像シーケンスで表現されているため numpad 記法へ変換する。
export type MoveCategory =
  | "normal"
  | "unique"
  | "special"
  | "super"
  | "throw"
  | "common";

export type MoveData = {
  name: string;
  startup: number;
  damage: number;
  category: MoveCategory;
};

// 公式ページのセクション見出し行 → カテゴリ。SA（super）はゲージ前提なので
// 確反の汎用提案から構造的に除外できるようデータに持たせる
const SECTION_MAP: Record<string, MoveCategory> = {
  "Normal Moves": "normal",
  "Unique Attacks": "unique",
  "Special Moves": "special",
  "Super Arts": "super",
  Throws: "throw",
  "Common Moves": "common",
};

// 画像ファイル名 → numpad 記法。key-nutral は公式サイト側の綴り
const TOKEN_MAP: Record<string, string> = {
  "key-d": "2",
  "key-dr": "3",
  "key-r": "6",
  "key-dl": "1",
  "key-l": "4",
  "key-nutral": "5",
  "key-ul": "7",
  "key-u": "8",
  "key-ur": "9",
  // タメ（charge）方向。numpad 記法の [4]6P 等に対応
  "key-dc": "[2]",
  "key-dlc": "[1]",
  "key-lc": "[4]",
  "key-rc": "[6]",
  "key-uc": "[8]",
  "key-circle": "360",
  "key-plus": "",
  "key-or": "/",
  arrow_3: "~",
  icon_punch_l: "LP",
  icon_punch_m: "MP",
  icon_punch_h: "HP",
  icon_punch: "P",
  icon_kick_l: "LK",
  icon_kick_m: "MK",
  icon_kick_h: "HK",
  icon_kick: "K",
};

const stripTags = (html: string) =>
  html
    .replace(/<[^>]+>/g, "")
    .normalize("NFKC")
    .replace(/\s+/g, " ")
    .trim();

// "800 (400x2)" や "1,000" から先頭の数値を取る。非数値（"-" 等）は 0
const leadingNumber = (html: string): number => {
  const m = stripTags(html).replace(/,/g, "").match(/^(\d+)/);
  return m ? Number(m[1]) : 0;
};

/**
 * 技名セルから numpad 記法の技名を作る。空中技は null（地上技のみ収録）。
 * コマンド画像がない技（Sun Veil 等の設置・共通行動）は英語技名を使う。
 */
export function moveName(cell: string): string | null {
  const arts = cell.match(/<span class="frame_arts[^"]*"[^>]*>(.*?)<\/span>/s);
  const artsName = arts ? stripTags(arts[1]) : stripTags(cell);
  const classic = cell.match(/<p class="frame_classic[^"]*"[^>]*>(.*?)<\/p>/s);
  if (!classic) return artsName;
  const body = classic[1];
  // 中立から出せない技（空中技・スタンス派生・追撃・ガード中限定）は
  // 確反提案の対象外。投げの近距離条件やリソース条件（飲酒レベル等）は残す
  const conditions = stripTags(body).match(/\([^)]*\)/g) ?? [];
  if (conditions.some((c) => /^\((During|After|While|while|When blocking|when coming)/.test(c))) {
    return null;
  }

  let name = "";
  const re = /<img[^>]*\/([a-z0-9_-]+)\.png[^>]*>|Hold/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(body)) !== null) {
    if (m[0] === "Hold") {
      name += "[H]";
      continue;
    }
    const mapped = TOKEN_MAP[m[1]];
    if (mapped === undefined) {
      throw new Error(`未知のコマンド画像: ${m[1]} (${artsName})`);
    }
    name += mapped;
  }
  // 回転2周（720 コマンド）は慣用表記に正規化
  name = name.replace("360360", "720");
  return name !== "" ? name : artsName;
}

/** フレームデータページ全体から地上技の一覧を抽出する。 */
export function parseCharacterPage(html: string): MoveData[] {
  const moves: MoveData[] = [];
  let category: MoveCategory | null = null;
  for (const row of html.matchAll(/<tr[^>]*>(.*?)<\/tr>/gs)) {
    const cells = [...row[1].matchAll(/<t[dh][^>]*>(.*?)<\/t[dh]>/gs)].map(
      (c) => c[1],
    );
    // セクション見出し行（セル数が少ない）でカテゴリを切り替える
    if (cells.length <= 2 && cells.length > 0) {
      const heading = stripTags(cells[0]);
      if (SECTION_MAP[heading] !== undefined) category = SECTION_MAP[heading];
      continue;
    }
    // データ行は [0]=技名 [1]=発生 [7]=ダメージ。発生が数値でない行は
    // 表ヘッダなのでスキップ
    if (cells.length < 8 || !/^\d/.test(stripTags(cells[1]))) continue;
    const name = moveName(cells[0]);
    if (name === null) continue;
    if (category === null) {
      // セクション見出しより先にデータ行が来た = ページ構造が変わった
      throw new Error(`セクション見出しが見つからないままデータ行が出現: ${name}`);
    }
    moves.push({
      name,
      startup: leadingNumber(cells[1]),
      damage: leadingNumber(cells[7]),
      category,
    });
  }
  return moves;
}
