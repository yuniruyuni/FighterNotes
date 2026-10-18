// 開発時に、指定した glob に合うファイルが変わるたびコマンドを走らせ直す。
//
// 以前は onchange を使っていたが、依存する chokidar@3 が braces を引き、
// braces には修正版の無い advisory (GHSA-vfj7-8cjw-p6xm) が残っている。
// 監視するのは数個の glob だけなので、fs.watch と Bun.Glob で足りる。
//
//   bun scripts/watch-run.ts 'src/**/*.css' -- bun run build:styles
//
// 起動時に 1 回走らせる (onchange の --initial 相当)。実行中に変更が来たら、
// 終わった後にもう 1 回だけ走らせる。
import { watch } from "node:fs";
import { resolve } from "node:path";

export interface WatchArgs {
  patterns: string[];
  command: string[];
}

export function parseArgs(argv: readonly string[]): WatchArgs {
  const separator = argv.indexOf("--");
  if (separator === -1) {
    throw new Error("usage: watch-run.ts <glob>... -- <command>...");
  }
  const patterns = argv.slice(0, separator);
  const command = argv.slice(separator + 1);
  if (patterns.length === 0 || command.length === 0) {
    throw new Error("usage: watch-run.ts <glob>... -- <command>...");
  }
  return { patterns, command };
}

const GLOB_CHARS = /[*?[\]{}!]/;

// glob のうち、ワイルドカードが現れる前のディレクトリ部分。ここを再帰監視する。
export function watchRoot(absolutePattern: string): string {
  const segments = absolutePattern.split("/");
  const firstGlob = segments.findIndex((segment) => GLOB_CHARS.test(segment));
  // ワイルドカードを含まないパターンはファイルそのものなので、親を監視する。
  const end = firstGlob === -1 ? segments.length - 1 : firstGlob;
  return segments.slice(0, end).join("/") || "/";
}

export function createMatcher(
  patterns: readonly string[],
  cwd: string,
): (absolutePath: string) => boolean {
  const globs = patterns.map((pattern) => new Bun.Glob(resolve(cwd, pattern)));
  return (absolutePath) => globs.some((glob) => glob.match(absolutePath));
}

async function main(): Promise<void> {
  const { patterns, command } = parseArgs(process.argv.slice(2));
  const cwd = process.cwd();
  const matches = createMatcher(patterns, cwd);
  const roots = [
    ...new Set(patterns.map((pattern) => watchRoot(resolve(cwd, pattern)))),
  ];

  let running = false;
  let pending = false;
  const run = async (): Promise<void> => {
    if (running) {
      pending = true;
      return;
    }
    running = true;
    do {
      pending = false;
      const child = Bun.spawn(command, {
        stdio: ["inherit", "inherit", "inherit"],
      });
      await child.exited;
    } while (pending);
    running = false;
  };

  for (const root of roots) {
    watch(root, { recursive: true }, (_event, filename) => {
      if (filename && matches(resolve(root, filename))) {
        void run();
      }
    });
  }
  await run();
}

if (import.meta.main) {
  await main();
}
