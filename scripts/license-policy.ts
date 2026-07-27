import parseSpdx from "spdx-expression-parse";
import policy from "./license-policy.json";

export interface ParsedSpdxExpression {
  readonly exceptions: ReadonlySet<string>;
  readonly licenses: ReadonlySet<string>;
}

interface LicensePolicy {
  readonly allowedLicenseExceptions: readonly string[];
  readonly allowedLicenseIdentifiers: readonly string[];
  readonly prohibitedLicenseIdentifiers: readonly string[];
}

type SpdxNode = ReturnType<typeof parseSpdx>;

function parseSpdxNode(expression: string): SpdxNode {
  if (expression.trim().length === 0) {
    throw new Error("SPDX license expression is empty");
  }
  try {
    return parseSpdx(expression);
  } catch (error) {
    throw new Error(
      `Invalid SPDX license expression: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

export function canonicalizeSpdxExpression(expression: string): string {
  const render = (node: SpdxNode): string => {
    if ("license" in node) {
      return `${node.license}${node.plus ? "+" : ""}${
        node.exception ? ` WITH ${node.exception}` : ""
      }`;
    }
    const operands: string[] = [];
    const collect = (candidate: SpdxNode): void => {
      if (
        "conjunction" in candidate &&
        candidate.conjunction === node.conjunction
      ) {
        collect(candidate.left);
        collect(candidate.right);
        return;
      }
      operands.push(render(candidate));
    };
    collect(node);
    return `(${operands.sort().join(` ${node.conjunction.toUpperCase()} `)})`;
  };
  return render(parseSpdxNode(expression));
}

export function parseSpdxExpression(expression: string): ParsedSpdxExpression {
  const parsed = parseSpdxNode(expression);
  const licenses = new Set<string>();
  const exceptions = new Set<string>();
  const visit = (node: SpdxNode): void => {
    if ("license" in node) {
      licenses.add(node.plus ? `${node.license}+` : node.license);
      if (node.exception) exceptions.add(node.exception);
      return;
    }
    visit(node.left);
    visit(node.right);
  };
  visit(parsed);

  return {
    exceptions,
    licenses,
  };
}

export const licensePolicy: LicensePolicy = policy;

function reviewedSet(values: readonly string[], label: string): Set<string> {
  const result = new Set(values);
  if (result.size !== values.length) {
    throw new Error(`${label} contains duplicate SPDX identifiers`);
  }
  return result;
}

const allowedLicenseIdentifiers = reviewedSet(
  licensePolicy.allowedLicenseIdentifiers,
  "allowedLicenseIdentifiers",
);
const allowedLicenseExceptions = reviewedSet(
  licensePolicy.allowedLicenseExceptions,
  "allowedLicenseExceptions",
);
const prohibitedLicenseIdentifiers = reviewedSet(
  licensePolicy.prohibitedLicenseIdentifiers,
  "prohibitedLicenseIdentifiers",
);
const contradictoryIdentifiers = [...allowedLicenseIdentifiers].filter(
  (identifier) => prohibitedLicenseIdentifiers.has(identifier),
);
if (contradictoryIdentifiers.length > 0) {
  throw new Error(
    `License policy both allows and prohibits: ${contradictoryIdentifiers.join(", ")}`,
  );
}
export function validateLicenseExpression(
  expression: string,
  component: string,
): ParsedSpdxExpression {
  if (
    expression.trim() === "UNLICENSED" ||
    expression.trim().startsWith("SEE LICENSE IN")
  ) {
    throw new Error(`${component} has unsupported license: ${expression}`);
  }
  let parsed: ParsedSpdxExpression;
  try {
    parsed = parseSpdxExpression(expression);
  } catch (error) {
    throw new Error(
      `${component} has an invalid SPDX license expression: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  const prohibited = [...parsed.licenses].filter((identifier) =>
    prohibitedLicenseIdentifiers.has(identifier),
  );
  if (prohibited.length > 0) {
    throw new Error(
      `${component} uses prohibited licenses: ${prohibited.join(", ")}`,
    );
  }
  const unknown = [...parsed.licenses].filter(
    (identifier) => !allowedLicenseIdentifiers.has(identifier),
  );
  const unknownExceptions = [...parsed.exceptions].filter(
    (identifier) => !allowedLicenseExceptions.has(identifier),
  );
  if (unknown.length > 0 || unknownExceptions.length > 0) {
    throw new Error(
      `${component} uses licenses outside the reviewed policy: ${[
        ...unknown,
        ...unknownExceptions,
      ].join(", ")}`,
    );
  }
  return parsed;
}

export function extractCopyrightNotices(
  documents: readonly { readonly text: string }[],
): string[] {
  const notices = new Set<string>();
  for (const document of documents) {
    for (const line of document.text.split("\n")) {
      const normalized = line.trim().replace(/\s+/g, " ");
      if (
        (/^Copyright(?:\s+\(c\)|\s+©|\s+\d{4}|\s*:)/i.test(normalized) ||
          /^©\s*\d{4}/.test(normalized)) &&
        !/\[(?:yyyy|year|name of copyright owner)\]/i.test(normalized)
      ) {
        notices.add(normalized);
      }
    }
  }
  return notices.size > 0
    ? [...notices]
    : [
        "Not separately stated in the distributed files; see the complete license and notice text.",
      ];
}
