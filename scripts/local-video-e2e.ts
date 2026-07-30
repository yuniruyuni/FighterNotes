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
  compareTimings,
  evaluateExpectations,
  type LocalVideoCase,
  parseLocalVideoManifest,
} from "./local-video-e2e-lib";

const DEFAULT_MANIFEST = "video/local-video-e2e.json";
const DEFAULT_OUTPUT = "output/local-video-e2e/current";
const DEFAULT_TIMEOUT_SECONDS = 600;

interface CliOptions {
  readonly manifestPath: string;
  readonly outputDir: string;
  readonly baselineDir?: string;
  readonly cdpUrl?: string;
  readonly browserExecutable?: string;
  readonly headed: boolean;
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
  readonly spatialWindows?: string;
  readonly spatialObservations?: string;
  readonly perfLogs?: string[];
  readonly analysisMs: number;
}

interface CaseSummary {
  readonly id: string;
  readonly videoName: string;
  readonly analysisMs: number;
  readonly assertionsPassed: boolean;
  readonly assertionFailures: readonly string[];
  readonly hashes: Readonly<Record<string, string>>;
}

interface RunSummary {
  readonly schemaVersion: 1;
  readonly generatedAt: string;
  readonly cases: readonly CaseSummary[];
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
for (const fixture of manifest.cases) {
  if (!isAbsolute(fixture.videoPath) || !existsSync(fixture.videoPath)) {
    throw new Error(
      `${fixture.id}: videoPath must be an existing absolute local path`,
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
    console.log(`[local-e2e] ${fixture.id}: starting`);
    const captured = await analyzeCase(browser.cdpUrl, appUrl, fixture);
    const report = parseArtifact(captured.report, "report", fixture.id);
    const artifacts = {
      schemaVersion: 1,
      caseId: fixture.id,
      videoName: basename(fixture.videoPath),
      settings: {
        side: fixture.side,
        ownCharacter: fixture.ownCharacter,
        opponentCharacter: fixture.opponentCharacter,
      },
      analysisMs: captured.analysisMs,
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

    const assertionFailures = evaluateExpectations(report, fixture.expect);
    failed ||= assertionFailures.length > 0;
    const hashes = {
      report: sha256(captured.report),
      timeline: sha256(captured.timeline),
      features: sha256(captured.features),
      trackedInputs: sha256(captured.trackedInputs ?? ""),
      fightMarkers: sha256(captured.fightMarkers ?? ""),
      attackInfo: sha256(captured.attackInfo ?? ""),
      spatialWindows: sha256(captured.spatialWindows ?? ""),
      spatialObservations: sha256(captured.spatialObservations ?? ""),
    };
    summaries.push({
      id: fixture.id,
      videoName: basename(fixture.videoPath),
      analysisMs: captured.analysisMs,
      assertionsPassed: assertionFailures.length === 0,
      assertionFailures,
      hashes,
    });
    console.log(
      `[local-e2e] ${fixture.id}: ${(captured.analysisMs / 1_000).toFixed(2)}s, ${
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
  schemaVersion: 1,
  generatedAt: new Date().toISOString(),
  cases: summaries,
};
await Bun.write(
  join(outputDir, "summary.json"),
  `${JSON.stringify(summary, null, 2)}\n`,
);
await printBaselineComparison(options.baselineDir, projectRoot, summaries);

console.log(`[local-e2e] artifacts: ${relative(projectRoot, outputDir)}`);
if (failed) process.exitCode = 1;

function parseArgs(args: readonly string[]): CliOptions {
  let manifestPath = DEFAULT_MANIFEST;
  let outputDir = DEFAULT_OUTPUT;
  let baselineDir: string | undefined;
  let cdpUrl: string | undefined;
  let browserExecutable: string | undefined;
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
    headed,
  };
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
            }
            if (message?.type === "done") {
              state.artifacts = {
                report: message.report,
                timeline: message.timeline,
                features: message.features,
                trackedInputs: message.trackedInputs,
                fightMarkers: message.fightMarkers,
                attackInfo: message.attackInfo,
                spatialWindows: state.spatialWindows,
                spatialObservations: message.spatialObservations,
                perfLogs: state.perfLogs,
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

async function printBaselineComparison(
  baselineOption: string | undefined,
  root: string,
  current: readonly CaseSummary[],
): Promise<void> {
  if (!baselineOption) return;
  const baselinePath = resolve(root, baselineOption, "summary.json");
  const baseline = JSON.parse(
    await readFile(baselinePath, "utf8"),
  ) as RunSummary;
  const comparisons = compareTimings(
    Object.fromEntries(current.map((entry) => [entry.id, entry.analysisMs])),
    Object.fromEntries(
      baseline.cases.map((entry) => [entry.id, entry.analysisMs]),
    ),
  );
  for (const comparison of comparisons) {
    const delta = (comparison.ratio - 1) * 100;
    console.log(
      `[local-e2e] ${comparison.id}: ${comparison.currentMs.toFixed(
        0,
      )}ms vs ${comparison.baselineMs.toFixed(0)}ms (${delta >= 0 ? "+" : ""}${delta.toFixed(1)}%)`,
    );
  }
}
