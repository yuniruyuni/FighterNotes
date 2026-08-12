import { useEffect, useRef } from "react";
import {
  shortcutActionForKey,
  type ViewerShortcutAction,
} from "./shortcuts.js";

/**
 * 表示中の画面がキー操作を受け取る。適用できた操作だけ既定動作を止める。
 *
 * 入力中のフォーム部品からはキーを奪わない。例外は再生位置の slider で、
 * 矢印は 1ms ではなく 1 フレーム移動を担う。ボタンやリンクへ focus がある間は
 * Space を渡さない。本来の「押す」を奪わないためで、再生は K で代替できる。
 */
export function useShortcutKeys(
  active: boolean,
  apply: (action: ViewerShortcutAction) => boolean,
): void {
  const latest = useRef(apply);

  useEffect(() => {
    latest.current = apply;
  }, [apply]);

  useEffect(() => {
    if (!active) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (holdsKeys(event.target, event.key)) return;
      const action = shortcutActionForKey(event.key, {
        ctrl: event.ctrlKey,
        shift: event.shiftKey,
      });
      if (!action || !latest.current(action)) return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [active]);
}

function holdsKeys(target: EventTarget | null, key: string): boolean {
  if (
    target instanceof HTMLSelectElement ||
    target instanceof HTMLTextAreaElement
  ) {
    return true;
  }
  if (target instanceof HTMLInputElement) return target.type !== "range";
  if (key !== " ") return false;
  return (
    target instanceof HTMLButtonElement || target instanceof HTMLAnchorElement
  );
}
