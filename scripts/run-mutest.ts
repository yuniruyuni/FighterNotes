/**
 * mutest-rs を走らせ、結果を判定する。
 *
 * mutest はテストからの呼び出しグラフを起点に変異を作るので、テストが
 * 一つも無い crate では変異が生成されず `mutations: none` と出て成功して
 * しまう。守られていない状態や未検出の変異が緑に見えるのは、門番として
 * 最悪の失敗様式なので、変異が 0 個、未検出が 1 個以上、時間切れまたは
 * ハーネス異常終了が 1 個以上なら失敗として扱う。
 *
 * 使い方: bun scripts/run-mutest.ts <crate> [<crate>...]
 */
const CRATES = process.argv.slice(2);
if (CRATES.length === 0) {
  console.error("使い方: bun scripts/run-mutest.ts <crate> [<crate>...]");
  process.exit(2);
}

type Result = {
  crate: string;
  total: number;
  detected: number;
  undetected: number;
  timedOut: number;
  crashed: number;
  seconds: number;
  failure?: string;
};

// test-support は一部の crate だけが持つ。持たない crate へ渡すと
// cargo が拒否するので、宣言している crate にだけ付ける。
const metadata = await new Response(
  Bun.spawn(["cargo", "metadata", "--no-deps", "--format-version", "1"]).stdout,
).json();
const hasTestSupport = new Set<string>(
  metadata.packages
    .filter((p: { features: Record<string, unknown> }) => "test-support" in p.features)
    .map((p: { name: string }) => p.name),
);

async function runOne(crate: string): Promise<Result> {
  const started = Date.now();
  const proc = Bun.spawn(
    [
      "cargo",
      "mutest",
      "run",
      "-p",
      crate,
      // 統合テストは別バイナリとして本体を link するため、mutest が
      // 本体側の変異ハーネスを見つけられない。crate 内のテストだけを使う。
      "--lib",
      // 変異によっては巻き戻せないパニックで abort する。隔離しないと
      // そこで走査全体が止まる。
      "--isolate=all",
      // span が実ファイルに対応しない変異（マクロ由来）があると、
      // メタデータ書き出しで mutest-driver 自身が落ちる。判定には
      // 使わない出力なので止める。
      "--no-emit-metadata",
      // 変異の評価を並列に回す。結果は変わらず、実測で 2 倍以上速い。
      "--parallel-mutants",
      ...(hasTestSupport.has(crate) ? ["--features", "test-support"] : []),
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  const [out, err] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
  ]);
  await proc.exited;
  const seconds = Math.round((Date.now() - started) / 1000);
  const text = `${out}\n${err}`;

  // 最後の集計行だけを見る。crate が複数のテスト対象を持つと複数出る。
  const summaries = [
    ...text.matchAll(
      /^mutations: .*?\. (\d+) detected \((\d+) timed out; (\d+) crashed\); (\d+) undetected; (\d+) total/gm,
    ),
  ];
  if (summaries.length === 0) {
    const compileError = text.match(/^error(\[E\d+\])?: .*/m);
    return {
      crate,
      total: 0,
      detected: 0,
      undetected: 0,
      timedOut: 0,
      crashed: 0,
      seconds,
      failure: compileError ? compileError[0] : "集計行が見つからない",
    };
  }
  const totals = summaries.reduce(
    (acc, m) => ({
      detected: acc.detected + Number(m[1]),
      timedOut: acc.timedOut + Number(m[2]),
      crashed: acc.crashed + Number(m[3]),
      undetected: acc.undetected + Number(m[4]),
      total: acc.total + Number(m[5]),
    }),
    { detected: 0, timedOut: 0, crashed: 0, undetected: 0, total: 0 },
  );
  return { crate, seconds, ...totals };
}

const results: Result[] = [];
for (const crate of CRATES) results.push(await runOne(crate));

const width = Math.max(...results.map((r) => r.crate.length));
const problems: string[] = [];
for (const r of results) {
  const head = `${r.crate.padEnd(width)}  ${String(r.seconds).padStart(3)}s`;
  if (r.failure) {
    console.log(`${head}  失敗: ${r.failure}`);
    problems.push(`${r.crate}: ${r.failure}`);
    continue;
  }
  // 変異が生成されないのは「守られている」ではなく「テストが無い」。
  if (r.total === 0) {
    console.log(`${head}  変異が 1 つも生成されなかった`);
    problems.push(`${r.crate}: テストからたどれるコードが無い`);
    continue;
  }
  const score = ((r.detected / r.total) * 100).toFixed(1);
  console.log(
    `${head}  ${String(r.total).padStart(5)} 変異  検出 ${score}%  ` +
      `未検出 ${r.undetected}${r.timedOut > 0 ? ` / 時間切れ ${r.timedOut}` : ""}` +
      `${r.crashed > 0 ? ` / 異常終了 ${r.crashed}` : ""}`,
  );
  if (r.undetected > 0) {
    problems.push(`${r.crate}: ${r.undetected} 変異が未検出`);
  }
  if (r.timedOut > 0) {
    problems.push(`${r.crate}: ${r.timedOut} 変異が時間切れ`);
  }
  if (r.crashed > 0) {
    problems.push(`${r.crate}: ${r.crashed} 変異でハーネスが異常終了`);
  }
}

const total = results.reduce((n, r) => n + r.total, 0);
const detected = results.reduce((n, r) => n + r.detected, 0);
console.log(
  `\n合計 ${total} 変異 / 検出 ${detected} ` +
    `(${total > 0 ? ((detected / total) * 100).toFixed(1) : "0.0"}%) / ` +
    `${results.reduce((n, r) => n + r.seconds, 0)} 秒`,
);

if (problems.length > 0) {
  console.error("\n変異検査に問題がある crate があります:");
  for (const p of problems) console.error(`  - ${p}`);
  process.exit(1);
}
