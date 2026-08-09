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

export default {
  testRunner: "command",
  commandRunner: {
    command: `cd client && bun --config=bunfig.mutation.toml test ${pureLogicTests}`,
  },
  coverageAnalysis: "off",
  disableTypeChecks: true,
  mutate: [
    "client/src/modules/{analysis,results,sharing}/{domain,application}/**/*.ts",
    "!client/src/**/*.test.ts",
    "client/src/modules/results/ui/debug/debug-frame-shortcuts.ts",
    "client/src/modules/results/ui/debug/debug-viewer-model.ts",
    "client/src/modules/results/ui/summary/damage-origin-format.ts",
  ],
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
  htmlReporter: { fileName: "reports/mutation/client.html" },
  jsonReporter: { fileName: "reports/mutation/client.json" },
  thresholds: { high: 100, low: 100, break: null },
};
