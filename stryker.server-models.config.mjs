export default {
  testRunner: "command",
  commandRunner: {
    command: "cd server && bun test src/models",
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
  concurrency: 2,
  timeoutFactor: 4,
  timeoutMS: 10_000,
  reporters: ["clear-text", "html", "json"],
  htmlReporter: { fileName: "reports/mutation/server-models.html" },
  jsonReporter: { fileName: "reports/mutation/server-models.json" },
  thresholds: { high: 100, low: 100, break: null },
};
