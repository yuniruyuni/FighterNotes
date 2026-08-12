import type { ShortcutHelpEntry } from "./shortcuts.js";

/** キー操作は覚えていないと使われない。画面の下に出しておく。 */
export function ShortcutLegend({
  entries,
}: {
  entries: readonly ShortcutHelpEntry[];
}) {
  return (
    <dl className="shortcut-legend" aria-label="キーボード操作">
      {entries.map((entry) => (
        <div key={entry.keys}>
          <dt>
            <kbd>{entry.keys}</kbd>
          </dt>
          <dd>{entry.label}</dd>
        </div>
      ))}
    </dl>
  );
}
