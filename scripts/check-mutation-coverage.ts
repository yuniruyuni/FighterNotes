/**
 * 変異検査の対象から漏れた crate が無いことを確かめる。
 *
 * cargo-mutants は package 名を書き間違えても 1 件も検査せずに成功する。
 * ワークフロー側でも検査件数 0 を弾いてはいるが、crate を足したときに
 * どのジョブにも入っていない状態には気付けない。ここで塞ぐ。
 */
const WORKFLOW = ".github/workflows/mutation.yml";

const workflow = await Bun.file(WORKFLOW).text();
// 全域を回すのは rust-clean 以降の二つのジョブだけ。変更行だけを見る
// rust-changed は --workspace なので対象の宣言を持たない。
const fullRunSection = workflow.slice(workflow.indexOf("\n  rust-clean:"));
if (fullRunSection.length === 0) {
  throw new Error(`${WORKFLOW} に rust-clean ジョブが見つかりません`);
}

const declaredIn = new Map<string, string[]>();
let label = "rust-clean";
for (const line of fullRunSection.split("\n")) {
  const named = line.match(/^\s*- name:\s*(\S+)/);
  if (named) label = named[1];
  for (const [, name] of line.matchAll(/--package\s+(\S+)/g)) {
    declaredIn.set(name, [...(declaredIn.get(name) ?? []), label]);
  }
}

const metadata = await new Response(
  Bun.spawn(["cargo", "metadata", "--no-deps", "--format-version", "1"]).stdout,
).json();
const crates: string[] = metadata.packages
  .map((crate: { name: string }) => crate.name)
  .sort();

const problems: string[] = [];
const missing = crates.filter((name) => !declaredIn.has(name));
if (missing.length > 0) {
  problems.push(`どのジョブにも入っていない crate: ${missing.join(", ")}`);
}
const unknown = [...declaredIn.keys()].filter((name) => !crates.includes(name));
if (unknown.length > 0) {
  problems.push(`存在しない crate を指している: ${unknown.join(", ")}`);
}
if (problems.length > 0) {
  console.error(`${WORKFLOW} の変異検査対象に問題があります`);
  for (const problem of problems) console.error(`  - ${problem}`);
  process.exit(1);
}

console.log(`変異検査は ${crates.length} crate を全て対象にしています`);
