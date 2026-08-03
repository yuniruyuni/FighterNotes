// 公式サイトから全キャラのフレームデータを取得し、確反用 frame_data.json と
// 打撃属性照合用 attack_data.json および manifest.json を再生成する。
// 実行: bun run scripts/gen-frame-data.ts YYYY-MM-DD.N （リポジトリルートで）

import { createHash } from "node:crypto";
import type { AttackMoveData } from "./frame-bundle-parser";
import {
  buildAttackMoves,
  frameBundlePath,
  parseOfficialFrameBundle,
} from "./frame-bundle-parser";
import type { MoveData } from "./frame-page-parser";
import { frameCharacterSlugs, parseCharacterPage } from "./frame-page-parser";

// key は frame_data.json のキャラ名キー（既存キーを維持、新キャラは同じ流儀）
const CHARACTERS: Array<{ slug: string; key: string }> = [
  { slug: "aki", key: "A_K_I" },
  { slug: "gouki_akuma", key: "AKUMA" },
  { slug: "alex", key: "ALEX" },
  { slug: "blanka", key: "BLANKA" },
  { slug: "cammy", key: "CAMMY" },
  { slug: "chunli", key: "CHUN_LI" },
  { slug: "cviper", key: "C_VIPER" },
  { slug: "deejay", key: "DEE_JAY" },
  { slug: "dhalsim", key: "DHALSIM" },
  { slug: "ed", key: "ED" },
  { slug: "ehonda", key: "E_HONDA" },
  { slug: "elena", key: "ELENA" },
  { slug: "guile", key: "GUILE" },
  { slug: "ingrid", key: "INGRID" },
  { slug: "jamie", key: "JAMIE" },
  { slug: "jp", key: "JP" },
  { slug: "juri", key: "JURI" },
  { slug: "ken", key: "KEN" },
  { slug: "kimberly", key: "KIMBERLY" },
  { slug: "lily", key: "LILY" },
  { slug: "luke", key: "LUKE" },
  { slug: "mai", key: "MAI" },
  { slug: "manon", key: "MANON" },
  { slug: "marisa", key: "MARISA" },
  { slug: "vega_mbison", key: "M_BISON" },
  { slug: "rashid", key: "RASHID" },
  { slug: "ryu", key: "RYU" },
  { slug: "sagat", key: "SAGAT" },
  { slug: "terry", key: "TERRY" },
  { slug: "yasmine", key: "YASMINE" },
  { slug: "zangief", key: "ZANGIEF" },
];

const dataVersion = process.argv[2];
if (!dataVersion || !/^\d{4}-\d{2}-\d{2}\.\d+$/.test(dataVersion)) {
  throw new Error(
    "data versionをYYYY-MM-DD.N形式で指定してください: " +
      "bun run scripts/gen-frame-data.ts 2026-08-03.1",
  );
}

// 通常のブラウザから閲覧した場合と同じ要求を再現し、ブラウザ向けの正しい
// レスポンスを受け取るために、一般的なブラウザ相当のUser-Agentを明示する。
// 認証やアクセス制御などの技術的保護を回避する処理ではない。
const BROWSER_USER_AGENT =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const fetchOfficial = async (url: string) => {
  const response = await fetch(url, {
    headers: { "User-Agent": BROWSER_USER_AGENT },
  });
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return response.text();
};

// SSR HTML は Classic タブだけなので、Modern 入力と攻撃属性は全キャラ分を
// 内包するフレーム表 chunk から取得する。chunk 名のハッシュは固定しない。
const indexUrl = "https://www.streetfighter.com/6/character/ingrid/frame";
const indexHtml = await fetchOfficial(indexUrl);
const configuredSlugs = CHARACTERS.map(({ slug }) => slug).sort();
const officialSlugs = frameCharacterSlugs(indexHtml);
if (JSON.stringify(configuredSlugs) !== JSON.stringify(officialSlugs)) {
  const configured = new Set(configuredSlugs);
  const official = new Set(officialSlugs);
  const missing = officialSlugs.filter((slug) => !configured.has(slug));
  const removed = configuredSlugs.filter((slug) => !official.has(slug));
  throw new Error(
    `公式character一覧とgenerator設定が一致しません (未設定: ${missing.join(", ") || "なし"}, 公式に存在しない設定: ${removed.join(", ") || "なし"})`,
  );
}
const bundleUrl = new URL(frameBundlePath(indexHtml), indexUrl).toString();
const officialRows = parseOfficialFrameBundle(await fetchOfficial(bundleUrl));

const result: Record<string, MoveData[]> = {};
const attackResult: Record<string, AttackMoveData[]> = {};
for (const { slug, key } of CHARACTERS) {
  const url = `https://www.streetfighter.com/6/character/${slug}/frame`;
  const html = slug === "ingrid" ? indexHtml : await fetchOfficial(url);
  const moves = parseCharacterPage(html);
  // パース失敗や構造変化の検知。派生技除外後も全キャラ 35 技以上あるはず
  // （最少は A.K.I. の 39。2026-07 時点）
  if (moves.length < 35) {
    throw new Error(`${key}: 技数が少なすぎる (${moves.length})`);
  }
  result[key] = moves;
  const attackMoves = buildAttackMoves(officialRows[slug] ?? []);
  if (attackMoves.length < 35) {
    throw new Error(
      `${key}: 入力照合可能な攻撃が少なすぎる (${attackMoves.length})`,
    );
  }
  attackResult[key] = attackMoves;
  console.log(`${key}: punish ${moves.length} / attack ${attackMoves.length}`);
  await sleep(1000);
}

const frameData = `${JSON.stringify(result, null, 1)}\n`;
const attackData = `${JSON.stringify(attackResult, null, 1)}\n`;
const sha256 = (value: string) =>
  createHash("sha256").update(value).digest("hex");
const moveCount = (value: Record<string, unknown[]>) =>
  Object.values(value).reduce((total, moves) => total + moves.length, 0);
const manifest = `${JSON.stringify(
  {
    schema_version: 1,
    data_version: dataVersion,
    files: {
      "frame_data.json": {
        sha256: sha256(frameData),
        character_count: Object.keys(result).length,
        move_count: moveCount(result),
      },
      "attack_data.json": {
        sha256: sha256(attackData),
        character_count: Object.keys(attackResult).length,
        move_count: moveCount(attackResult),
      },
    },
  },
  null,
  1,
)}\n`;

await Bun.write("crates/video-analyzer/data/frame_data.json", frameData);
console.log("wrote crates/video-analyzer/data/frame_data.json");
await Bun.write("crates/video-analyzer/data/attack_data.json", attackData);
console.log("wrote crates/video-analyzer/data/attack_data.json");
await Bun.write("crates/video-analyzer/data/manifest.json", manifest);
console.log("wrote crates/video-analyzer/data/manifest.json");
