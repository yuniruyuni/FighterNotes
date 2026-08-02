import { existsSync } from "node:fs";
import { mkdir, mkdtemp, readFile, rm } from "node:fs/promises";
import { homedir, tmpdir } from "node:os";
import {
  basename,
  extname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from "node:path";
import {
  type BaselineCaseArtifact,
  CAPTURE_HASH_FIELDS,
  type CaptureHashField,
  canonicalJson,
  compareArtifactIdentity,
  compareFixtureIdSets,
  computeArtifactIdentity,
  type FixtureSettings,
  parseBaselineArtifact,
  REQUIRED_PERFORMANCE_STAGES,
  RUNNER_VERSION,
} from "./local-video-e2e-baseline";
import {
  compareDetectorMetrics,
  comparePerformance,
  type DetectorId,
  type DetectorMetrics,
  diffSemanticValues,
  evaluateExpectations,
  evaluateRegressionEvents,
  type LocalVideoCase,
  type LocalVideoPerformancePolicy,
  parseLocalVideoManifest,
  semanticSnapshot,
  summarizeTimings,
  type TimingSummary,
} from "./local-video-e2e-lib";

const DEFAULT_MANIFEST = "video/local-video-e2e.json";
const DEFAULT_OUTPUT = "output/local-video-e2e/current";
const DEFAULT_TIMEOUT_SECONDS = 600;
const DETECTOR_IDS: readonly DetectorId[] = [
  "round",
  "fight",
  "damage",
  "super",
  "attackInfo",
  "attackInfoAttribution",
  "adviceEvidence",
];

interface CliOptions {
  readonly manifestPath: string;
  readonly outputDir: string;
  readonly baselineDir?: string;
  readonly cdpUrl?: string;
  readonly browserExecutable?: string;
  readonly headed: boolean;
  readonly measuredRuns?: number;
  readonly warmupRuns?: number;
}

interface BrowserHandle {
  readonly cdpUrl: string;
  close(): Promise<void>;
}

interface CdpTarget {
  readonly id: string;
  readonly webSocketDebuggerUrl: string;
}

interface CapturedWorkerArtifacts {
  readonly report: string;
  readonly timeline: string;
  readonly features: string;
  readonly trackedInputs?: string;
  readonly fightMarkers?: string;
  readonly attackInfo?: string;
  readonly regressionEvents: string;
  readonly spatialWindows?: string;
  readonly spatialObservations?: string;
  readonly perfLogs?: string[];
  readonly stageTimings?: Readonly<Record<string, number>>;
  readonly analysisMs: number;
}

interface CaseSummary {
  readonly id: string;
  readonly videoName: string;
  readonly fixtureFingerprint: string;
  readonly settings: FixtureSettings;
  readonly expectationHash: string;
  readonly analysisMs: number;
  readonly performance: TimingSummary;
  readonly detectorMetrics: Partial<
    Readonly<Record<DetectorId, DetectorMetrics>>
  >;
  readonly syntheticCoverage: {
    readonly ported: number;
    readonly pending: number;
    readonly pendingIds: readonly string[];
  };
  readonly assertionsPassed: boolean;
  readonly assertionFailures: readonly string[];
  readonly hashes: Readonly<Record<CaptureHashField, string>>;
  readonly semanticHash: string;
}

interface RunSummary {
  readonly schemaVersion: 2;
  readonly runnerVersion: number;
  readonly warmupRuns: number;
  readonly measuredRuns: number;
  readonly generatedAt: string;
  readonly cases: readonly CaseSummary[];
}

interface BaselineData {
  readonly directory: string;
  readonly summary: RunSummary;
}

const options = parseArgs(Bun.argv.slice(2));
const projectRoot = resolve(import.meta.dir, "..");
const manifestPath = resolve(projectRoot, options.manifestPath);
const outputDir = resolve(projectRoot, options.outputDir);
const staticRoot = resolve(projectRoot, "client/static");

if (!existsSync(join(staticRoot, "index.html"))) {
  throw new Error(
    "client/static is missing. Run `bun run build` before the local E2E.",
  );
}

const manifest = parseLocalVideoManifest(
  JSON.parse(await readFile(manifestPath, "utf8")),
);
const performancePolicy = manifest.performance ?? {};
const measuredRuns =
  options.measuredRuns ??
  performancePolicy.measuredRuns ??
  (options.baselineDir ? 3 : 1);
const warmupRuns =
  options.warmupRuns ??
  performancePolicy.warmupRuns ??
  (options.baselineDir ? 1 : 0);
if (options.baselineDir && measuredRuns < 3) {
  throw new Error(
    "baseline performance comparison requires at least 3 measured runs",
  );
}
if (options.baselineDir && warmupRuns < 1) {
  throw new Error("baseline comparison requires at least 1 warm-up run");
}
const baseline = await loadBaseline(
  options.baselineDir,
  projectRoot,
  outputDir,
);
if (
  baseline &&
  (baseline.summary.measuredRuns !== measuredRuns ||
    baseline.summary.warmupRuns !== warmupRuns)
) {
  throw new Error(
    `baseline measurement contract is ${baseline.summary.warmupRuns} warm-up + ${baseline.summary.measuredRuns} measured run(s), current run requested ${warmupRuns} + ${measuredRuns}`,
  );
}
if (baseline) {
  const fixtureIdentityFailures = compareFixtureIdSets(
    manifest.cases.map((fixture) => fixture.id),
    baseline.summary.cases.map((fixture) => fixture.id),
  );
  if (fixtureIdentityFailures.length > 0) {
    throw new Error(
      `baseline fixture identity mismatch:\n${fixtureIdentityFailures
        .map((failure) => `- ${failure}`)
        .join("\n")}`,
    );
  }
}
for (const fixture of manifest.cases) {
  if (!isAbsolute(fixture.videoPath) || !existsSync(fixture.videoPath)) {
    throw new Error(
      `${fixture.id}: videoPath must be an existing absolute local path`,
    );
  }
  if (baseline && !hasAccuracyContract(fixture)) {
    throw new Error(
      `${fixture.id}: baseline comparison requires semanticEvents and at least one detectorGate`,
    );
  }
}

await mkdir(outputDir, { recursive: true });
const staticServer = startStaticServer(staticRoot);
const browser = await openBrowser(options);
const summaries: CaseSummary[] = [];
let failed = false;

try {
  const appUrl = `http://127.0.0.1:${staticServer.port}/`;
  for (const fixture of manifest.cases) {
    const fixtureFingerprint = await sha256File(fixture.videoPath);
    const settings: FixtureSettings = {
      side: fixture.side,
      ownCharacter: fixture.ownCharacter,
      opponentCharacter: fixture.opponentCharacter,
    };
    const expectationHash = sha256(canonicalJson(fixture.expect ?? null));
    console.log(
      `[local-e2e] ${fixture.id}: ${warmupRuns} warm-up + ${measuredRuns} measured run(s)`,
    );
    for (let run = 0; run < warmupRuns; run += 1) {
      console.log(
        `[local-e2e] ${fixture.id}: warm-up ${run + 1}/${warmupRuns}`,
      );
      await analyzeCase(browser.cdpUrl, appUrl, fixture);
    }
    let captured: CapturedWorkerArtifacts | undefined;
    let semanticDigest: string | undefined;
    let semanticChangedBetweenRuns = false;
    const timingRuns: Array<{
      readonly analysisMs: number;
      readonly stages: Readonly<Record<string, number>>;
    }> = [];
    for (let run = 0; run < measuredRuns; run += 1) {
      console.log(
        `[local-e2e] ${fixture.id}: measured ${run + 1}/${measuredRuns}`,
      );
      const measured = await analyzeCase(browser.cdpUrl, appUrl, fixture);
      const digest = semanticCapturedDigest(measured);
      if (semanticDigest === undefined) semanticDigest = digest;
      else if (semanticDigest !== digest) semanticChangedBetweenRuns = true;
      captured ??= measured;
      timingRuns.push({
        analysisMs: measured.analysisMs,
        stages: requiredStageTimings(measured, fixture.id),
      });
    }
    if (!captured)
      throw new Error(`${fixture.id}: no measured run was captured`);
    const report = parseArtifact(captured.report, "report", fixture.id);
    const regressionEvents = parseArtifact(
      captured.regressionEvents,
      "regressionEvents",
      fixture.id,
    );
    const performance = summarizeTimings(timingRuns);
    const artifacts: Record<string, unknown> = {
      schemaVersion: 2,
      runnerVersion: RUNNER_VERSION,
      caseId: fixture.id,
      videoName: basename(fixture.videoPath),
      fixtureContract: {
        fixtureFingerprint,
        settings,
        expectationHash,
      },
      analysisMs: performance.medianMs,
      performance,
      report,
      timeline: parseArtifact(captured.timeline, "timeline", fixture.id),
      hpFeatures: parseArtifact(captured.features, "features", fixture.id),
      trackedInputs: parseOptionalArtifact(
        captured.trackedInputs,
        "trackedInputs",
        fixture.id,
      ),
      fightMarkers: parseOptionalArtifact(
        captured.fightMarkers,
        "fightMarkers",
        fixture.id,
      ),
      attackInfo: parseOptionalArtifact(
        captured.attackInfo,
        "attackInfo",
        fixture.id,
      ),
      regressionEvents,
      spatialWindows: parseOptionalArtifact(
        captured.spatialWindows,
        "spatialWindows",
        fixture.id,
      ),
      spatialObservations: parseOptionalArtifact(
        captured.spatialObservations,
        "spatialObservations",
        fixture.id,
      ),
      perfLogs: captured.perfLogs ?? [],
    };
    const artifactText = JSON.stringify(artifacts, null, 2);
    await Bun.write(join(outputDir, `${fixture.id}.json`), `${artifactText}\n`);

    const regression = evaluateRegressionEvents(
      {
        report,
        fightMarkers: artifacts.fightMarkers,
        regressionEvents,
      },
      fixture.expect,
    );
    const assertionFailures = [
      ...evaluateExpectations(report, fixture.expect),
      ...regression.failures,
      ...(semanticChangedBetweenRuns
        ? [`${fixture.id}: semantic artifacts changed between measured runs`]
        : []),
      ...(baseline
        ? await compareWithBaseline(
            baseline,
            fixture,
            artifacts,
            performance,
            regression.metrics,
            performancePolicy,
            fixtureFingerprint,
            settings,
            expectationHash,
          )
        : []),
    ];
    failed ||= assertionFailures.length > 0;
    const identity = computeArtifactIdentity(artifacts);
    summaries.push({
      id: fixture.id,
      videoName: basename(fixture.videoPath),
      fixtureFingerprint,
      settings,
      expectationHash,
      analysisMs: performance.medianMs,
      performance,
      detectorMetrics: regression.metrics,
      syntheticCoverage: regression.syntheticCoverage,
      assertionsPassed: assertionFailures.length === 0,
      assertionFailures,
      hashes: identity.hashes,
      semanticHash: identity.semanticHash,
    });
    console.log(
      `[local-e2e] ${fixture.id}: median ${(
        performance.medianMs / 1_000
      ).toFixed(2)}s, p90 ${(performance.p90Ms / 1_000).toFixed(2)}s, ${
        assertionFailures.length === 0
          ? "expectations passed"
          : `${assertionFailures.length} expectation(s) failed`
      }`,
    );
    for (const failure of assertionFailures) {
      console.error(`  - ${failure}`);
    }
  }
} finally {
  staticServer.stop(true);
  await browser.close();
}

const summary: RunSummary = {
  schemaVersion: 2,
  runnerVersion: RUNNER_VERSION,
  warmupRuns,
  measuredRuns,
  generatedAt: new Date().toISOString(),
  cases: summaries,
};
await Bun.write(
  join(outputDir, "summary.json"),
  `${JSON.stringify(summary, null, 2)}\n`,
);
console.log(`[local-e2e] artifacts: ${relative(projectRoot, outputDir)}`);
if (failed) process.exitCode = 1;

function parseArgs(args: readonly string[]): CliOptions {
  let manifestPath = DEFAULT_MANIFEST;
  let outputDir = DEFAULT_OUTPUT;
  let baselineDir: string | undefined;
  let cdpUrl: string | undefined;
  let browserExecutable: string | undefined;
  let measuredRuns: number | undefined;
  let warmupRuns: number | undefined;
  let headed = false;

  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--headed") {
      headed = true;
      continue;
    }
    const value = args[index + 1];
    if (!value) throw new Error(`${argument} requires a value`);
    switch (argument) {
      case "--manifest":
        manifestPath = value;
        break;
      case "--output":
        outputDir = value;
        break;
      case "--baseline":
        baselineDir = value;
        break;
      case "--cdp":
        cdpUrl = value.replace(/\/$/, "");
        break;
      case "--browser":
        browserExecutable = value;
        break;
      case "--runs":
        measuredRuns = positiveInteger(value, "--runs", 1);
        break;
      case "--warmup-runs":
        warmupRuns = positiveInteger(value, "--warmup-runs", 0);
        break;
      default:
        throw new Error(`unknown option: ${argument}`);
    }
    index += 1;
  }
  return {
    manifestPath,
    outputDir,
    ...(baselineDir ? { baselineDir } : {}),
    ...(cdpUrl ? { cdpUrl } : {}),
    ...(browserExecutable ? { browserExecutable } : {}),
    ...(measuredRuns === undefined ? {} : { measuredRuns }),
    ...(warmupRuns === undefined ? {} : { warmupRuns }),
    headed,
  };
}

function positiveInteger(
  value: string,
  option: string,
  minimum: number,
): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < minimum) {
    throw new Error(`${option} must be an integer >= ${minimum}`);
  }
  return parsed;
}

function startStaticServer(staticDirectory: string): Bun.Server<undefined> {
  const rootPrefix = `${staticDirectory}${sep}`;
  return Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    async fetch(request) {
      const requestUrl = new URL(request.url);
      const pathname =
        requestUrl.pathname === "/"
          ? "index.html"
          : decodeURIComponent(requestUrl.pathname.slice(1));
      const path = resolve(staticDirectory, pathname);
      if (path !== staticDirectory && !path.startsWith(rootPrefix)) {
        return new Response("not found", { status: 404 });
      }
      const file = Bun.file(path);
      if (!(await file.exists())) {
        return new Response("not found", { status: 404 });
      }
      return new Response(file, {
        headers: {
          "Cache-Control": "no-store, must-revalidate",
          "Content-Type": contentType(path),
        },
      });
    },
  });
}

function contentType(path: string): string {
  switch (extname(path)) {
    case ".html":
      return "text/html; charset=utf-8";
    case ".js":
      return "text/javascript; charset=utf-8";
    case ".css":
      return "text/css; charset=utf-8";
    case ".wasm":
      return "application/wasm";
    case ".jpg":
    case ".jpeg":
      return "image/jpeg";
    case ".png":
      return "image/png";
    case ".txt":
      return "text/plain; charset=utf-8";
    default:
      return "application/octet-stream";
  }
}

async function openBrowser(options: CliOptions): Promise<BrowserHandle> {
  if (options.cdpUrl) {
    await waitForCdp(options.cdpUrl, 5_000);
    return { cdpUrl: options.cdpUrl, close: async () => undefined };
  }

  const executable =
    options.browserExecutable ?? (await discoverBrowserExecutable());
  if (!executable) {
    throw new Error(
      "Chromium/Chrome was not found. Pass --browser /path/to/chrome or --cdp http://127.0.0.1:9222.",
    );
  }

  const cdpPort = await findFreePort();
  const profileDir = await mkdtemp(
    join(tmpdir(), "fighter-notes-local-video-e2e-"),
  );
  const subprocess = Bun.spawn(
    [
      executable,
      ...(options.headed ? [] : ["--headless=new"]),
      "--no-sandbox",
      "--disable-dev-shm-usage",
      "--disable-background-timer-throttling",
      "--disable-renderer-backgrounding",
      `--remote-debugging-port=${cdpPort}`,
      `--user-data-dir=${profileDir}`,
      "about:blank",
    ],
    { stderr: "ignore", stdout: "ignore" },
  );
  const cdpUrl = `http://127.0.0.1:${cdpPort}`;
  try {
    await waitForCdp(cdpUrl, 15_000);
  } catch (error) {
    subprocess.kill();
    await subprocess.exited;
    await rm(profileDir, { recursive: true, force: true });
    throw error;
  }

  return {
    cdpUrl,
    async close() {
      subprocess.kill();
      await subprocess.exited;
      await rm(profileDir, { recursive: true, force: true });
    },
  };
}

async function discoverBrowserExecutable(): Promise<string | undefined> {
  for (const command of [
    "chromium",
    "chromium-browser",
    "google-chrome",
    "google-chrome-stable",
  ]) {
    const found = Bun.which(command);
    if (found) return found;
  }

  const cacheRoot = join(homedir(), ".cache/ms-playwright");
  if (!existsSync(cacheRoot)) return undefined;
  const glob = new Bun.Glob("chromium-*/chrome-linux*/chrome");
  const candidates: string[] = [];
  for await (const candidate of glob.scan({
    cwd: cacheRoot,
    absolute: true,
    onlyFiles: true,
  })) {
    candidates.push(candidate);
  }
  return candidates.sort().at(-1);
}

async function findFreePort(): Promise<number> {
  const server = Bun.listen({
    hostname: "127.0.0.1",
    port: 0,
    socket: {
      data() {},
    },
  });
  const port = server.port;
  server.stop(true);
  return port;
}

async function waitForCdp(cdpUrl: string, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: unknown;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${cdpUrl}/json/version`);
      if (response.ok) return;
      lastError = new Error(`CDP returned HTTP ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    await Bun.sleep(100);
  }
  throw new Error(`could not connect to Chrome at ${cdpUrl}`, {
    cause: lastError,
  });
}

async function analyzeCase(
  cdpUrl: string,
  appUrl: string,
  fixture: LocalVideoCase,
): Promise<CapturedWorkerArtifacts> {
  const target = await createTarget(cdpUrl);
  const cdp = await CdpSession.connect(target.webSocketDebuggerUrl);
  try {
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("DOM.enable");
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", {
      source: captureBootstrap(),
    });

    const loaded = cdp.waitForEvent("Page.loadEventFired");
    await cdp.send("Page.navigate", { url: appUrl });
    await loaded;
    await poll(
      async () =>
        cdp.evaluate<boolean>(
          "Boolean(document.querySelector('#file-input') && document.querySelector('#side-select'))",
        ),
      15_000,
      `${fixture.id}: setup page did not become ready`,
    );

    const document = await cdp.send<{ root: { nodeId: number } }>(
      "DOM.getDocument",
      { depth: -1, pierce: true },
    );
    const input = await cdp.send<{ nodeId: number }>("DOM.querySelector", {
      nodeId: document.root.nodeId,
      selector: "#file-input",
    });
    if (!input.nodeId) {
      throw new Error(`${fixture.id}: #file-input was not found`);
    }
    await cdp.send("DOM.setFileInputFiles", {
      nodeId: input.nodeId,
      files: [fixture.browserVideoPath ?? fixture.videoPath],
    });

    await cdp.evaluate(
      setupExpression(
        fixture.side,
        fixture.ownCharacter,
        fixture.opponentCharacter,
      ),
    );
    await poll(
      async () =>
        cdp.evaluate<boolean>(
          "document.querySelector('.analyze-btn')?.disabled === false",
        ),
      10_000,
      `${fixture.id}: analyze button did not become enabled`,
    );
    const started = await cdp.evaluate<boolean>(`
      (() => {
        const state = globalThis.__fighterNotesLocalE2E;
        const button = document.querySelector(".analyze-btn");
        if (!state || !(button instanceof HTMLButtonElement) || button.disabled) {
          return false;
        }
        state.startedAt = performance.now();
        button.click();
        return true;
      })()
    `);
    if (!started) throw new Error(`${fixture.id}: failed to start analysis`);

    const timeoutMs =
      (fixture.timeoutSeconds ?? DEFAULT_TIMEOUT_SECONDS) * 1_000;
    await poll(
      async () => {
        const status = await cdp.evaluate<{
          done: boolean;
          workerError?: string;
          appError?: string;
        }>(`
          (() => ({
            done: Boolean(globalThis.__fighterNotesLocalE2E?.done),
            workerError: globalThis.__fighterNotesLocalE2E?.workerError,
            appError: document.querySelector(".analysis-error")?.textContent || undefined,
          }))()
        `);
        if (status.workerError) {
          throw new Error(`${fixture.id}: worker error: ${status.workerError}`);
        }
        if (status.appError) {
          throw new Error(`${fixture.id}: ${status.appError}`);
        }
        return status.done;
      },
      timeoutMs,
      `${fixture.id}: analysis timed out after ${timeoutMs / 1_000}s`,
    );
    const serialized = await cdp.evaluate<string>(
      "JSON.stringify(globalThis.__fighterNotesLocalE2E.artifacts)",
    );
    return JSON.parse(serialized) as CapturedWorkerArtifacts;
  } finally {
    cdp.close();
    await fetch(`${cdpUrl}/json/close/${target.id}`).catch(() => undefined);
  }
}

function captureBootstrap(): string {
  return `
    (() => {
      const NativeWorker = globalThis.Worker;
      const state = {
        done: false,
        startedAt: 0,
        firstPassAt: 0,
        workerError: undefined,
        spatialWindows: undefined,
        perfLogs: [],
        artifacts: undefined,
      };
      globalThis.__fighterNotesLocalE2E = state;
      const nativeConsoleLog = console.log.bind(console);
      console.log = (...args) => {
        if (typeof args[0] === "string" && args[0].startsWith("[perf]")) {
          state.perfLogs.push(args.map(String).join(" "));
        }
        nativeConsoleLog(...args);
      };
      globalThis.Worker = class LocalE2EWorker extends NativeWorker {
        constructor(...args) {
          super(...args);
          this.addEventListener("message", (event) => {
            const message = event.data;
            if (message?.type === "firstPass") {
              state.spatialWindows = message.spatialWindows;
              state.firstPassAt = performance.now();
            }
            if (message?.type === "done") {
              state.artifacts = {
                report: message.report,
                timeline: message.timeline,
                features: message.features,
                trackedInputs: message.trackedInputs,
                fightMarkers: message.fightMarkers,
                attackInfo: message.attackInfo,
                regressionEvents: message.regressionEvents,
                spatialWindows: state.spatialWindows,
                spatialObservations: message.spatialObservations,
                perfLogs: state.perfLogs,
                stageTimings: {
                  firstPass: state.firstPassAt - state.startedAt,
                  spatialPass: performance.now() - state.firstPassAt,
                },
                analysisMs: performance.now() - state.startedAt,
              };
              state.done = true;
            }
          });
          this.addEventListener("error", (event) => {
            state.workerError = event.message || "unknown Worker error";
          });
        }
      };
    })();
  `;
}

function setupExpression(
  side: string,
  ownCharacter: string,
  opponentCharacter: string,
): string {
  return `
    (() => {
      const setSelect = (id, value) => {
        const select = document.getElementById(id);
        if (!(select instanceof HTMLSelectElement)) {
          throw new Error(id + " was not found");
        }
        const setter = Object.getOwnPropertyDescriptor(
          HTMLSelectElement.prototype,
          "value",
        ).set;
        setter.call(select, value);
        select.dispatchEvent(new Event("change", { bubbles: true }));
      };
      setSelect("side-select", ${JSON.stringify(side)});
      setSelect("char-select", ${JSON.stringify(ownCharacter)});
      setSelect("opponent-char-select", ${JSON.stringify(opponentCharacter)});
      return true;
    })()
  `;
}

async function createTarget(cdpUrl: string): Promise<CdpTarget> {
  const response = await fetch(
    `${cdpUrl}/json/new?${encodeURIComponent("about:blank")}`,
    { method: "PUT" },
  );
  if (!response.ok) {
    throw new Error(`Chrome could not create a page: HTTP ${response.status}`);
  }
  return (await response.json()) as CdpTarget;
}

class CdpSession {
  readonly #socket: WebSocket;
  readonly #pending = new Map<
    number,
    {
      resolve(value: unknown): void;
      reject(reason: unknown): void;
    }
  >();
  readonly #eventWaiters = new Map<
    string,
    Array<(parameters: unknown) => void>
  >();
  #nextId = 1;

  private constructor(socket: WebSocket) {
    this.#socket = socket;
    socket.addEventListener("message", (event) => {
      const message = JSON.parse(String(event.data)) as {
        id?: number;
        method?: string;
        params?: unknown;
        result?: unknown;
        error?: { message?: string };
      };
      if (message.id !== undefined) {
        const pending = this.#pending.get(message.id);
        if (!pending) return;
        this.#pending.delete(message.id);
        if (message.error) {
          pending.reject(new Error(message.error.message ?? "CDP error"));
        } else {
          pending.resolve(message.result);
        }
        return;
      }
      if (!message.method) return;
      const waiters = this.#eventWaiters.get(message.method);
      if (!waiters) return;
      this.#eventWaiters.delete(message.method);
      for (const resolveEvent of waiters) resolveEvent(message.params);
    });
    socket.addEventListener("close", () => {
      for (const pending of this.#pending.values()) {
        pending.reject(new Error("Chrome DevTools connection closed"));
      }
      this.#pending.clear();
    });
  }

  static async connect(url: string): Promise<CdpSession> {
    const socket = new WebSocket(url);
    await new Promise<void>((resolveOpen, rejectOpen) => {
      socket.addEventListener("open", () => resolveOpen(), { once: true });
      socket.addEventListener(
        "error",
        () => rejectOpen(new Error("Chrome DevTools connection failed")),
        { once: true },
      );
    });
    return new CdpSession(socket);
  }

  send<T = unknown>(
    method: string,
    params: Readonly<Record<string, unknown>> = {},
  ): Promise<T> {
    const id = this.#nextId;
    this.#nextId += 1;
    return new Promise<T>((resolveResult, rejectResult) => {
      this.#pending.set(id, {
        resolve: (value) => resolveResult(value as T),
        reject: rejectResult,
      });
      this.#socket.send(JSON.stringify({ id, method, params }));
    });
  }

  async evaluate<T = unknown>(expression: string): Promise<T> {
    const response = await this.send<{
      result: { value?: T; description?: string };
      exceptionDetails?: { text?: string };
    }>("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (response.exceptionDetails) {
      throw new Error(
        response.exceptionDetails.text ??
          response.result.description ??
          "browser evaluation failed",
      );
    }
    return response.result.value as T;
  }

  waitForEvent(method: string): Promise<unknown> {
    return new Promise((resolveEvent) => {
      const waiters = this.#eventWaiters.get(method) ?? [];
      waiters.push(resolveEvent);
      this.#eventWaiters.set(method, waiters);
    });
  }

  close(): void {
    this.#socket.close();
  }
}

async function poll(
  predicate: () => Promise<boolean>,
  timeoutMs: number,
  message: string,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await Bun.sleep(100);
  }
  throw new Error(message);
}

function parseArtifact(value: string, label: string, caseId: string): unknown {
  try {
    return JSON.parse(value);
  } catch (error) {
    throw new Error(`${caseId}: ${label} is not valid JSON`, { cause: error });
  }
}

function parseOptionalArtifact(
  value: string | undefined,
  label: string,
  caseId: string,
): unknown {
  return value === undefined ? null : parseArtifact(value, label, caseId);
}

function sha256(value: string): string {
  return new Bun.CryptoHasher("sha256").update(value).digest("hex");
}

async function sha256File(path: string): Promise<string> {
  const hasher = new Bun.CryptoHasher("sha256");
  for await (const chunk of Bun.file(path).stream()) hasher.update(chunk);
  return hasher.digest("hex");
}

function hasAccuracyContract(fixture: LocalVideoCase): boolean {
  return (
    (fixture.expect?.semanticEvents?.length ?? 0) > 0 &&
    Object.values(fixture.expect?.detectorGates ?? {}).some(
      (gate) => gate !== undefined && Object.keys(gate).length > 0,
    )
  );
}

function extractStageTimings(
  captured: CapturedWorkerArtifacts,
): Readonly<Record<string, number>> {
  const stages: Record<string, number> = { ...(captured.stageTimings ?? {}) };
  const totalLog = [...(captured.perfLogs ?? [])]
    .reverse()
    .find((line) => /^\[perf\] \d+f total:/.test(line));
  if (!totalLog) return stages;
  const labels: Readonly<Record<string, string>> = {
    "draw+get": "frameExtraction",
    worker_copy: "workerCopy",
    meter: "meterWasm",
    hud: "hudWasm",
  };
  for (const match of totalLog.matchAll(/([a-z+_]+)=(\d+(?:\.\d+)?)ms/g)) {
    const label = labels[match[1]];
    const value = Number(match[2]);
    if (label && Number.isFinite(value)) stages[label] = value;
  }
  return stages;
}

function requiredStageTimings(
  captured: CapturedWorkerArtifacts,
  caseId: string,
): Readonly<Record<string, number>> {
  const stages = extractStageTimings(captured);
  for (const stage of REQUIRED_PERFORMANCE_STAGES) {
    const value = stages[stage];
    if (!Number.isFinite(value) || value < 0) {
      throw new Error(
        `${caseId}: required performance stage ${stage} is missing`,
      );
    }
  }
  return stages;
}

function semanticCapturedDigest(captured: CapturedWorkerArtifacts): string {
  return sha256(
    [
      captured.report,
      captured.timeline,
      captured.features,
      captured.trackedInputs ?? "",
      captured.fightMarkers ?? "",
      captured.attackInfo ?? "",
      captured.regressionEvents,
      captured.spatialWindows ?? "",
      captured.spatialObservations ?? "",
    ].join("\u0000"),
  );
}

function parseRunSummary(value: unknown, label: string): RunSummary {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(
    value,
    [
      "schemaVersion",
      "runnerVersion",
      "warmupRuns",
      "measuredRuns",
      "generatedAt",
      "cases",
    ],
    label,
  );
  if (value.schemaVersion !== 2 || value.runnerVersion !== RUNNER_VERSION) {
    throw new Error(
      `${label} is an incompatible baseline; regenerate it with this runner`,
    );
  }
  const warmupRuns = requiredInteger(
    value.warmupRuns,
    `${label}.warmupRuns`,
    1,
  );
  const measuredRuns = requiredInteger(
    value.measuredRuns,
    `${label}.measuredRuns`,
    3,
  );
  const generatedAt = requiredString(value.generatedAt, `${label}.generatedAt`);
  if (!Number.isFinite(Date.parse(generatedAt))) {
    throw new Error(`${label}.generatedAt must be an ISO timestamp`);
  }
  if (!Array.isArray(value.cases) || value.cases.length === 0) {
    throw new Error(`${label}.cases must contain at least one case`);
  }
  const cases = value.cases.map((entry, index) =>
    parseCaseSummary(entry, `${label}.cases[${index}]`, measuredRuns),
  );
  const ids = new Set(cases.map((entry) => entry.id));
  if (ids.size !== cases.length)
    throw new Error(`${label}.cases contains duplicate ids`);
  return {
    schemaVersion: 2,
    runnerVersion: RUNNER_VERSION,
    warmupRuns,
    measuredRuns,
    generatedAt,
    cases,
  };
}

function parseCaseSummary(
  value: unknown,
  label: string,
  measuredRuns: number,
): CaseSummary {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(
    value,
    [
      "id",
      "videoName",
      "fixtureFingerprint",
      "settings",
      "expectationHash",
      "analysisMs",
      "performance",
      "detectorMetrics",
      "syntheticCoverage",
      "assertionsPassed",
      "assertionFailures",
      "hashes",
      "semanticHash",
    ],
    label,
  );
  const performance = parseTimingSummary(
    value.performance,
    `${label}.performance`,
    measuredRuns,
  );
  const analysisMs = requiredFiniteNumber(
    value.analysisMs,
    `${label}.analysisMs`,
    0,
  );
  if (Math.abs(analysisMs - performance.medianMs) > 1e-6) {
    throw new Error(`${label}.analysisMs must equal performance.medianMs`);
  }
  if (typeof value.assertionsPassed !== "boolean") {
    throw new Error(`${label}.assertionsPassed must be boolean`);
  }
  if (
    !Array.isArray(value.assertionFailures) ||
    value.assertionFailures.some((failure) => typeof failure !== "string")
  ) {
    throw new Error(`${label}.assertionFailures must be a string array`);
  }
  if (value.assertionsPassed !== (value.assertionFailures.length === 0)) {
    throw new Error(
      `${label}.assertionsPassed must agree with assertionFailures`,
    );
  }
  if (!isRecord(value.hashes))
    throw new Error(`${label}.hashes must be an object`);
  const rawHashes = value.hashes;
  assertExactKeys(rawHashes, CAPTURE_HASH_FIELDS, `${label}.hashes`);
  const hashes = Object.fromEntries(
    CAPTURE_HASH_FIELDS.map((field) => [
      field,
      requiredSha256(rawHashes[field], `${label}.hashes.${field}`),
    ]),
  ) as Record<CaptureHashField, string>;
  const syntheticCoverage = parseSyntheticCoverage(
    value.syntheticCoverage,
    `${label}.syntheticCoverage`,
  );
  return {
    id: requiredString(value.id, `${label}.id`),
    videoName: requiredString(value.videoName, `${label}.videoName`),
    fixtureFingerprint: requiredSha256(
      value.fixtureFingerprint,
      `${label}.fixtureFingerprint`,
    ),
    settings: parseFixtureSettings(value.settings, `${label}.settings`),
    expectationHash: requiredSha256(
      value.expectationHash,
      `${label}.expectationHash`,
    ),
    analysisMs,
    performance,
    detectorMetrics: parseDetectorMetrics(
      value.detectorMetrics,
      `${label}.detectorMetrics`,
    ),
    syntheticCoverage,
    assertionsPassed: value.assertionsPassed,
    assertionFailures: value.assertionFailures,
    hashes,
    semanticHash: requiredSha256(value.semanticHash, `${label}.semanticHash`),
  };
}

function parseTimingSummary(
  value: unknown,
  label: string,
  measuredRuns: number,
): TimingSummary {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["runsMs", "medianMs", "p90Ms", "stages"], label);
  if (
    !Array.isArray(value.runsMs) ||
    value.runsMs.length !== measuredRuns ||
    value.runsMs.some((run) => !isFiniteNumberAtLeast(run, 0))
  ) {
    throw new Error(
      `${label}.runsMs must contain ${measuredRuns} finite values`,
    );
  }
  const runsMs = value.runsMs as number[];
  const derived = summarizeTimings(
    runsMs.map((analysisMs) => ({ analysisMs, stages: {} })),
  );
  const medianMs = requiredFiniteNumber(value.medianMs, `${label}.medianMs`, 0);
  const p90Ms = requiredFiniteNumber(value.p90Ms, `${label}.p90Ms`, 0);
  if (
    Math.abs(medianMs - derived.medianMs) > 1e-6 ||
    Math.abs(p90Ms - derived.p90Ms) > 1e-6
  ) {
    throw new Error(`${label} median/p90 must agree with runsMs`);
  }
  if (!isRecord(value.stages))
    throw new Error(`${label}.stages must be an object`);
  const stages: Record<string, { medianMs: number; p90Ms: number }> = {};
  for (const [stage, timing] of Object.entries(value.stages)) {
    if (!isRecord(timing))
      throw new Error(`${label}.stages.${stage} must be an object`);
    assertExactKeys(timing, ["medianMs", "p90Ms"], `${label}.stages.${stage}`);
    stages[stage] = {
      medianMs: requiredFiniteNumber(
        timing.medianMs,
        `${label}.stages.${stage}.medianMs`,
        0,
      ),
      p90Ms: requiredFiniteNumber(
        timing.p90Ms,
        `${label}.stages.${stage}.p90Ms`,
        0,
      ),
    };
  }
  for (const stage of REQUIRED_PERFORMANCE_STAGES) {
    if (!stages[stage]) throw new Error(`${label}.stages.${stage} is required`);
  }
  return {
    runsMs,
    medianMs,
    p90Ms,
    stages,
  };
}

function parseDetectorMetrics(
  value: unknown,
  label: string,
): Partial<Readonly<Record<DetectorId, DetectorMetrics>>> {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  const result: Partial<Record<DetectorId, DetectorMetrics>> = {};
  for (const [detector, metric] of Object.entries(value)) {
    if (!DETECTOR_IDS.includes(detector as DetectorId) || !isRecord(metric)) {
      throw new Error(`${label}.${detector} is invalid`);
    }
    assertExactKeys(
      metric,
      [
        "expected",
        "actual",
        "matched",
        "falsePositives",
        "falseNegatives",
        "precision",
        "recall",
        "meanAbsoluteFrameError",
        "maxAbsoluteFrameError",
      ],
      `${label}.${detector}`,
    );
    const parsed: DetectorMetrics = {
      expected: requiredInteger(
        metric.expected,
        `${label}.${detector}.expected`,
        0,
      ),
      actual: requiredInteger(metric.actual, `${label}.${detector}.actual`, 0),
      matched: requiredInteger(
        metric.matched,
        `${label}.${detector}.matched`,
        0,
      ),
      falsePositives: requiredInteger(
        metric.falsePositives,
        `${label}.${detector}.falsePositives`,
        0,
      ),
      falseNegatives: requiredInteger(
        metric.falseNegatives,
        `${label}.${detector}.falseNegatives`,
        0,
      ),
      precision: requiredRatio(
        metric.precision,
        `${label}.${detector}.precision`,
      ),
      recall: requiredRatio(metric.recall, `${label}.${detector}.recall`),
      meanAbsoluteFrameError: requiredFiniteNumber(
        metric.meanAbsoluteFrameError,
        `${label}.${detector}.meanAbsoluteFrameError`,
        0,
      ),
      maxAbsoluteFrameError: requiredFiniteNumber(
        metric.maxAbsoluteFrameError,
        `${label}.${detector}.maxAbsoluteFrameError`,
        0,
      ),
    };
    if (
      parsed.matched > parsed.expected ||
      parsed.matched > parsed.actual ||
      parsed.falsePositives !== parsed.actual - parsed.matched ||
      parsed.falseNegatives !== parsed.expected - parsed.matched
    ) {
      throw new Error(`${label}.${detector} counts are inconsistent`);
    }
    result[detector as DetectorId] = parsed;
  }
  return result;
}

function parseFixtureSettings(value: unknown, label: string): FixtureSettings {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["side", "ownCharacter", "opponentCharacter"], label);
  if (value.side !== "p1" && value.side !== "p2") {
    throw new Error(`${label}.side must be p1 or p2`);
  }
  return {
    side: value.side,
    ownCharacter: requiredString(value.ownCharacter, `${label}.ownCharacter`),
    opponentCharacter: requiredString(
      value.opponentCharacter,
      `${label}.opponentCharacter`,
    ),
  };
}

function parseSyntheticCoverage(
  value: unknown,
  label: string,
): CaseSummary["syntheticCoverage"] {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["ported", "pending", "pendingIds"], label);
  if (
    !Array.isArray(value.pendingIds) ||
    value.pendingIds.some((id) => typeof id !== "string")
  ) {
    throw new Error(`${label}.pendingIds must be a string array`);
  }
  const ported = requiredInteger(value.ported, `${label}.ported`, 0);
  const pending = requiredInteger(value.pending, `${label}.pending`, 0);
  if (pending !== value.pendingIds.length) {
    throw new Error(`${label}.pending must equal pendingIds.length`);
  }
  return { ported, pending, pendingIds: value.pendingIds };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function assertExactKeys(
  value: Readonly<Record<string, unknown>>,
  expected: readonly string[],
  label: string,
): void {
  const actual = Object.keys(value).sort();
  const required = [...expected].sort();
  if (
    actual.length !== required.length ||
    actual.some((key, index) => key !== required[index])
  ) {
    throw new Error(
      `${label} fields must be ${required.join(", ")}; got ${actual.join(", ")}`,
    );
  }
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value;
}

function requiredSha256(value: unknown, label: string): string {
  const hash = requiredString(value, label);
  if (!/^[0-9a-f]{64}$/.test(hash)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
  return hash;
}

function requiredInteger(
  value: unknown,
  label: string,
  minimum: number,
): number {
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < minimum
  ) {
    throw new Error(`${label} must be an integer >= ${minimum}`);
  }
  return value;
}

function isFiniteNumberAtLeast(
  value: unknown,
  minimum: number,
): value is number {
  return (
    typeof value === "number" && Number.isFinite(value) && value >= minimum
  );
}

function requiredFiniteNumber(
  value: unknown,
  label: string,
  minimum: number,
): number {
  if (!isFiniteNumberAtLeast(value, minimum)) {
    throw new Error(`${label} must be a finite number >= ${minimum}`);
  }
  return value;
}

function requiredRatio(value: unknown, label: string): number {
  const ratio = requiredFiniteNumber(value, label, 0);
  if (ratio > 1) throw new Error(`${label} must be <= 1`);
  return ratio;
}

async function loadBaseline(
  baselineOption: string | undefined,
  root: string,
  outputDirectory: string,
): Promise<BaselineData | undefined> {
  if (!baselineOption) return undefined;
  const directory = resolve(root, baselineOption);
  if (directory === outputDirectory) {
    throw new Error("--baseline and --output must be different directories");
  }
  const summaryPath = join(directory, "summary.json");
  const parsed: unknown = JSON.parse(await readFile(summaryPath, "utf8"));
  return {
    directory,
    summary: parseRunSummary(parsed, relative(root, summaryPath)),
  };
}

async function compareWithBaseline(
  baseline: BaselineData,
  fixture: LocalVideoCase,
  currentArtifact: Record<string, unknown>,
  currentPerformance: TimingSummary,
  currentMetrics: Partial<Readonly<Record<DetectorId, DetectorMetrics>>>,
  policy: LocalVideoPerformancePolicy,
  fixtureFingerprint: string,
  settings: FixtureSettings,
  expectationHash: string,
): Promise<string[]> {
  const previous = baseline.summary.cases.find(
    (candidate) => candidate.id === fixture.id,
  );
  if (!previous)
    return [`${fixture.id}: case is missing from baseline summary`];
  const failures: string[] = [];
  if (!previous.assertionsPassed) {
    failures.push(`${fixture.id}: baseline did not pass its assertions`);
  }
  if (previous.fixtureFingerprint !== fixtureFingerprint) {
    failures.push(
      `${fixture.id}: video content does not match the baseline fixture`,
    );
  }
  if (JSON.stringify(previous.settings) !== JSON.stringify(settings)) {
    failures.push(
      `${fixture.id}: side or character settings do not match the baseline`,
    );
  }
  if (previous.expectationHash !== expectationHash) {
    failures.push(
      `${fixture.id}: annotation/expectation contract changed; create and approve a new baseline`,
    );
  }
  if (previous.performance.runsMs.length < 3) {
    failures.push(
      `${fixture.id}: baseline must contain at least 3 measured runs`,
    );
  }

  failures.push(
    ...comparePerformance(currentPerformance, previous.performance, policy),
    ...compareDetectorMetrics(
      currentMetrics,
      previous.detectorMetrics,
      fixture.expect?.detectorGates,
    ),
  );
  const ratio = currentPerformance.medianMs / previous.performance.medianMs;
  console.log(
    `[local-e2e] ${fixture.id}: median ${currentPerformance.medianMs.toFixed(
      0,
    )}ms vs ${previous.performance.medianMs.toFixed(0)}ms (${ratio >= 1 ? "+" : ""}${(
      (ratio - 1) * 100
    ).toFixed(1)}%)`,
  );

  const artifactPath = join(baseline.directory, `${fixture.id}.json`);
  if (!existsSync(artifactPath)) {
    failures.push(`${fixture.id}: baseline artifact is missing`);
    return failures;
  }
  const baselineArtifact: unknown = JSON.parse(
    await readFile(artifactPath, "utf8"),
  );
  let parsedArtifact: BaselineCaseArtifact;
  try {
    parsedArtifact = parseBaselineArtifact(
      baselineArtifact,
      relative(baseline.directory, artifactPath),
      baseline.summary.measuredRuns,
    );
  } catch (error) {
    failures.push(
      `${fixture.id}: baseline artifact is invalid: ${error instanceof Error ? error.message : String(error)}`,
    );
    return failures;
  }
  const baselineContract = parsedArtifact.fixtureContract;
  if (
    parsedArtifact.caseId !== previous.id ||
    parsedArtifact.videoName !== previous.videoName ||
    parsedArtifact.analysisMs !== previous.analysisMs ||
    canonicalJson(parsedArtifact.performance) !==
      canonicalJson(previous.performance) ||
    baselineContract.fixtureFingerprint !== previous.fixtureFingerprint ||
    baselineContract.expectationHash !== previous.expectationHash ||
    canonicalJson(baselineContract.settings) !==
      canonicalJson(previous.settings)
  ) {
    failures.push(
      `${fixture.id}: baseline artifact does not match its summary contract`,
    );
    return failures;
  }
  if (
    baselineContract.fixtureFingerprint !== fixtureFingerprint ||
    baselineContract.expectationHash !== expectationHash ||
    canonicalJson(baselineContract.settings) !== canonicalJson(settings)
  ) {
    failures.push(
      `${fixture.id}: baseline artifact fixture contract is invalid`,
    );
    return failures;
  }
  const identityFailures = compareArtifactIdentity(parsedArtifact, previous);
  if (identityFailures.length > 0) {
    failures.push(
      ...identityFailures.map(
        (failure) => `${fixture.id}: baseline artifact ${failure}`,
      ),
    );
    return failures;
  }
  const semanticDifferences = diffSemanticValues(
    semanticSnapshot(parsedArtifact),
    semanticSnapshot(currentArtifact),
  );
  if (semanticDifferences.length > 0) {
    failures.push(
      `${fixture.id}: semantic output changed; inspect the structured diff and promote a new baseline only after approval`,
    );
    console.error(`[local-e2e] ${fixture.id}: semantic diff (first 50)`);
    for (const difference of semanticDifferences)
      console.error(`  - ${difference}`);
  }
  return failures;
}
