/**
 * 変異検査の対象から漏れた crate が無いことを確かめる。
 *
 * 対象は二箇所にある。手元用の `package.json` の `mutation:rust:mutest` と、
 * CI 用の workflow のシャード。どちらかに書き忘れると、その crate は誰にも
 * 検査されないまま緑になる。実行するより先に、ここで気付けるようにする。
 */
import { parse } from "yaml";

const script: string | undefined = (
  await Bun.file("package.json").json()
).scripts?.["mutation:rust:mutest"];
if (!script) {
  console.error("package.json に mutation:rust:mutest がありません");
  process.exit(1);
}
// `bun scripts/run-mutest.ts a b c` の crate 名だけを取り出す。
const declared = script.split(/\s+/).filter(Boolean).slice(2);

const workflow = parse(
  await Bun.file(".github/workflows/mutation.yml").text(),
) as {
  jobs: {
    rust: { strategy: { matrix: { include: { crates: string }[] } } };
  };
};
const shards = workflow.jobs.rust.strategy.matrix.include;
const sharded = shards.flatMap((shard) => shard.crates.split(/\s+/)).filter(Boolean);

const metadata = await new Response(
  Bun.spawn(["cargo", "metadata", "--no-deps", "--format-version", "1"]).stdout,
).json();
const crates: string[] = metadata.packages
  .map((crate: { name: string }) => crate.name)
  .sort();

const problems: string[] = [];
for (const [label, list] of [
  ["package.json", declared],
  ["workflow のシャード", sharded],
] as const) {
  const missing = crates.filter((name) => !list.includes(name));
  if (missing.length > 0) {
    problems.push(`${label} に入っていない crate: ${missing.join(", ")}`);
  }
  const unknown = list.filter((name) => !crates.includes(name));
  if (unknown.length > 0) {
    problems.push(`${label} が存在しない crate を指している: ${unknown.join(", ")}`);
  }
  const duplicated = list.filter((name, index) => list.indexOf(name) !== index);
  if (duplicated.length > 0) {
    problems.push(`${label} で重複している: ${[...new Set(duplicated)].join(", ")}`);
  }
}

if (problems.length > 0) {
  console.error("変異検査の対象に問題があります");
  for (const problem of problems) console.error(`  - ${problem}`);
  process.exit(1);
}

console.log(
  `変異検査は ${crates.length} crate を全て対象にしています` +
    `（CI は ${shards.length} シャード）`,
);
