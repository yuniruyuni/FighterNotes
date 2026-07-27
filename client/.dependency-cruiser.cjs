/** @type {import('dependency-cruiser').IConfiguration} */
module.exports = {
  forbidden: [
    {
      name: "no-circular",
      comment: "Production modules must remain acyclic.",
      severity: "error",
      from: { pathNot: "\\.test\\.[jt]sx?$" },
      to: { circular: true },
    },
    {
      name: "shared-is-independent",
      comment: "Shared code cannot depend on composition or feature layers.",
      severity: "error",
      from: { path: "^src/shared/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/(app|entrypoints|modules|pages)/" },
    },
    {
      name: "modules-do-not-own-composition",
      comment:
        "Feature modules cannot depend on app, page, or entrypoint code.",
      severity: "error",
      from: { path: "^src/modules/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/(app|entrypoints|pages)/" },
    },
    {
      name: "domain-is-pure",
      comment:
        "Domain code cannot depend on application, infrastructure, or UI.",
      severity: "error",
      from: {
        path: "^src/modules/[^/]+/domain/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: { path: "^src/modules/[^/]+/(application|infrastructure|ui)/" },
    },
    {
      name: "application-does-not-use-adapters",
      comment:
        "Application services depend on ports and domain code, not adapters or UI.",
      severity: "error",
      from: {
        path: "^src/modules/[^/]+/application/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: { path: "^src/modules/[^/]+/(infrastructure|ui)/" },
    },
    {
      name: "analysis-infrastructure-leaves-do-not-use-pipeline",
      comment:
        "Analysis pipeline coordinates leaf adapters; leaf adapters cannot depend back on it.",
      severity: "error",
      from: {
        path: "^src/modules/analysis/infrastructure/(diagnostics|frame-extraction|spatial-analysis|video-decoding|wasm-bridge|worker-bridge)/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: { path: "^src/modules/analysis/infrastructure/pipeline/" },
    },
    {
      name: "ui-does-not-use-adapters",
      comment: "React UI receives adapters through module service providers.",
      severity: "error",
      from: { path: "^src/modules/[^/]+/ui/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/modules/[^/]+/infrastructure/" },
    },
    {
      name: "infrastructure-does-not-use-ui",
      comment:
        "Infrastructure implements application ports and cannot depend on presentation code.",
      severity: "error",
      from: {
        path: "^src/modules/[^/]+/infrastructure/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: { path: "^src/modules/[^/]+/ui/" },
    },
    {
      name: "analysis-is-foundational",
      comment: "Analysis cannot depend on results or sharing.",
      severity: "error",
      from: { path: "^src/modules/analysis/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/modules/(results|sharing)/" },
    },
    {
      name: "results-and-sharing-are-independent",
      comment: "Results and sharing are composed by pages, not by each other.",
      severity: "error",
      from: { path: "^src/modules/results/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/modules/sharing/" },
    },
    {
      name: "sharing-does-not-use-results",
      comment: "Sharing only consumes stable analysis contracts.",
      severity: "error",
      from: { path: "^src/modules/sharing/", pathNot: "\\.test\\.[jt]sx?$" },
      to: { path: "^src/modules/results/" },
    },
    {
      name: "analysis-internals-are-private",
      comment:
        "Other modules must use analysis contracts or browser entrypoints.",
      severity: "error",
      from: {
        path: "^src/modules/(results|sharing)/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: {
        path: "^src/modules/analysis/",
        pathNot: "^src/modules/analysis/(browser|contracts)\\.ts$",
      },
    },
    {
      name: "composition-uses-module-entrypoints",
      comment:
        "App, pages, and entrypoints cannot reach into module internals.",
      severity: "error",
      from: {
        path: "^src/(app|entrypoints|pages)/",
        pathNot: "\\.test\\.[jt]sx?$",
      },
      to: {
        path: "^src/modules/[^/]+/",
        pathNot: "^src/modules/[^/]+/(browser|contracts|index|worker)\\.ts$",
      },
    },
  ],
  options: {
    doNotFollow: { path: "node_modules" },
    tsConfig: { fileName: "tsconfig.json" },
    tsPreCompilationDeps: true,
  },
};
