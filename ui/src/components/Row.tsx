import type { ReactNode } from "react";
import { useToast } from "./Toaster";

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
 * Mono values are tap-to-copy. A short tap copies the whole value to the
 * clipboard with a toast; long-press still triggers the browser's native
 * selection menu so the user can grab a substring. */
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
  const toast = useToast();

  function copy() {
    if (!value) return;
    navigator.clipboard.writeText(value).then(
      () => toast.push("copied", "success"),
      () => toast.push("copy failed", "error"),
    );
  }

  return (
    <div className="flex gap-3 items-baseline select-text">
      <span className="text-xs text-[var(--text-tertiary)] shrink-0 w-24">{label}</span>
      {right ? (
        <div className="flex-1 min-w-0">{right}</div>
      ) : mono ? (
        <button
          type="button"
          onClick={copy}
          className="flex-1 min-w-0 text-left text-xs leading-5 break-words mono text-[var(--id-color)] cursor-pointer hover:underline underline-offset-2 decoration-[var(--border-subtle)]"
        >
          {value}
        </button>
      ) : (
        <span className="flex-1 min-w-0 text-xs leading-5 break-words text-[var(--text-primary)]">
          {value}
        </span>
      )}
    </div>
  );
}
