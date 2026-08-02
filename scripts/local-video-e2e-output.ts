import { mkdir, mkdtemp, realpath, rename, rm, stat } from "node:fs/promises";
import { basename, dirname, isAbsolute, join, relative, sep } from "node:path";

export interface OutputTransaction {
  readonly directory: string;
  publish(): Promise<void>;
  discard(): Promise<void>;
}

export function assertBaselineAnalyzedFileBinding(
  caseId: string,
  browserVideoPath: string | undefined,
): void {
  if (browserVideoPath === undefined) return;
  throw new Error(
    `${caseId}: baseline comparison cannot verify browserVideoPath content against videoPath; remove browserVideoPath and use a browser that can read videoPath`,
  );
}

export async function prepareOutputDirectories(
  outputDirectory: string,
  baselineDirectory?: string,
): Promise<string | undefined> {
  await mkdir(outputDirectory, { recursive: true });
  const physicalOutput = await realpath(outputDirectory);
  const outputStat = await stat(physicalOutput);
  if (!outputStat.isDirectory()) {
    throw new Error(`output path is not a directory: ${outputDirectory}`);
  }
  if (!baselineDirectory) return undefined;

  const physicalBaseline = await realpath(baselineDirectory);
  const baselineStat = await stat(physicalBaseline);
  if (!baselineStat.isDirectory()) {
    throw new Error(`baseline path is not a directory: ${baselineDirectory}`);
  }
  if (
    physicalBaseline === physicalOutput ||
    (baselineStat.dev === outputStat.dev && baselineStat.ino === outputStat.ino)
  ) {
    throw new Error(
      "--baseline and --output must be different physical directories",
    );
  }
  if (
    isDescendant(physicalOutput, physicalBaseline) ||
    isDescendant(physicalBaseline, physicalOutput)
  ) {
    throw new Error(
      "--baseline and --output must not contain one another physically",
    );
  }
  return physicalBaseline;
}

function isDescendant(parent: string, candidate: string): boolean {
  const path = relative(parent, candidate);
  return (
    path !== "" &&
    path !== ".." &&
    !path.startsWith(`..${sep}`) &&
    !isAbsolute(path)
  );
}

export async function beginOutputTransaction(
  outputDirectory: string,
): Promise<OutputTransaction> {
  const parent = dirname(outputDirectory);
  const name = basename(outputDirectory);
  await mkdir(parent, { recursive: true });
  const stagingDirectory = await mkdtemp(join(parent, `.${name}.staging-`));
  let active = true;

  return {
    directory: stagingDirectory,
    async publish() {
      if (!active) throw new Error("output transaction is no longer active");
      const summary = await stat(join(stagingDirectory, "summary.json")).catch(
        () => undefined,
      );
      if (!summary?.isFile()) {
        throw new Error(
          "staged E2E output is incomplete: summary.json is missing",
        );
      }

      const backupDirectory = await mkdtemp(join(parent, `.${name}.backup-`));
      await rm(backupDirectory, { recursive: true });
      let existingOutputMoved = false;
      try {
        await rename(outputDirectory, backupDirectory);
        existingOutputMoved = true;
        await rename(stagingDirectory, outputDirectory);
        active = false;
      } catch (error) {
        if (existingOutputMoved) {
          try {
            await rename(backupDirectory, outputDirectory);
          } catch (rollbackError) {
            throw new AggregateError(
              [error, rollbackError],
              `failed to publish E2E output and restore the previous output; previous output remains at ${backupDirectory}`,
            );
          }
        }
        throw error;
      }
      await rm(backupDirectory, { recursive: true, force: true });
    },
    async discard() {
      if (!active) return;
      active = false;
      await rm(stagingDirectory, { recursive: true, force: true });
    },
  };
}
