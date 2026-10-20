// dependency-cruiser で client の層構造を検査する。
//
// dependency-cruiser は 1 module も読めなかったときでも「違反なし」で exit 0 を返す。
// TypeScript 7 では tsc パーサが .ts を 1 つも読まずに素通りし、check:arch が
// 長いあいだ何も検査していなかった。読んだ module が 0 件なら失敗として扱う。
import { join } from "node:path";
import { format, type ICruiseResult } from "dependency-cruiser";

const clientDir = join(import.meta.dir, "..");

const result = Bun.spawnSync({
  cmd: [
    join(clientDir, "node_modules/.bin/depcruise"),
    "src",
    "--config",
    ".dependency-cruiser.cjs",
    "--output-type",
    "json",
  ],
  cwd: clientDir,
  stdout: "pipe",
  stderr: "inherit",
});

const stdout = new TextDecoder().decode(result.stdout);
let cruised: ICruiseResult;
try {
  cruised = JSON.parse(stdout) as ICruiseResult;
} catch {
  console.error(stdout);
  console.error(`depcruise exited with ${result.exitCode} without JSON output`);
  process.exit(result.exitCode || 1);
}

const report = await format(cruised, { outputType: "err" });
process.stdout.write(String(report.output));

if (cruised.summary.totalCruised === 0) {
  console.error(
    "dependency-cruiser cruised 0 modules; the parser is not reading src/",
  );
  process.exit(1);
}
process.exit(Math.max(result.exitCode ?? 1, report.exitCode));
