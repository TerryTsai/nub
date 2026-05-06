import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";

export interface ComboOption {
  value: string;
  /** Primary text shown in the option row and trigger. Defaults to `value`. */
  label?: string;
  /** Optional secondary text rendered muted under the label. */
  sublabel?: string;
}

/** Bottom-sheet style picker that replaces both `<select>` (which is
 * unstylable on iOS) and horizontal chip rows (which scale poorly past a
 * handful of options).
 *
 * Closed: looks like an `.input` with a trailing chevron. Open: a dialog
 * with the same chrome regardless of mode — filter input on top, option
 * list below.
 *
 * Set `freeText` to allow values not in `options`. The filter input
 * doubles as the typed value; when the typed text doesn't match an
 * existing option, a leading "+ <typed>" row appears so the user can
 * commit it. */
export function Combobox({
  value,
  onChange,
  options,
  freeText,
  placeholder,
  freeTextHint,
  mono,
  cell,
  pill,
  attribute,
  dim,
}: {
  value: string;
  onChange: (v: string) => void;
  options: ComboOption[];
  freeText?: boolean;
  placeholder?: string;
  freeTextHint?: string;
  /** Use mono font for value/options — reads better for image refs etc. */
  mono?: boolean;
  /** Render the trigger as an inline spec cell (mono amber + chev, no input
   * border) instead of a full-width `.input` button. */
  cell?: boolean;
  /** Render the trigger as a subnav-style pill — same geometry as `.btn-sm`,
   * with an optional `attribute` prefix (`status: All ▾`). Use for filter
   * subnavs where each filterable axis becomes one pill that opens a sheet. */
  pill?: boolean;
  /** Pill-only: a small label rendered in tertiary tone before the value,
   * e.g. "status" for a filter pill that reads `status: All ▾`. */
  attribute?: string;
  /** Cell-only: render the value in placeholder tone (tertiary italic)
   * instead of amber. Use when the current value is a default the user
   * hasn't actively chosen, so the cell reads as "system default" rather
   * than "configured". */
  dim?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [filter, setFilter] = useState("");

  const selected = options.find((o) => o.value === value);
  const displayLabel = selected?.label ?? value ?? "";

  const q = filter.trim().toLowerCase();
  const filtered = q
    ? options.filter(
        (o) => o.value.toLowerCase().includes(q) || o.label?.toLowerCase().includes(q),
      )
    : options;
  const showCustomRow =
    freeText && filter.trim() !== "" && !options.some((o) => o.value === filter.trim());

  function pick(v: string) {
    onChange(v);
    setOpen(false);
    setFilter("");
  }

  return (
    <>
      <button
        type="button"
        onClick={() => {
          setOpen(true);
          setFilter("");
        }}
        className={
          cell
            ? `inline-flex items-center gap-1 text-left max-w-full text-base leading-snug py-0 border-b border-dashed border-transparent hover:border-[var(--border-strong)] transition-colors ${
                mono ? "mono" : ""
              } ${
                !displayLabel || dim
                  ? "text-[var(--text-tertiary)] italic"
                  : "text-[var(--id-color)]"
              }`
            : pill
            ? "shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium border border-[var(--border-subtle)] bg-transparent text-[var(--text-secondary)] active:opacity-80 transition-opacity"
            : `input flex items-center justify-between text-left ${mono ? "mono" : ""} ${
                displayLabel ? "" : "text-[var(--text-tertiary)]"
              }`
        }
      >
        {pill && attribute && (
          <span className="text-[var(--text-tertiary)]">{attribute}:</span>
        )}
        <span className={pill ? "" : "truncate"}>{displayLabel || placeholder || "Select…"}</span>
        <Chevron className={cell || pill ? "shrink-0" : "shrink-0 ml-2"} />
      </button>
      <Dialog.Root open={open} onOpenChange={setOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/60 z-40" />
          <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(92vw,420px)] max-h-[80vh] flex flex-col bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] z-50 overflow-hidden">
            <Dialog.Title className="sr-only">Pick a value</Dialog.Title>
            <div className="p-3 border-b border-[var(--border-subtle)]">
              <input
                className={`input ${mono ? "mono" : ""}`}
                type="text"
                autoFocus
                autoCapitalize="off"
                autoCorrect="off"
                spellCheck={false}
                placeholder={freeText ? freeTextHint || "filter or type" : "filter"}
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
              />
            </div>
            <div className="overflow-y-auto">
              {showCustomRow && (
                <button
                  type="button"
                  onClick={() => pick(filter.trim())}
                  className="w-full text-left px-3 py-3 flex items-center gap-2 border-b border-[var(--border-subtle)] active:bg-[var(--bg-elevated)]"
                >
                  <span className={`text-sm truncate flex-1 min-w-0 ${mono ? "mono" : ""}`}>
                    {filter.trim()}
                  </span>
                  <span className="text-[var(--accent)] shrink-0" aria-hidden="true">+</span>
                </button>
              )}
              {filtered.length === 0 && !showCustomRow && (
                <div className="px-3 py-6 text-center text-[var(--text-tertiary)] text-xs">
                  No matches.
                </div>
              )}
              {filtered.map((o) => {
                const active = o.value === value;
                return (
                  <button
                    key={o.value}
                    type="button"
                    onClick={() => pick(o.value)}
                    className="w-full text-left px-3 py-3 flex flex-col gap-0.5 border-b border-[var(--border-subtle)] last:border-b-0 active:bg-[var(--bg-elevated)]"
                  >
                    <span className={`text-sm truncate ${mono ? "mono" : ""} ${active ? "text-[var(--accent)]" : ""}`}>
                      {o.label ?? o.value}
                    </span>
                    {o.sublabel && (
                      <span className="text-[11px] text-[var(--text-tertiary)] mono truncate">
                        {o.sublabel}
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}

function Chevron({ className = "" }: { className?: string }) {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      aria-hidden="true"
      className={`text-[var(--text-tertiary)] ${className}`}
    >
      <path
        d="M2 4l3 3 3-3"
        stroke="currentColor"
        fill="none"
        strokeWidth="1.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
