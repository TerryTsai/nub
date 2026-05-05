import { useState, type ReactNode } from "react";
import { copyText } from "@/lib/copy";
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
 * Mono values are tap-to-copy. A short tap copies the whole value and
 * briefly flashes "copied" inline (so the feedback is at the tap point,
 * not far away in a toast). Long-press still triggers the browser's
 * native selection menu so the user can grab a substring. */
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
  const [copied, setCopied] = useState(false);

  async function copy() {
    if (!value) return;
    const ok = await copyText(value);
    if (ok) {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1000);
    } else {
      toast.push("copy failed", "error");
    }
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
          className={`flex-1 min-w-0 text-left text-xs leading-5 whitespace-nowrap overflow-x-auto no-scrollbar mono cursor-pointer transition-colors ${
            copied ? "text-[var(--success)]" : "text-[var(--id-color)]"
          }`}
        >
          {copied ? "copied" : value}
        </button>
      ) : (
        <span className="flex-1 min-w-0 text-xs leading-5 break-words text-[var(--text-primary)]">
          {value}
        </span>
      )}
    </div>
  );
}
