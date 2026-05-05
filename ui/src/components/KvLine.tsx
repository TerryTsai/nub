import { CopyLine } from "./CopyLine";

/** A key=value spec line: tertiary mono key in its natural column,
 * scrollable mono amber value to the right. Tap on the value copies
 * `copyAs` (default: `value`) — env-var sections pass `${k}=${v}` so a
 * tap yields a paste-ready entry.
 *
 * Unlike Row, the key column has no fixed width — env var and label
 * names run wider than the 96px Row label column more often than spec
 * keys do, and clipping them with ellipsis would lose information.
 * Within an env/labels/options section, all lines share the same
 * pattern; consistency is per-section, not across sections. */
export function KvLine({ k, v, copyAs }: { k: string; v: string; copyAs?: string }) {
  return (
    <div className="flex gap-3 items-baseline">
      <span className="text-xs text-[var(--text-tertiary)] mono shrink-0">{k}</span>
      <div className="flex-1 min-w-0">
        <CopyLine value={v} copyValue={copyAs} />
      </div>
    </div>
  );
}
