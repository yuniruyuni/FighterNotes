// Stryker は終了コードだけで生死を決めるので、変異を殺す test が 1 つ見つかれば
// 残りは走らせなくてよい。DB を使わない test を先に置くと、大半の変異は重い
// integration へ到達する前に死ぬ。生き残る変異が走らせる test の集合は変わらない。
const repositoryTests = [
  'bun test --bail $(find src/repositories -name "*.test.ts" ! -name "*.integration.test.ts")',
  'bun test --bail $(find src/repositories -name "*.integration.test.ts")',
];

/** @type {import('@stryker-mutator/api/core').PartialStrykerOptions} */
const config = {
  testRunner: "command",
  commandRunner: {
    command: [
      'test -n "$TEST_DATABASE_URL"',
      "cd server",
      ...repositoryTests,
    ].join(" && "),
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
