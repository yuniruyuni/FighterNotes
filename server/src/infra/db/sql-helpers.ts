import type { Comp } from "../../models/common";
import { isCompLogical } from "../../models/common";
import { type SQLFragment, sql } from "./sql";

export function compToSQL<T>(
  spec: Comp<T>,
  convert: (value: T) => SQLFragment,
): SQLFragment {
  if (isCompLogical(spec)) {
    switch (spec.type) {
      case "and": {
        if (spec.children.length === 0) return sql.empty();
        const children = spec.children.map((child) =>
          compToSQL(child, convert),
        );
        return sql`(${sql.join(children, " AND ")})`;
      }
      case "or": {
        if (spec.children.length === 0) return sql`1=0`;
        const children = spec.children.map((child) =>
          compToSQL(child, convert),
        );
        return sql`(${sql.join(children, " OR ")})`;
      }
      case "not":
        return sql`NOT (${compToSQL(spec.child, convert)})`;
    }
  }

  return convert(spec as T);
}

export function dateFromSQL(value: string | Date): Date {
  return value instanceof Date ? value : new Date(value);
}

export function assertNever(value: never): never {
  throw new Error(`Unhandled specification: ${JSON.stringify(value)}`);
}
