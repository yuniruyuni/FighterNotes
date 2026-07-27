export function deepFreeze<T>(value: T): T {
  return deepFreezeSeen(value, new WeakSet<object>());
}

function deepFreezeSeen<T>(value: T, seen: WeakSet<object>): T {
  if (typeof value !== "object" || value === null) return value;
  if (seen.has(value)) return value;

  seen.add(value);
  Object.freeze(value);
  for (const child of Object.values(value)) deepFreezeSeen(child, seen);
  return value;
}
