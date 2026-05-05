import type { ReactNode } from "react";
import { CopyLine } from "./CopyLine";

/** Key/value row used inside a Section.
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
      <span className="text-xs text-[var(--text-tertiary)] shrink-0 w-24">{label}</span>
      {right ? (
        <div className="flex-1 min-w-0">{right}</div>
      ) : mono ? (
        <div className="flex-1 min-w-0">
          <CopyLine value={value ?? ""} />
        </div>
      ) : (
        <span className="flex-1 min-w-0 text-xs leading-5 break-words text-[var(--text-primary)]">
          {value}
        </span>
      )}
    </div>
  );
}
