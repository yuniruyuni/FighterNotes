import { readFile } from "node:fs/promises";

const RSS_SAMPLING_INTERVAL_MS = 100;

export interface ProcessTreeRssSampler {
  readonly peakBytes: number | null;
  stop(): Promise<void>;
}

interface ProcessTreeRssSamplerDependencies {
  readonly platform?: NodeJS.Platform;
  readonly readTreeRssBytes?: (rootPid: number) => Promise<number>;
  readonly sleep?: (milliseconds: number) => Promise<void>;
}

export function startProcessTreeRssSampler(
  rootPid: number | undefined,
  dependencies: ProcessTreeRssSamplerDependencies = {},
): ProcessTreeRssSampler {
  const platform = dependencies.platform ?? process.platform;
  if (platform !== "linux" || rootPid === undefined) {
    return { peakBytes: null, stop: async () => undefined };
  }
  if (!Number.isSafeInteger(rootPid) || rootPid <= 0) {
    throw new Error("Chrome root PID must be a positive safe integer");
  }

  const readTree = dependencies.readTreeRssBytes ?? linuxProcessTreeRssBytes;
  const sleep = dependencies.sleep ?? Bun.sleep;
  let running = true;
  let peakBytes = 0;
  let stopping: Promise<void> | undefined;
  const sampling = (async () => {
    while (running) {
      peakBytes = Math.max(peakBytes, await readTree(rootPid));
      if (running) await sleep(RSS_SAMPLING_INTERVAL_MS);
    }
    peakBytes = Math.max(peakBytes, await readTree(rootPid));
  })();

  return {
    get peakBytes() {
      return peakBytes;
    },
    stop() {
      running = false;
      stopping ??= sampling.then(() => {
        if (peakBytes <= 0) {
          throw new Error(
            "Chrome process-tree RSS sampling returned no readable memory data",
          );
        }
      });
      return stopping;
    },
  };
}

export async function linuxProcessTreeRssBytes(
  rootPid: number,
): Promise<number> {
  const pending = [rootPid];
  const seen = new Set<number>();
  let total = 0;
  while (pending.length > 0) {
    const pid = pending.pop();
    if (pid === undefined || seen.has(pid)) continue;
    seen.add(pid);
    const [status, children] = await Promise.all([
      readProcFile(`/proc/${pid}/status`),
      readProcFile(`/proc/${pid}/task/${pid}/children`),
    ]);
    const rss = status?.match(/^VmRSS:\s+(\d+)\s+kB$/m);
    if (rss) total += Number(rss[1]) * 1024;
    for (const child of children?.trim().split(/\s+/) ?? []) {
      const childPid = Number(child);
      if (Number.isSafeInteger(childPid) && childPid > 0) {
        pending.push(childPid);
      }
    }
  }
  return total;
}

async function readProcFile(path: string): Promise<string | undefined> {
  try {
    return await readFile(path, "utf8");
  } catch {
    // A Chrome child may exit while its process tree is sampled.
    return undefined;
  }
}
