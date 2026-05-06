import { useState } from "react";
import { copyText } from "@/lib/copy";

/** A copyable, horizontally-scrollable single line of text. Used as the
 * value side of a Row, or stand-alone for variable-length lists (env
 * vars, labels). Tap copies the whole value with a brief inline "copied"
 * flash; long-press still triggers the browser's native selection menu.
 *
 * Defaults to `mono` (amber identifier treatment). Pass `mono={false}` for
 * classification-style values (counts, enums, yes/no) where the body font
 * and primary text tone are appropriate.
 *
 * Pass `copyValue` when the displayed text is a slice of what should
 * actually go to the clipboard — e.g. an env var renders only the
 * VALUE half but should copy `KEY=VALUE`.
 *
 * Pass `dim` to render in tertiary — used for the key half of a KvLine,
 * where the key follows the same tertiary treatment Row uses for static
 * keys. */
export function CopyLine({
  value,
  copyValue,
  mono = true,
  dim,
}: {
  value: string;
  copyValue?: string;
  mono?: boolean;
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
    : mono
    ? "text-[var(--id-color)]"
    : "text-[var(--text-primary)]";

  return (
    <div className="min-w-0 overflow-x-auto no-scrollbar scroll-fade-r">
      <button
        type="button"
        onClick={copy}
        aria-label={`copy ${target}`}
        className={`block text-left text-base leading-snug whitespace-nowrap cursor-pointer transition-colors active:opacity-75 ${mono ? "mono" : ""} ${tone}`}
      >
        {copied ? "copied" : value}
      </button>
    </div>
  );
}
