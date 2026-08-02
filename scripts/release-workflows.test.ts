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

    for (const file of [
      "cloudrun.yaml",
      "cloudrun-job.yaml",
      "cloudrun-cleanup-job.yaml",
    ]) {
      const source = read(file);
      for (const match of source.matchAll(/^\s*image:\s*([^\s#]+)/gm)) {
        const image = match[1];
        if (!image || image === "IMAGE_PLACEHOLDER") continue;
        expect(image, `${file}: ${image}`).toMatch(digestPattern);
      }
      expect(source).not.toContain("cloudflare/cloudflared:latest");
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

  test("production release is CI-gated, digest-based, and non-canceling", () => {
    const deploy = read(".github/workflows/deploy.yml");
    const build = read(".github/workflows/build-image.yml");
    expect(deploy).toMatch(/on:\n\s+workflow_run:/);
    expect(deploy).toContain(
      "github.event.workflow_run.conclusion == 'success'",
    );
    expect(deploy).toContain("github.event.workflow_run.head_sha");
    expect(deploy).toContain("group: fighter-production-release");
    expect(deploy).toContain("cancel-in-progress: false");
    expect(deploy).not.toContain("secrets: inherit");
    expect(deploy).toContain("artifact_image_digest");
    expect(deploy).toContain("environment: production");
    expect(deploy).toContain("https://fighter.yuniruyuni.net/health");
    expect(deploy).toContain("https://fighter.yuniruyuni.net/ready");
    expect(build).toContain("workflow_call:");
    expect(build).toContain("GCP_BUILDER_WORKLOAD_IDENTITY_PROVIDER:");
    expect(build).toContain("GCP_BUILDER_SERVICE_ACCOUNT:");
  });

  test("image builds pass only non-secret digests into the release job", () => {
    const deploy = read(".github/workflows/deploy.yml");
    const build = read(".github/workflows/build-image.yml");

    expect(build).toContain("Non-secret digest returned by Artifact Registry");
    expect(build).toContain(
      `artifact_image_digest: \${{ steps.push-artifact.outputs.digest }}`,
    );
    expect(build).toContain('echo "digest=$DIGEST" >> "$GITHUB_OUTPUT"');
    expect(build).toContain("Artifact Registry digest:");
    expect(build).not.toContain("artifact_image_ref");
    expect(build).not.toContain("ARTIFACT_IMAGE_REF");
    expect(build).not.toContain("outputs.image_ref");
    expect(build).not.toMatch(/image_ref=.*GITHUB_OUTPUT/);

    expect(deploy).toContain(
      `APP_IMAGE_DIGEST: \${{ needs.build-app.outputs.artifact_image_digest }}`,
    );
    expect(deploy).toContain(
      `MIGRATION_IMAGE_DIGEST: \${{ needs.build-migration.outputs.artifact_image_digest }}`,
    );
    expect(deploy).not.toContain("outputs.artifact_image_ref");
    expect(deploy).not.toMatch(/APP_IMAGE_REF:\s*\$\{\{/);
    expect(deploy).not.toMatch(/MIGRATION_IMAGE_REF:\s*\$\{\{/);
    expect(deploy).toContain("Validate and compose immutable image references");
    expect(deploy).toContain("^sha256:[0-9a-f]{64}$");
    expect(deploy).toContain(
      `IMAGE_REPOSITORY="\${GCP_REGION}-docker.pkg.dev/\${GCP_PROJECT_ID}/fighter"`,
    );
    expect(deploy).toContain("printf 'APP_IMAGE_REF=%s/%s@%s\\n'");
    expect(deploy).toContain("printf 'MIGRATION_IMAGE_REF=%s/%s@%s\\n'");
    expect(deploy).toContain('>> "$GITHUB_ENV"');
    expect(deploy).toContain("Application digest:");
    expect(deploy).toContain("Migration digest:");
    expect(deploy).not.toContain('echo "- Application: \\`$APP_IMAGE_REF\\`"');
    expect(deploy).not.toContain(
      'echo "- Migration: \\`$MIGRATION_IMAGE_REF\\`"',
    );
  });
});
