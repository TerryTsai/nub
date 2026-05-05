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
 * Closed: looks like an `.input` with a trailing chevron. Open: dialog
 * with an optional filter/free-text field and a scrollable option list.
 *
 * Set `freeText` to allow values not in `options`. The filter input
 * doubles as the typed value, and a "Use «query»" footer row commits
 * the typed text on tap. */
export function Combobox({
  value,
  onChange,
  options,
  freeText,
  placeholder,
  freeTextHint,
  mono,
  cell,
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
            : `input flex items-center justify-between text-left ${mono ? "mono" : ""} ${
                displayLabel ? "" : "text-[var(--text-tertiary)]"
              }`
        }
      >
        <span className="truncate">{displayLabel || placeholder || "Select…"}</span>
        <span
          className={
            cell
              ? "text-[var(--text-tertiary)] text-[10px] shrink-0"
              : "text-[var(--text-tertiary)] text-sm pl-2 shrink-0"
          }
        >
          ▾
        </span>
      </button>
      <Dialog.Root open={open} onOpenChange={setOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/60 z-40" />
          <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(92vw,420px)] max-h-[80vh] flex flex-col bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] z-50 overflow-hidden">
            <Dialog.Title className="sr-only">Pick a value</Dialog.Title>
            {(freeText || options.length > 5) && (
              <div className="p-3 border-b border-[var(--border-subtle)]">
                <input
                  className={`input ${mono ? "mono" : ""}`}
                  type="text"
                  autoFocus
                  autoCapitalize="off"
                  autoCorrect="off"
                  spellCheck={false}
                  placeholder={freeText ? freeTextHint || "type or pick" : "filter"}
                  value={filter}
                  onChange={(e) => setFilter(e.target.value)}
                />
              </div>
            )}
            <div className="overflow-y-auto">
              {showCustomRow && (
                <button
                  type="button"
                  onClick={() => pick(filter.trim())}
                  className="w-full text-left px-3 py-3 flex items-baseline gap-2 border-b border-[var(--border-subtle)] active:bg-[var(--bg-elevated)]"
                >
                  <span className="text-[11px] uppercase tracking-wide text-[var(--text-tertiary)]">use</span>
                  <span className={`text-sm truncate ${mono ? "mono" : ""}`}>{filter.trim()}</span>
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
