import type { ReactNode } from "react";
import { CopyLine } from "./CopyLine";

/** Key/value row used inside a Collapsible.
 *
 * Always rendered — the row is structural, not contingent on having a
 * value. Empty/missing values render as a dim em-dash so the page layout
 * stays stable while detail data loads asynchronously. Pass the value
 * directly as `detail?.x`; undefined and empty string both render as the
 * placeholder.
 *
 * Color rule: pass `mono` for engine-returned data the user might
 * select-copy — refs, IDs, names, paths, timestamps, code tokens, env
 * values, network/volume identifiers. The amber treatment matches
 * foundry's "ID-as-link" convention. Omit `mono` for human classifications:
 * counts ("3 containers"), enums ("unless-stopped"), yes/no, formatted
 * summaries ("12 MB"). When in doubt, ask: would you copy this string and
 * paste it elsewhere? Yes → `mono`. No → plain.
 *
 * Mono values are tap-to-copy and horizontally scrollable. See
 * `<CopyLine />` for the implementation; long-press still triggers the
 * browser's native selection menu so the user can grab a substring. */
export function Row({
  label,
  value,
  mono,
  right,
}: {
  label: string;
  value?: string;
  mono?: boolean;
  right?: ReactNode;
}) {
  return (
    <div className="flex gap-3 items-baseline select-text">
      <div className="w-24 shrink-0 overflow-x-auto no-scrollbar scroll-fade-r">
        <span className="text-xs text-[var(--text-tertiary)] whitespace-nowrap">{label}</span>
      </div>
      {right ? (
        <div className="flex-1 min-w-0">{right}</div>
      ) : !value ? (
        <span className="flex-1 min-w-0 text-xs text-[var(--text-tertiary)]">—</span>
      ) : (
        <div className="flex-1 min-w-0">
          <CopyLine value={value} mono={mono} />
        </div>
      )}
    </div>
  );
}
