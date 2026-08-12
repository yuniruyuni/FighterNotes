export default {
  testRunner: "command",
  commandRunner: {
    // Stryker は終了コードだけで生死を決めるので、変異を殺す test が 1 つ
    // 見つかれば残りは走らせなくてよい。
    command: "cd server && bun test --bail src/models",
  },
  coverageAnalysis: "off",
  disableTypeChecks: true,
  mutate: ["server/src/models/**/*.ts", "!server/src/**/*.test.ts"],
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
  htmlReporter: { fileName: "reports/mutation/server-models.html" },
  jsonReporter: { fileName: "reports/mutation/server-models.json" },
  thresholds: { high: 100, low: 100, break: null },
};
