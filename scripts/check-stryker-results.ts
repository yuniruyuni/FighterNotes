type MutantStatus =
  | "Killed"
  | "Survived"
  | "NoCoverage"
  | "CompileError"
  | "RuntimeError"
  | "Timeout"
  | "Ignored"
  | "Pending";

interface MutantResult {
  id: string;
  status: MutantStatus;
  mutatorName: string;
  description?: string;
  statusReason?: string;
  location: { start: { line: number; column: number } };
}

interface MutationReport {
  files: Record<string, { mutants: MutantResult[] }>;
}

const blockingStatuses = new Set<MutantStatus>([
  "Survived",
  "NoCoverage",
  "RuntimeError",
  "Timeout",
  "Pending",
]);

export interface MutationSummary {
  total: number;
  counts: Partial<Record<MutantStatus, number>>;
  blocking: Array<{ file: string; mutant: MutantResult }>;
}

export function summarizeMutationReport(
  report: MutationReport,
): MutationSummary {
  const summary: MutationSummary = { total: 0, counts: {}, blocking: [] };
  for (const [file, result] of Object.entries(report.files)) {
    for (const mutant of result.mutants) {
      summary.total += 1;
      summary.counts[mutant.status] = (summary.counts[mutant.status] ?? 0) + 1;
      const undocumentedIgnore =
        mutant.status === "Ignored" && !mutant.statusReason?.trim();
      if (blockingStatuses.has(mutant.status) || undocumentedIgnore) {
        summary.blocking.push({ file, mutant });
      }
    }
  }
  return summary;
}

function formatMutant(file: string, mutant: MutantResult): string {
  const { line, column } = mutant.location.start;
  const detail = mutant.description ?? mutant.mutatorName;
  return `${file}:${line}:${column} ${mutant.status} ${detail}`;
}

async function main(reportPath: string | undefined): Promise<void> {
  if (!reportPath)
    throw new Error("Usage: check-stryker-results.ts REPORT.json");

  const reportFile = Bun.file(reportPath);
  if (!(await reportFile.exists())) {
    throw new Error(`Stryker report does not exist: ${reportPath}`);
  }

  const report = (await reportFile.json()) as MutationReport;
  const summary = summarizeMutationReport(report);
  if (summary.total === 0) throw new Error("Stryker produced no mutants");

  console.log(`Mutation results: ${JSON.stringify(summary.counts)}`);
  if (summary.blocking.length === 0) return;

  for (const { file, mutant } of summary.blocking.slice(0, 50)) {
    console.error(formatMutant(file, mutant));
  }
  throw new Error(
    `${summary.blocking.length} mutation result(s) need tests or explicit review`,
  );
}

if (import.meta.main) {
  await main(Bun.argv[2]);
}
