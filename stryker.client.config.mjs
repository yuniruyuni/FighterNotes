const pureLogicTests = [
  "src/modules/analysis/domain",
  "src/modules/analysis/application",
  "src/modules/results/domain",
  "src/modules/results/application",
  "src/modules/sharing/domain",
  "src/modules/sharing/application",
  "src/modules/results/ui/debug/debug-frame-shortcuts.test.ts",
  "src/modules/results/ui/debug/debug-viewer-model.test.ts",
  "src/modules/results/ui/summary/damage-origin-format.test.ts",
].join(" ");

// 変異の対象を module ごとに分け、CI では shard として並列に回す。走らせる test は
// どの shard でも同じで、対象だけが変わる。module をまたいで殺している変異があっても
// 見落とさない。`MUTATION_SCOPE` が無ければ全 module を対象にする。
export const mutationScopes = {
  analysis: ["client/src/modules/analysis/{domain,application}/**/*.ts"],
  results: [
    "client/src/modules/results/{domain,application}/**/*.ts",
    "client/src/modules/results/ui/debug/debug-frame-shortcuts.ts",
    "client/src/modules/results/ui/debug/debug-viewer-model.ts",
    "client/src/modules/results/ui/summary/damage-origin-format.ts",
  ],
  sharing: ["client/src/modules/sharing/{domain,application}/**/*.ts"],
};

const scope = process.env.MUTATION_SCOPE;
if (scope && !(scope in mutationScopes)) {
  throw new Error(
    `MUTATION_SCOPE=${scope} は未定義。使えるのは ${Object.keys(mutationScopes).join(", ")}`,
  );
}
const target = scope
  ? mutationScopes[scope]
  : Object.values(mutationScopes).flat();
const reportName = scope ? `client-${scope}` : "client";

export default {
  testRunner: "command",
  commandRunner: {
    // Stryker は終了コードだけで生死を決めるので、変異を殺す test が 1 つ
    // 見つかれば残りは走らせなくてよい。
    command: `cd client && bun --config=bunfig.mutation.toml test --bail ${pureLogicTests}`,
  },
  coverageAnalysis: "off",
  disableTypeChecks: true,
  mutate: [...target, "!client/src/**/*.test.ts"],
  ignorePatterns: [
    "/client/static/**",
    "/crates/wasm-bridge/pkg/**",
    "/target/**",
    "/output/**",
    "/video/**",
  ],
  // runner は 4 vCPU。既定（cpus/2）だと半分遊ぶ。ここのテストは
  // domain / application の純粋ロジックで共有資源を持たないため、
  // 走らせるだけ並列にできる。
  concurrency: 4,
  timeoutFactor: 4,
  timeoutMS: 10_000,
  reporters: ["clear-text", "html", "json"],
  htmlReporter: { fileName: `reports/mutation/${reportName}.html` },
  jsonReporter: { fileName: `reports/mutation/${reportName}.json` },
  thresholds: { high: 100, low: 100, break: null },
};
