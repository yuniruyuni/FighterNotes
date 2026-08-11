import { useCallback, useEffect, useMemo, useRef } from "react";

/**
 * URL を変えずに画面内の位置を browser history へ積む。
 *
 * entry には積んだ深さを一緒に残すため、再mountしても離脱時にまとめて戻せる。
 * key ごとに独立した stack として扱い、別の key が残した entry は無視する。
 */
interface StackedEntry<T> {
  depth: number;
  value: T;
}

export interface HistoryStack<T> {
  /** 現在位置の1つ上へ value を積む。 */
  push(value: T): void;
  /** この stack で積んだ entry をすべて戻る。戻り終えたら onSettled を呼ぶ。 */
  unwind(onSettled?: () => void): void;
}

export function readHistoryStack<T>(key: string): T | null {
  return stackedEntry<T>(key, window.history.state)?.value ?? null;
}

export function useHistoryStack<T>(
  key: string,
  onRestore: (value: T | null) => void,
): HistoryStack<T> {
  const restore = useRef(onRestore);

  useEffect(() => {
    restore.current = onRestore;
  }, [onRestore]);

  useEffect(() => {
    const handlePopState = (event: PopStateEvent) => {
      restore.current(stackedEntry<T>(key, event.state)?.value ?? null);
    };
    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, [key]);

  const push = useCallback(
    (value: T) => {
      const state = window.history.state;
      const depth = (stackedEntry<T>(key, state)?.depth ?? 0) + 1;
      window.history.pushState(
        { ...asRecord(state), [key]: { depth, value } },
        "",
        window.location.href,
      );
    },
    [key],
  );

  const unwind = useCallback(
    (onSettled?: () => void) => {
      const depth = stackedEntry<T>(key, window.history.state)?.depth ?? 0;
      if (depth <= 0) return;
      if (onSettled) {
        const settle = () => {
          window.removeEventListener("popstate", settle);
          onSettled();
        };
        window.addEventListener("popstate", settle);
      }
      window.history.go(-depth);
    },
    [key],
  );

  return useMemo(() => ({ push, unwind }), [push, unwind]);
}

function stackedEntry<T>(key: string, state: unknown): StackedEntry<T> | null {
  const entry = asRecord(state)[key];
  if (!isRecord(entry) || typeof entry.depth !== "number") return null;
  return { depth: entry.depth, value: entry.value as T };
}

function asRecord(value: unknown): Record<string, unknown> {
  return isRecord(value) ? value : {};
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
