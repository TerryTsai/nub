import { useState } from "react";
import { copyText } from "@/lib/copy";

/** A copyable, horizontally-scrollable single line of mono amber text.
 * Used as the value side of a Row, or stand-alone for variable-length
 * lists (env vars, labels) where each line should be tappable on its
 * own. Tap copies the whole value with a brief inline "copied" flash;
 * long-press still triggers the browser's native selection menu. */
export function CopyLine({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    if (!value) return;
    const ok = await copyText(value);
    if (ok) {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1000);
    }
  }

  return (
    <div className="min-w-0 overflow-x-auto no-scrollbar">
      <button
        type="button"
        onClick={copy}
        className={`block text-left text-xs leading-5 whitespace-nowrap mono cursor-pointer transition-colors ${
          copied ? "text-[var(--success)]" : "text-[var(--id-color)]"
        }`}
      >
        {copied ? "copied" : value}
      </button>
    </div>
  );
}
