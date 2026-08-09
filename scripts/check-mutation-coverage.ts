/**
 * 変異検査の対象から漏れた crate が無いことを確かめる。
 *
 * 対象は package.json の `mutation:rust:mutest` が正本。crate を足したときに
 * そこへ書き忘れると、その crate は誰にも検査されないまま CI が緑になる。
 * 実行するより先に、ここで気付けるようにする。
 */
const script: string | undefined = (
  await Bun.file("package.json").json()
).scripts?.["mutation:rust:mutest"];
if (!script) {
  console.error("package.json に mutation:rust:mutest がありません");
  process.exit(1);
}

// `bun scripts/run-mutest.ts a b c` の crate 名だけを取り出す。
const declared = script
  .split(/\s+/)
  .filter((token) => token.length > 0)
  .slice(2);

const metadata = await new Response(
  Bun.spawn(["cargo", "metadata", "--no-deps", "--format-version", "1"]).stdout,
).json();
const crates: string[] = metadata.packages
  .map((crate: { name: string }) => crate.name)
  .sort();

const problems: string[] = [];
const missing = crates.filter((name) => !declared.includes(name));
if (missing.length > 0) {
  problems.push(`対象に入っていない crate: ${missing.join(", ")}`);
}
const unknown = declared.filter((name) => !crates.includes(name));
if (unknown.length > 0) {
  problems.push(`存在しない crate を指している: ${unknown.join(", ")}`);
}
const duplicated = declared.filter(
  (name, index) => declared.indexOf(name) !== index,
);
if (duplicated.length > 0) {
  problems.push(`重複して指定されている: ${[...new Set(duplicated)].join(", ")}`);
}

if (problems.length > 0) {
  console.error("mutation:rust:mutest の対象に問題があります");
  for (const problem of problems) console.error(`  - ${problem}`);
  process.exit(1);
}

console.log(`変異検査は ${crates.length} crate を全て対象にしています`);
