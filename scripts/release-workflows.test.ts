import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const projectRoot = join(import.meta.dir, "..");
const workflowDirectory = join(projectRoot, ".github/workflows");
const workflowFiles = [
  ...new Bun.Glob("*.yml").scanSync({ cwd: workflowDirectory }),
].sort();
const digestPattern = /@sha256:[0-9a-f]{64}$/;

function read(relativePath: string): string {
  return readFileSync(join(projectRoot, relativePath), "utf8");
}

describe("release workflow safety contracts", () => {
  test("third-party actions use full commit SHAs with readable versions", () => {
    for (const file of workflowFiles) {
      const source = read(`.github/workflows/${file}`);
      for (const match of source.matchAll(/^\s*uses:\s*([^\s#]+)(.*)$/gm)) {
        const action = match[1];
        if (!action || action.startsWith("./")) continue;
        expect(action, `${file}: ${action}`).toMatch(/@[0-9a-f]{40}$/);
        expect(match[2]?.trim(), `${file}: ${action} version comment`).toMatch(
          /^#\s+\S+/,
        );
      }
    }
  });

  test("workflow service and release container images use immutable digests", () => {
    for (const file of workflowFiles) {
      const source = read(`.github/workflows/${file}`);
      for (const match of source.matchAll(/^\s*image:\s*([^\s#]+)/gm)) {
        expect(match[1], `${file}: ${match[1]}`).toMatch(digestPattern);
      }
    }

    for (const file of ["Dockerfile", "Dockerfile.migration"]) {
      const source = read(file);
      for (const match of source.matchAll(/^FROM\s+([^\s]+)/gm)) {
        expect(match[1], `${file}: ${match[1]}`).toMatch(digestPattern);
      }
    }
  });

  test("schema plan publishes output but restores the planner failure", () => {
    const source = read(".github/workflows/schema-plan.yml");
    expect(source).not.toMatch(/plan_output\.txt 2>&1\s*\|\|\s*true/);
    expect(source).toContain("PLAN_EXIT_CODE=$?");
    expect(source).toContain("> plan_exit_code.txt");
    expect(source).toContain("Enforce successful schema plan");
  });

  test("schema plan comments are best-effort without weakening enforcement", () => {
    const source = read(".github/workflows/schema-plan.yml");
    const commentStart = source.indexOf("- name: Comment plan on PR");
    const enforceStart = source.indexOf(
      "- name: Enforce successful schema plan",
    );
    const stopStart = source.indexOf("- name: Stop PostgreSQL");
    const commentStep = source.slice(commentStart, enforceStart);
    const enforceStep = source.slice(enforceStart, stopStart);

    expect(commentStart).toBeGreaterThan(-1);
    expect(enforceStart).toBeGreaterThan(commentStart);
    expect(stopStart).toBeGreaterThan(enforceStart);
    expect(commentStep).toContain('>> "$GITHUB_STEP_SUMMARY"');
    expect(commentStep).toContain("publish_plan_comment()");
    expect(commentStep).toContain("if ! publish_plan_comment; then");
    expect(commentStep).toContain("::warning title=Schema plan comment::");
    expect(commentStep).not.toContain("exit 1");
    expect(enforceStep).toContain("plan_output.txt");
    expect(enforceStep).toContain("plan_exit_code.txt");
    expect(enforceStep).toContain("exit 1");
  });

  test("schema plan is a stable required check for every pull request", () => {
    const source = read(".github/workflows/schema-plan.yml");
    const schemaCondition =
      "if: steps.changes.outputs.schema_changed == 'true'";
    const alwaysSchemaCondition =
      "if: always() && steps.changes.outputs.schema_changed == 'true'";

    expect(source).toMatch(/^name: Schema Plan$/m);
    expect(source).toMatch(/^on:\n {2}pull_request: \{\}$/m);
    expect(source).toMatch(/^ {2}plan:\n {4}runs-on: ubuntu-latest$/m);
    expect(source).not.toContain("    paths:");
    expect(source).not.toContain("    services:");
    expect(source).toContain("Detect schema-impacting changes");
    expect(source).toContain("github.event.pull_request.base.sha");
    expect(source).toContain("github.event.pull_request.head.sha");
    expect(source).toContain(
      'git diff --name-only --no-renames -z "$BASE_SHA" "$HEAD_SHA"',
    );
    expect(source).toContain(
      "schema/*|.pgschemaignore|Dockerfile.migration|bin/migrate.sh",
    );
    expect(source).toContain(
      "- name: No schema changes\n" +
        "        if: steps.changes.outputs.schema_changed != 'true'",
    );
    expect(source).toContain(
      'echo "No schema-impacting changes; schema plan is a successful no-op."',
    );

    for (const step of [
      "Build migration toolchain",
      "Start PostgreSQL",
      "Prepare application role",
      "Apply base branch schema (current state)",
      "Plan PR schema changes (diff from base)",
    ]) {
      expect(source).toContain(`- name: ${step}\n        ${schemaCondition}`);
    }
    for (const step of [
      "Comment plan on PR",
      "Enforce successful schema plan",
      "Stop PostgreSQL",
    ]) {
      expect(source).toContain(
        `- name: ${step}\n        ${alwaysSchemaCondition}`,
      );
    }
  });

  test("デプロイは途中で打ち切らず、公開先の健全性まで見る", () => {
    const deploy = read(".github/workflows/deploy-yunirun.yml");
    // 中断は blue/green の入れ替えを中途半端な状態で止めうる。
    expect(deploy).toContain("cancel-in-progress: false");
    // 入れ替えが済んだことを外から確かめる。
    expect(deploy).toContain("https://fighter.yuniruyuni.net/health");
  });

  test("デプロイ job に environment を付けない", () => {
    // environment を使うと OIDC の sub が ...:environment:<name> に変わり、
    // VPS 側の認可と一致しなくなって ssh に入れなくなる。
    const deploy = read(".github/workflows/deploy-yunirun.yml");
    expect(deploy).not.toMatch(/^\s+environment:/m);
  });

  test("GHCR のトークンは argv ではなく stdin で渡す", () => {
    // argv に載せると ps から見える。
    const deploy = read(".github/workflows/deploy-yunirun.yml");
    expect(deploy).toContain("--arg token");
    expect(deploy).not.toMatch(/ssh .*\$GHCR_TOKEN/);
  });

  test("Cloudflare client IP を信じる条件が宣言に揃っている", () => {
    // CF-Connecting-IP を信じてよいのは Cloudflare 経由でしか到達できない
    // ときだけ。VPS ではコンテナが loopback にだけ束縛され、その手前に
    // HAProxy と cloudflared がいることでこれが成り立つ。アプリ側は公開 URL
    // が HTTPS であることを起動時に確かめるので、両方が揃っている必要がある。
    const manifest = read("yunirun.jsonc");
    expect(manifest).toContain('"TRUST_CLOUDFLARE_CONNECTING_IP": "true"');
    expect(manifest).toContain(
      '"PUBLIC_BASE_URL": "https://fighter.yuniruyuni.net"',
    );
  });

  test("cleanup は定期実行として宣言されている", () => {
    // Cloud Scheduler の 23 2 * * * (JST) を systemd timer へ移した。
    // schedule が抜けると、期限切れの共有データが溜まり続ける。
    const manifest = read("yunirun.jsonc");
    expect(manifest).toContain('"cleanup": {');
    expect(manifest).toContain('"schedule": "02:23"');
    expect(manifest).toContain('"--batch=cleanup"');
  });
});
