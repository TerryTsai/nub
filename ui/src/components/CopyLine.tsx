import { useState } from "react";
import { copyText } from "@/lib/copy";

/** A copyable, horizontally-scrollable single line of mono text.
 * Used as the value side of a Row, or stand-alone for variable-length
 * lists (env vars, labels) where each line should be tappable on its
 * own. Tap copies the whole value with a brief inline "copied" flash;
 * long-press still triggers the browser's native selection menu.
 *
 * Pass `copyValue` when the displayed text is a slice of what should
 * actually go to the clipboard — e.g. an env var renders only the
 * VALUE half but should copy `KEY=VALUE`.
 *
 * Pass `dim` to render in tertiary instead of amber — used for the
 * key half of a KvLine, where the key sits in the label position and
 * follows the same tertiary-mono treatment Row uses for static keys. */
export function CopyLine({
  value,
  copyValue,
  dim,
}: {
  value: string;
  copyValue?: string;
  dim?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const target = copyValue ?? value;

  async function copy() {
    if (!target) return;
    const ok = await copyText(target);
    if (ok) {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1000);
    }
  }

  const tone = copied
    ? "text-[var(--success)]"
    : dim
    ? "text-[var(--text-tertiary)]"
    : "text-[var(--id-color)]";

  return (
    <div className="min-w-0 overflow-x-auto no-scrollbar">
      <button
        type="button"
        onClick={copy}
        className={`block text-left text-xs leading-5 whitespace-nowrap mono cursor-pointer transition-colors ${tone}`}
      >
        {copied ? "copied" : value}
      </button>
    </div>
  );
}
