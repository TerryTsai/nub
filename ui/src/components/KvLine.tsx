import { CopyLine } from "./CopyLine";

/** A key=value spec line: tertiary mono key in a 96px column, scrollable
 * mono amber value to the right. The 96px width matches Row's label
 * column so KvLine and Row entries line up vertically across the detail
 * page; long user-provided keys (env vars, labels, mount paths) scroll
 * horizontally inside their column the same way values do.
 *
 * Both columns are tap-to-copy. The key copies just the key (useful for
 * grabbing an env var name on its own); the value copies `copyAs`
 * (default: `value`) — env-var sections pass `${k}=${v}` so a tap on
 * the value yields a paste-ready entry. */
export function KvLine({ k, v, copyAs }: { k: string; v: string; copyAs?: string }) {
  return (
    <div className="flex gap-3 items-baseline">
      <div className="w-24 shrink-0">
        <CopyLine value={k} dim />
      </div>
      <div className="flex-1 min-w-0">
        <CopyLine value={v} copyValue={copyAs} />
      </div>
    </div>
  );
}
