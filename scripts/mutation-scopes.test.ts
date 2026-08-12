/**
 * client の変異検査は module ごとの shard に分かれている。対象は二箇所にある。
 * Stryker config の scope と、workflow の matrix。どちらかから漏れた module は
 * 誰にも検査されないまま緑になるので、実行するより先にここで気付けるようにする。
 */
import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { parse } from "yaml";
import { mutationScopes } from "../stryker.client.config.mjs";

const projectRoot = join(import.meta.dir, "..");
const scopeNames = Object.keys(mutationScopes);

function clientModules(): string[] {
  const modules = join(projectRoot, "client/src/modules");
  return readdirSync(modules, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .filter((name) =>
      readdirSync(join(modules, name), { withFileTypes: true }).some(
        (child) =>
          child.isDirectory() &&
          (child.name === "domain" || child.name === "application"),
      ),
    )
    .sort();
}

describe("client 変異検査の対象", () => {
  test("domain か application を持つ module はすべて shard に入っている", () => {
    expect(scopeNames.sort()).toEqual(clientModules());
  });

  test("scope の glob は自分の module だけを指す", () => {
    for (const [scope, globs] of Object.entries(mutationScopes)) {
      for (const glob of globs) {
        expect(glob, `${scope}: ${glob}`).toStartWith(
          `client/src/modules/${scope}/`,
        );
      }
    }
  });

  test("workflow の matrix は scope と一致する", () => {
    const workflow = parse(
      readFileSync(join(projectRoot, ".github/workflows/mutation.yml"), "utf8"),
    ) as { jobs: { client: { strategy: { matrix: { scope: string[] } } } } };
    const sharded = [...workflow.jobs.client.strategy.matrix.scope].sort();

    expect(sharded).toEqual(scopeNames.sort());
  });
});
