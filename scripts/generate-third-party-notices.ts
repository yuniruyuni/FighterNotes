import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import {
  basename,
  dirname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from "node:path";
import { fileURLToPath } from "node:url";
import {
  init as checkNpmLicenses,
  type ModuleInfo,
  type ModuleInfos,
} from "license-checker-rseidelsohn";
import {
  canonicalizeSpdxExpression,
  extractCopyrightNotices,
  validateLicenseExpression,
} from "./license-policy";

interface PackageJson {
  name?: string;
  version?: string;
  license?: string;
  repository?: string | { url?: string };
  homepage?: string;
  dependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
}

interface CargoAboutPackage {
  authors: string[];
  license_file: string | null;
  manifest_path: string;
  name: string;
  license: string | null;
  repository: string | null;
  source: string | null;
  version: string;
}

interface CargoAboutOutput {
  crates: Array<{
    license: string;
    package: CargoAboutPackage;
  }>;
}

type LicenseDocumentOrigin =
  | "canonical-fallback"
  | "package-file"
  | "reviewed-override";

interface LicenseDocument {
  name: string;
  origin: LicenseDocumentOrigin;
  text: string;
}

interface NpmComponentOverride {
  copyrights?: string[];
  license?: string;
  licenseDocument?: string;
  source?: string;
}

interface Component {
  copyrights: string[];
  documents: LicenseDocument[];
  ecosystem: "Cargo" | "npm";
  license: string;
  name: string;
  source: string;
  targets: Set<string>;
  version: string;
}

interface IndexedLicenseDocument {
  hash: string;
  names: Set<string>;
  text: string;
  uses: Array<{ component: string; document: string }>;
}

interface DistributionMaterial {
  category:
    | "Data/model"
    | "Font"
    | "Generated output"
    | "Icon"
    | "Image"
    | "Runtime/platform";
  location: string;
  name: string;
  reference: string | null;
  treatment: string;
}

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..");
const generatedSource = join(
  projectRoot,
  "client/src/generated/third-party-licenses.ts",
);
const noticeFile = join(projectRoot, "THIRD_PARTY_NOTICES.md");
const checkOnly = process.argv.includes("--check");
const cargoAboutVersion = "0.9.1";
const npmLicenseCheckerVersion = "5.0.1";

const fallbackLicenseDocuments = new Map([
  ["Unlicense", join(projectRoot, "scripts/license-overrides/Unlicense.md")],
]);

const npmComponentOverrides = new Map<string, NpmComponentOverride>([
  [
    "@hono/trpc-server@0.4.2",
    {
      licenseDocument: join(
        projectRoot,
        "scripts/license-overrides/hono-trpc-server-MIT.md",
      ),
    },
  ],
  [
    "pg-types@2.2.0",
    {
      licenseDocument: join(
        projectRoot,
        "scripts/license-overrides/pg-types-MIT.md",
      ),
    },
  ],
  [
    "pgpass@1.0.5",
    {
      licenseDocument: join(
        projectRoot,
        "scripts/license-overrides/pgpass-MIT.md",
      ),
    },
  ],
]);
const usedNpmComponentOverrides = new Set<string>();
const usedLicenseOverridePaths = new Set<string>();

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(path, "utf8")) as T;
}

function verifyNpmLicenseCheckerVersion(): void {
  const installed = readJson<PackageJson>(
    join(projectRoot, "node_modules/license-checker-rseidelsohn/package.json"),
  ).version;
  if (installed !== npmLicenseCheckerVersion) {
    throw new Error(
      `Expected license-checker-rseidelsohn ${npmLicenseCheckerVersion}, found ${installed ?? "nothing"}`,
    );
  }
}

function sha256(path: string): string {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function sha256Text(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function normalizeText(value: string): string {
  return `${value.replaceAll("\r\n", "\n").trim()}\n`;
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function packagePath(packageName: string, nodeModules: string): string {
  return join(nodeModules, ...packageName.split("/"));
}

function findPackage(
  packageName: string,
  searchPaths: readonly string[],
): string | undefined {
  for (const nodeModules of searchPaths) {
    const candidate = packagePath(packageName, nodeModules);
    if (existsSync(join(candidate, "package.json"))) {
      return realpathSync(candidate);
    }
  }
  return undefined;
}

function containingNodeModules(packageDir: string, name: string): string {
  return name.startsWith("@")
    ? dirname(dirname(packageDir))
    : dirname(packageDir);
}

function repositoryUrl(
  repository: PackageJson["repository"],
  homepage: string | undefined,
  name: string,
  version: string,
): string {
  const raw =
    (typeof repository === "string" ? repository : repository?.url) ??
    homepage ??
    `https://www.npmjs.com/package/${encodeURIComponent(name)}/v/${encodeURIComponent(version)}`;
  if (/^[\w.-]+\/[\w.-]+$/.test(raw)) {
    return `https://github.com/${raw}`;
  }
  const normalized = raw
    .replace(/^git\+/, "")
    .replace(/^git:\/\/github\.com\//, "https://github.com/")
    .replace(/^ssh:\/\/git@github\.com\//, "https://github.com/")
    .replace(/^git@github\.com:/, "https://github.com/")
    .replace(/\.git$/, "");
  const url = new URL(normalized);
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error(`${name}@${version} has a non-HTTP source URL`);
  }
  return url.toString().replace(/\/$/, "");
}

function isLicenseNoticeName(name: string): boolean {
  return /^(licen[cs]e|copying|copyright|notice)(?:[._-].*)?$/i.test(name);
}

function licenseFileNames(packageDir: string): string[] {
  return readdirSync(packageDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && isLicenseNoticeName(entry.name))
    .map((entry) => entry.name)
    .sort(compareText);
}

function isPathInsideDirectory(path: string, directory: string): boolean {
  const child = realpathSync(path);
  const parent = realpathSync(directory);
  const relativePath = relative(parent, child);
  return (
    relativePath.length > 0 &&
    relativePath !== ".." &&
    !relativePath.startsWith(`..${sep}`) &&
    !isAbsolute(relativePath)
  );
}

function deduplicateLicenseDocuments(
  documents: readonly LicenseDocument[],
): LicenseDocument[] {
  const byText = new Map<
    string,
    { names: Set<string>; origin: LicenseDocumentOrigin; text: string }
  >();
  for (const document of documents) {
    const hash = sha256Text(document.text);
    const existing = byText.get(hash);
    if (existing) {
      existing.names.add(document.name);
      continue;
    }
    byText.set(hash, {
      names: new Set([document.name]),
      origin: document.origin,
      text: document.text,
    });
  }
  return [...byText.values()]
    .map((document) => ({
      name: [...document.names].sort(compareText).join(" / "),
      origin: document.origin,
      text: document.text,
    }))
    .sort(
      (left, right) =>
        compareText(left.name, right.name) ||
        compareText(left.origin, right.origin),
    );
}

function inspectNpmPackage(
  packageDir: string,
  component: string,
): Promise<ModuleInfo> {
  return new Promise((resolvePromise, rejectPromise) => {
    checkNpmLicenses(
      {
        start: packageDir,
        production: true,
        direct: true,
        customFormat: {
          copyright: "",
          licenseFile: "",
          licenseText: "",
          licenses: "",
          name: "",
          noticeFile: "",
          version: "",
        },
      },
      (error: Error | undefined, result: ModuleInfos) => {
        if (error) {
          rejectPromise(
            new Error(
              `license-checker failed for ${component}: ${error.message}`,
            ),
          );
          return;
        }
        const inspected =
          result[component] ??
          Object.values(result).find(
            (candidate) =>
              `${candidate.name}@${candidate.version}` === component,
          );
        if (!inspected) {
          rejectPromise(
            new Error(
              `license-checker did not return metadata for ${component}`,
            ),
          );
          return;
        }
        resolvePromise(inspected);
      },
    );
  });
}

function inspectedLicenseExpression(
  inspected: ModuleInfo,
  component: string,
): string {
  const values = Array.isArray(inspected.licenses)
    ? inspected.licenses
    : inspected.licenses
      ? [inspected.licenses]
      : [];
  if (values.some((value) => value.trim().endsWith("*"))) {
    throw new Error(
      `license-checker only guessed the license for ${component}; add a version-pinned reviewed override`,
    );
  }
  const normalized = values.map((value) => value.trim());
  if (
    normalized.length === 0 ||
    normalized.some(
      (value) => value.length === 0 || value.toUpperCase() === "UNKNOWN",
    )
  ) {
    throw new Error(
      `license-checker found no declared license for ${component}`,
    );
  }
  return normalized.join(" OR ");
}

function sameSpdxExpression(left: string, right: string): boolean {
  validateLicenseExpression(left, "license-checker result");
  validateLicenseExpression(right, "package metadata");
  return canonicalizeSpdxExpression(left) === canonicalizeSpdxExpression(right);
}

function npmLicenseDocuments(
  packageDir: string,
  license: string,
  component: string,
  inspected: ModuleInfo,
): LicenseDocument[] {
  const documents: LicenseDocument[] = [];
  const packagePaths = new Set<string>();
  for (const name of licenseFileNames(packageDir)) {
    const path = join(packageDir, name);
    const realPath = realpathSync(path);
    if (!isPathInsideDirectory(realPath, packageDir)) {
      throw new Error(
        `${component} license file resolves outside its package: ${path}`,
      );
    }
    packagePaths.add(realPath);
  }
  for (const inspectedPath of [inspected.licenseFile, inspected.noticeFile]) {
    if (
      !inspectedPath ||
      !existsSync(inspectedPath) ||
      !isLicenseNoticeName(basename(inspectedPath)) ||
      !isPathInsideDirectory(inspectedPath, packageDir)
    ) {
      continue;
    }
    packagePaths.add(realpathSync(inspectedPath));
  }
  for (const path of [...packagePaths].sort(compareText)) {
    documents.push({
      name: basename(path),
      origin: "package-file",
      text: normalizeText(readFileSync(path, "utf8")),
    });
  }

  if (documents.length === 0) {
    const componentDocument =
      npmComponentOverrides.get(component)?.licenseDocument;
    if (componentDocument) {
      usedLicenseOverridePaths.add(componentDocument);
      documents.push({
        name: "Reviewed license notice",
        origin: "reviewed-override",
        text: normalizeText(readFileSync(componentDocument, "utf8")),
      });
    }
  }
  if (documents.length === 0) {
    const fallback = fallbackLicenseDocuments.get(license);
    if (fallback) {
      usedLicenseOverridePaths.add(fallback);
      documents.push({
        name: `${license} license text`,
        origin: "canonical-fallback",
        text: normalizeText(readFileSync(fallback, "utf8")),
      });
    }
  }
  if (documents.length === 0) {
    throw new Error(
      `${component} declares ${license} but has no package-owned license notice or reviewed override`,
    );
  }

  return deduplicateLicenseDocuments(documents);
}

function validateLicenseOverrides(): void {
  const configured = new Set([
    ...fallbackLicenseDocuments.values(),
    ...[...npmComponentOverrides.values()]
      .map((override) => override.licenseDocument)
      .filter((path): path is string => path !== undefined),
  ]);
  const directory = join(projectRoot, "scripts/license-overrides");
  const registeredFiles = filesUnder(directory).filter(
    (path) => basename(path) !== "README.md",
  );
  for (const path of configured) {
    if (!existsSync(path)) {
      throw new Error(`Configured license override is missing: ${path}`);
    }
    if (!usedLicenseOverridePaths.has(path)) {
      throw new Error(`Configured license override is unused: ${path}`);
    }
  }
  const unusedComponentOverrides = [...npmComponentOverrides.keys()].filter(
    (component) => !usedNpmComponentOverrides.has(component),
  );
  if (unusedComponentOverrides.length > 0) {
    throw new Error(
      `Configured npm metadata overrides are unused: ${unusedComponentOverrides.join(", ")}`,
    );
  }
  const unregistered = registeredFiles.filter((path) => !configured.has(path));
  if (unregistered.length > 0) {
    throw new Error(
      `License override files are not registered: ${unregistered.join(", ")}`,
    );
  }
}

function bunLockedPackages(): Set<string> {
  const parsed = Bun.JSONC.parse(
    readFileSync(join(projectRoot, "bun.lock"), "utf8"),
  ) as {
    packages?: Record<string, [string, ...unknown[]]>;
  };
  return new Set(
    Object.values(parsed.packages ?? {})
      .map(([resolved]) => resolved)
      .filter((resolved) => !resolved.includes("@workspace:")),
  );
}

async function collectNpmComponents(): Promise<Component[]> {
  const locked = bunLockedPackages();
  const components = new Map<
    string,
    Component & { packageDir: string; visitedTargets: Set<string> }
  >();
  const workspaceNodeModules = [
    join(projectRoot, "client/node_modules"),
    join(projectRoot, "server/node_modules"),
    join(projectRoot, "node_modules"),
  ];

  const visit = async (
    dependency: string,
    target: "browser" | "server",
    searchPaths: readonly string[],
    optional: boolean,
  ): Promise<void> => {
    const packageDir = findPackage(dependency, searchPaths);
    if (!packageDir) {
      if (optional) return;
      throw new Error(
        `Could not resolve runtime dependency ${dependency} for ${target}`,
      );
    }
    const metadata = readJson<PackageJson>(join(packageDir, "package.json"));
    if (!metadata.name || !metadata.version) {
      throw new Error(`${packageDir}/package.json lacks name or version`);
    }
    const lockedId = `${metadata.name}@${metadata.version}`;
    if (!locked.has(lockedId)) {
      throw new Error(`${lockedId} is installed but is not pinned by bun.lock`);
    }
    const override = npmComponentOverrides.get(lockedId);
    if (override) usedNpmComponentOverrides.add(lockedId);
    const license = metadata.license ?? override?.license;
    if (!license) {
      throw new Error(
        `${lockedId} has no license metadata or reviewed manual override`,
      );
    }
    validateLicenseExpression(license, lockedId);

    const key = `npm:${lockedId}`;
    let component = components.get(key);
    if (!component) {
      const inspected = await inspectNpmPackage(packageDir, lockedId);
      const detectedLicense = inspectedLicenseExpression(inspected, lockedId);
      if (!sameSpdxExpression(detectedLicense, license)) {
        throw new Error(
          `${lockedId} license-checker result (${detectedLicense}) does not match package metadata (${license})`,
        );
      }
      const documents = npmLicenseDocuments(
        packageDir,
        license,
        lockedId,
        inspected,
      );
      component = {
        copyrights: override?.copyrights ?? extractCopyrightNotices(documents),
        documents,
        ecosystem: "npm",
        name: metadata.name,
        version: metadata.version,
        license,
        source:
          override?.source ??
          repositoryUrl(
            metadata.repository,
            metadata.homepage,
            metadata.name,
            metadata.version,
          ),
        targets: new Set(),
        packageDir,
        visitedTargets: new Set(),
      };
      components.set(key, component);
    }
    component.targets.add(target);
    if (component.visitedTargets.has(target)) return;
    component.visitedTargets.add(target);

    const localNodeModules = containingNodeModules(
      component.packageDir,
      component.name,
    );
    const nestedSearchPaths = [
      localNodeModules,
      ...searchPaths,
      ...workspaceNodeModules,
    ];
    for (const child of Object.keys(metadata.dependencies ?? {}).sort()) {
      await visit(child, target, nestedSearchPaths, false);
    }
    for (const child of Object.keys(
      metadata.optionalDependencies ?? {},
    ).sort()) {
      await visit(child, target, nestedSearchPaths, true);
    }
  };

  for (const target of ["client", "server"] as const) {
    const metadata = readJson<PackageJson>(
      join(projectRoot, target, "package.json"),
    );
    const distributionTarget = target === "client" ? "browser" : "server";
    for (const dependency of Object.keys(metadata.dependencies ?? {}).sort()) {
      await visit(
        dependency,
        distributionTarget,
        [
          join(projectRoot, target, "node_modules"),
          join(projectRoot, "node_modules"),
        ],
        false,
      );
    }
  }

  return [...components.values()].map(
    ({ packageDir: _, visitedTargets: __, ...component }) => component,
  );
}

function cargoAboutOutput(): CargoAboutOutput {
  const versionResult = Bun.spawnSync({
    cmd: ["cargo", "about", "--version"],
    cwd: projectRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (versionResult.exitCode !== 0) {
    throw new Error(
      `cargo-about ${cargoAboutVersion} is required. Install it with: cargo install cargo-about --version ${cargoAboutVersion} --locked --features cli`,
    );
  }
  const installedVersion = new TextDecoder()
    .decode(versionResult.stdout)
    .trim();
  if (installedVersion !== `cargo-about ${cargoAboutVersion}`) {
    throw new Error(
      `Expected cargo-about ${cargoAboutVersion}, found ${installedVersion}`,
    );
  }

  const result = Bun.spawnSync({
    cmd: [
      "cargo",
      "about",
      "generate",
      "--format",
      "json",
      "--manifest-path",
      join(projectRoot, "crates/wasm-bridge/Cargo.toml"),
      "--config",
      join(projectRoot, "about.toml"),
      "--frozen",
      "--fail",
    ],
    cwd: projectRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(new TextDecoder().decode(result.stderr));
  }
  return JSON.parse(
    new TextDecoder().decode(result.stdout),
  ) as CargoAboutOutput;
}

function cargoLicenseDocuments(
  packageValue: CargoAboutPackage,
  component: string,
): LicenseDocument[] {
  if (!existsSync(packageValue.manifest_path)) {
    throw new Error(
      `${component} Cargo manifest is missing: ${packageValue.manifest_path}`,
    );
  }
  const packageDir = dirname(realpathSync(packageValue.manifest_path));
  const paths = new Set(
    licenseFileNames(packageDir).map((name) =>
      realpathSync(join(packageDir, name)),
    ),
  );
  if (packageValue.license_file) {
    const configuredPath = isAbsolute(packageValue.license_file)
      ? packageValue.license_file
      : resolve(packageDir, packageValue.license_file);
    if (!existsSync(configuredPath)) {
      throw new Error(
        `${component} declares a missing Cargo license-file: ${configuredPath}`,
      );
    }
    paths.add(realpathSync(configuredPath));
  }
  for (const path of paths) {
    if (!isPathInsideDirectory(path, packageDir)) {
      throw new Error(
        `${component} license file resolves outside its packaged crate: ${path}`,
      );
    }
  }
  if (paths.size === 0) {
    throw new Error(
      `${component} has no package-owned LICENSE, COPYING, COPYRIGHT, NOTICE, or Cargo license-file; add a version-pinned reviewed clarification`,
    );
  }
  return deduplicateLicenseDocuments(
    [...paths].sort(compareText).map((path) => ({
      name: basename(path),
      origin: "package-file",
      text: normalizeText(readFileSync(path, "utf8")),
    })),
  );
}

function collectCargoComponents(): Component[] {
  const inventory = cargoAboutOutput();
  const components: Component[] = [];
  for (const crate of inventory.crates) {
    const packageValue = crate.package;
    if (packageValue.source === null) continue;
    const componentName = `${packageValue.name}@${packageValue.version}`;
    const declaredLicense = packageValue.license ?? crate.license;
    if (!declaredLicense) {
      throw new Error(`${componentName} has no declared Cargo license`);
    }
    validateLicenseExpression(declaredLicense, componentName);
    const documents = cargoLicenseDocuments(packageValue, componentName);
    const extractedCopyrights = extractCopyrightNotices(documents);
    const copyrights = extractedCopyrights[0]?.startsWith(
      "Not separately stated",
    )
      ? packageValue.authors.map((author) => `Package author: ${author}`)
      : extractedCopyrights;
    components.push({
      copyrights: copyrights.length > 0 ? copyrights : extractedCopyrights,
      documents,
      ecosystem: "Cargo",
      name: packageValue.name,
      version: packageValue.version,
      license: declaredLicense,
      source:
        packageValue.repository ??
        `https://crates.io/crates/${encodeURIComponent(packageValue.name)}/${encodeURIComponent(packageValue.version)}`,
      targets: new Set(["browser/WASM"]),
    });
  }
  return components;
}

function sortComponents(components: Component[]): Component[] {
  return components.sort(
    (left, right) =>
      compareText(left.ecosystem, right.ecosystem) ||
      compareText(left.name, right.name) ||
      compareText(left.version, right.version),
  );
}

function indexLicenseDocuments(
  components: readonly Component[],
): IndexedLicenseDocument[] {
  const documents = new Map<string, IndexedLicenseDocument>();
  for (const component of components) {
    for (const document of component.documents) {
      const hash = sha256Text(document.text);
      const entry = documents.get(hash) ?? {
        hash,
        names: new Set<string>(),
        text: document.text,
        uses: [],
      };
      entry.names.add(document.name);
      entry.uses.push({
        component: `${component.name} ${component.version}`,
        document: document.name,
      });
      documents.set(hash, entry);
    }
  }
  return [...documents.values()]
    .map((document) => ({
      ...document,
      uses: document.uses.sort(
        (left, right) =>
          compareText(left.component, right.component) ||
          compareText(left.document, right.document),
      ),
    }))
    .sort((left, right) => compareText(left.hash, right.hash));
}

function filesUnder(directory: string): string[] {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesUnder(path) : [path];
  });
}

function assertFiles(
  directory: string,
  expectedRelativePaths: readonly string[],
): void {
  const actual = filesUnder(directory)
    .map((path) => path.slice(directory.length + 1))
    .sort(compareText);
  const expected = [...expectedRelativePaths].sort(compareText);
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `Distributed asset inventory for ${directory} is stale.\nExpected: ${expected.join(", ")}\nActual: ${actual.join(", ")}`,
    );
  }
}

function collectDistributionMaterials(
  components: readonly Component[],
): DistributionMaterial[] {
  assertFiles(join(projectRoot, "client/src/shared/assets/images"), [
    "fighter-notes-ogp.jpg",
  ]);
  const unexpectedFonts = filesUnder(
    join(projectRoot, "client/src/shared/assets"),
  ).filter((path) => /\.(?:otf|ttf|woff2?)$/i.test(path));
  if (unexpectedFonts.length > 0) {
    throw new Error(
      `Web font files are outside the reviewed inventory: ${unexpectedFonts.join(", ")}`,
    );
  }
  for (const relativePath of [
    "crates/frame-meter/src/data/meter_digits.bin",
    "crates/video-analyzer/data/attack_data.json",
    "crates/video-analyzer/data/frame_data.json",
    "crates/video-analyzer/data/manifest.json",
    "crates/video-analyzer/src/input_history/templates.rs",
    "crates/video-analyzer/src/round_start/fight_template.bin",
  ]) {
    if (!existsSync(join(projectRoot, relativePath))) {
      throw new Error(`Distributed data/model is missing: ${relativePath}`);
    }
  }
  const lucide = components.find(
    (component) =>
      component.ecosystem === "npm" && component.name === "lucide-react",
  );
  if (!lucide) {
    throw new Error(
      "lucide-react is used for icons but is absent from notices",
    );
  }
  const dockerfile = readFileSync(join(projectRoot, "Dockerfile"), "utf8");
  const bunVersions = new Set(
    [...dockerfile.matchAll(/^FROM oven\/bun:([^@\s]+)@sha256:/gm)].map(
      (match) => match[1],
    ),
  );
  if (bunVersions.size !== 1) {
    throw new Error(
      "Dockerfile must pin one consistent oven/bun version for license inventory",
    );
  }
  const bunVersion = [...bunVersions][0];
  const runtimeImage = [...dockerfile.matchAll(/^FROM (gcr\.io\/\S+)$/gm)].at(
    -1,
  )?.[1];
  if (!runtimeImage) {
    throw new Error("Dockerfile runtime base image was not found");
  }
  return [
    {
      category: "Font",
      name: "System font stacks",
      location: "CSS font-family declarations",
      treatment:
        "No font binary or external Web font is distributed; glyphs come from the user’s OS/browser.",
      reference: null,
    },
    {
      category: "Icon",
      name: `Lucide icons (${lucide.name} ${lucide.version})`,
      location: "Browser JavaScript bundle",
      treatment: `${lucide.license}; tracked in the component index and complete notices.`,
      reference: lucide.source,
    },
    {
      category: "Image",
      name: "Fighter Notes OGP image",
      location:
        "client/src/shared/assets/images/fighter-notes-ogp.jpg → /images/fighter-notes-ogp.jpg",
      treatment:
        "Project-specific media asset; not part of the third-party software inventory. Its reuse terms are documented in the source repository.",
      reference: "https://github.com/yuniruyuni/FighterNotes#ライセンス",
    },
    {
      category: "Data/model",
      name: "Analyzer data and recognition models",
      location:
        "crates/video-analyzer/data, input_history/templates.rs, round_start/fight_template.bin, and meter_digits.bin",
      treatment:
        "Project data/model; not part of the third-party software inventory. DATA_NOTICE applies.",
      reference: "/DATA_NOTICE.txt",
    },
    {
      category: "Generated output",
      name: "Browser bundles and WebAssembly analyzer",
      location:
        "index.js, analyzer-worker.js, wasm_bridge_bg.wasm, HTML, and CSS",
      treatment:
        "Bundled npm/Cargo portions retain the licenses listed below; the build does not replace those terms.",
      reference: "/THIRD_PARTY_NOTICES.txt",
    },
    {
      category: "Runtime/platform",
      name: `Bun ${bunVersion} runtime`,
      location: "Compiled server executable",
      treatment:
        "Embedded by bun build --compile. It is outside the bun.lock application-package inventory and remains subject to Bun’s runtime and linked-library notices.",
      reference: "https://bun.sh/docs/project/license",
    },
    {
      category: "Runtime/platform",
      name: "Distroless/Debian runtime image",
      location: runtimeImage,
      treatment:
        "Base-container operating-system packages are outside the application-package inventory and remain subject to their upstream notices.",
      reference: "https://github.com/GoogleContainerTools/distroless",
    },
  ];
}

function indent(text: string): string {
  return text
    .trimEnd()
    .split("\n")
    .map((line) => (line.length === 0 ? "" : `    ${line}`))
    .join("\n");
}

function licenseDocumentOriginLabel(origin: LicenseDocumentOrigin): string {
  switch (origin) {
    case "package-file":
      return "package file";
    case "reviewed-override":
      return "reviewed upstream override";
    case "canonical-fallback":
      return "canonical fallback";
  }
}

function renderNotice(
  components: readonly Component[],
  materials: readonly DistributionMaterial[],
  lockHashes: { bun: string; cargo: string },
): string {
  const documents = indexLicenseDocuments(components);

  const lines = [
    "# Third-Party Notices",
    "",
    "This file is generated. Do not edit it by hand.",
    "",
    "It covers production application-package dependency closures resolved",
    "from bun.lock and Cargo.lock for the browser bundle, WebAssembly analyzer,",
    "and server application. Development/test packages, the Bun runtime embedded",
    "in the compiled server, compiler toolchains, and base-container operating-",
    "system packages are outside this application-package inventory and remain",
    "subject to their upstream license notices; they are identified below.",
    "Identical license documents are stored once and referenced by hash.",
    "Declared SPDX expressions are preserved without selecting an OR alternative.",
    "Every packaged license document is retained for inspection; AND denotes terms",
    "that apply together, while OR preserves the alternatives offered upstream.",
    "",
    `- bun.lock SHA-256: \`${lockHashes.bun}\``,
    `- Cargo.lock SHA-256: \`${lockHashes.cargo}\``,
    `- npm scanner: \`license-checker-rseidelsohn ${npmLicenseCheckerVersion}\``,
    `- Cargo scanner: \`cargo-about ${cargoAboutVersion}\``,
    "",
    "## Distributed material inventory",
    "",
    ...materials.flatMap((material) => [
      `### ${material.name}`,
      "",
      `- Category: ${material.category}`,
      `- Location: ${material.location}`,
      `- Treatment: ${material.treatment}`,
      ...(material.reference ? [`- Reference: ${material.reference}`] : []),
      "",
    ]),
    "## Component index",
    "",
    ...components.map(
      (component) =>
        `- ${component.name} ${component.version} (${component.ecosystem}; ${[...component.targets].sort().join(", ")}) — ${component.license}`,
    ),
    "",
  ];
  for (const component of components) {
    lines.push(
      "---",
      "",
      `## ${component.name} ${component.version}`,
      "",
      `- Ecosystem: ${component.ecosystem}`,
      `- Used by: ${[...component.targets].sort().join(", ")}`,
      `- Declared license: ${component.license}`,
      `- Source: ${component.source}`,
      "- Copyright / attribution:",
      ...component.copyrights.map((notice) => `  - ${notice}`),
      "",
    );
    for (const document of component.documents) {
      const hash = sha256Text(document.text);
      lines.push(
        `- ${document.name} (${licenseDocumentOriginLabel(document.origin)}): [license text ${hash.slice(0, 12)}](#license-text-${hash})`,
      );
    }
    lines.push("");
  }
  lines.push("# License texts", "");
  for (const document of documents) {
    lines.push(
      "---",
      "",
      `## License text ${document.hash}`,
      "",
      `Document names: ${[...document.names].sort(compareText).join(", ")}`,
      "",
      "Referenced by:",
      "",
      ...document.uses.map((use) => `- ${use.component} — ${use.document}`),
      "",
      indent(document.text),
      "",
    );
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

function renderGeneratedSource(
  components: readonly Component[],
  materials: readonly DistributionMaterial[],
  lockHashes: { bun: string; cargo: string },
): string {
  const documents = indexLicenseDocuments(components);
  const summaries = components.map((component) => ({
    copyrights: component.copyrights,
    documents: component.documents.map((document) => ({
      id: `license-text-${sha256Text(document.text)}`,
      name: document.name,
      origin: document.origin,
    })),
    ecosystem: component.ecosystem,
    name: component.name,
    version: component.version,
    license: component.license,
    source: component.source,
    targets: [...component.targets].sort(),
  }));
  const documentSummaries = documents.map((document) => ({
    components: [
      ...new Set(document.uses.map((usage) => usage.component)),
    ].sort(compareText),
    id: `license-text-${document.hash}`,
    names: [...document.names].sort(compareText),
    text: document.text,
  }));
  return `// Generated by scripts/generate-third-party-notices.ts. Do not edit.
export const thirdPartyNoticeMetadata = ${JSON.stringify(
    {
      bunLockSha256: lockHashes.bun,
      cargoAboutVersion,
      cargoLockSha256: lockHashes.cargo,
      npmLicenseCheckerVersion,
    },
    null,
    2,
  )} as const;

export const distributedMaterials = ${JSON.stringify(materials, null, 2)} as const;

export const thirdPartyComponents = ${JSON.stringify(summaries, null, 2)} as const;

export const thirdPartyLicenseDocuments = ${JSON.stringify(documentSummaries, null, 2)} as const;
`;
}

function formatTypeScript(content: string): string {
  const result = Bun.spawnSync({
    cmd: [
      join(projectRoot, "node_modules/.bin/biome"),
      "format",
      "--stdin-file-path",
      generatedSource,
    ],
    cwd: projectRoot,
    stdin: new TextEncoder().encode(content),
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(new TextDecoder().decode(result.stderr));
  }
  return new TextDecoder().decode(result.stdout);
}

function verifyOrWrite(path: string, content: string): void {
  if (checkOnly) {
    if (!existsSync(path) || readFileSync(path, "utf8") !== content) {
      throw new Error(
        `${path} is stale. Run: bun scripts/generate-third-party-notices.ts`,
      );
    }
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

verifyNpmLicenseCheckerVersion();
const components = sortComponents([
  ...(await collectNpmComponents()),
  ...collectCargoComponents(),
]);
validateLicenseOverrides();
const materials = collectDistributionMaterials(components);
const lockHashes = {
  bun: sha256(join(projectRoot, "bun.lock")),
  cargo: sha256(join(projectRoot, "Cargo.lock")),
};
verifyOrWrite(noticeFile, renderNotice(components, materials, lockHashes));
verifyOrWrite(
  generatedSource,
  formatTypeScript(renderGeneratedSource(components, materials, lockHashes)),
);
console.log(
  `${checkOnly ? "Verified" : "Generated"} third-party notices for ${components.length} components.`,
);
