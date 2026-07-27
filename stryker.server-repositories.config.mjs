/** @type {import('@stryker-mutator/api/core').PartialStrykerOptions} */
const config = {
  testRunner: "command",
  commandRunner: {
    command:
      'test -n "$TEST_DATABASE_URL" && cd server && bun test src/repositories',
  },
  mutate: [
    "server/src/repositories/**/postgres/*.ts",
    "!server/src/repositories/**/postgres/index.ts",
    "!server/src/repositories/**/postgres/integration-suite.ts",
    "!server/src/repositories/**/*.test.ts",
  ],
  coverageAnalysis: "off",
  disableTypeChecks: true,
  ignorePatterns: [
    ".git",
    ".stryker-tmp",
    "client/static",
    "crates/wasm-bridge/pkg",
    "node_modules",
    "reports",
    "server/dist",
    "target",
  ],
  concurrency: 1,
  timeoutFactor: 4,
  timeoutMS: 20_000,
  reporters: ["clear-text", "html", "json"],
  htmlReporter: { fileName: "reports/mutation/server-repositories.html" },
  jsonReporter: { fileName: "reports/mutation/server-repositories.json" },
  thresholds: { high: 100, low: 100, break: null },
};

export default config;
