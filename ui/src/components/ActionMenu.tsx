import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Link } from "react-router-dom";

export interface ActionItem {
  label: string;
  /** Internal route — renders as a `<Link>`. Mutually exclusive with onClick. */
  to?: string;
  /** Click handler — renders as a `<button>`. Mutually exclusive with `to`. */
  onClick?: () => void;
  disabled?: boolean;
  /** Visual tone. Defaults to neutral; `destructive` renders red. */
  tone?: "default" | "destructive";
}

/** Subnav action group — one pill that opens a small popover of items.
 * Used to keep the subnav from filling up with secondary action pills.
 * The trigger is sized to match `.btn-sm` and the Combobox pill.
 *
 * The popover renders via a portal because the subnav strip uses
 * `overflow-x: auto`, which (per CSS spec) coerces overflow-y to auto and
 * would clip an in-place absolute popover invisibly. Portaling to body
 * sidesteps that. */
export function ActionMenu({
  label = "More",
  trigger,
  items,
}: {
  label?: string;
  /** Optional override for the trigger text (defaults to "More"). */
  trigger?: ReactNode;
  items: ActionItem[];
}) {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [pos, setPos] = useState<{ top: number; right: number } | null>(null);

  useLayoutEffect(() => {
    if (!open || !triggerRef.current) return;
    const r = triggerRef.current.getBoundingClientRect();
    setPos({ top: r.bottom + 4, right: window.innerWidth - r.right });
  }, [open]);

  useEffect(() => {
    if (!open) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setOpen(false);
    }
    function onResize() {
      if (!triggerRef.current) return;
      const r = triggerRef.current.getBoundingClientRect();
      setPos({ top: r.bottom + 4, right: window.innerWidth - r.right });
    }
    window.addEventListener("keydown", onKey);
    window.addEventListener("resize", onResize);
    window.addEventListener("scroll", onResize, true);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onResize);
      window.removeEventListener("scroll", onResize, true);
    };
  }, [open]);

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border border-[var(--border-subtle)] bg-transparent text-[var(--text-secondary)] active:opacity-80 transition-opacity"
      >
        <span>{trigger ?? label}</span>
        <Chevron />
      </button>
      {open && pos && createPortal(
        <>
          <div
            className="fixed inset-0 z-40"
            aria-hidden="true"
            onClick={() => setOpen(false)}
          />
          <div
            className="fixed min-w-[180px] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] py-1 z-50 shadow-lg"
            style={{ top: pos.top, right: pos.right }}
          >
            {items.map((it, i) => (
              <Item key={i} item={it} onSelect={() => setOpen(false)} />
            ))}
          </div>
        </>,
        document.body,
      )}
    </>
  );
}

function Item({ item, onSelect }: { item: ActionItem; onSelect: () => void }) {
  const tone = item.tone === "destructive"
    ? "text-[var(--error)]"
    : "text-[var(--text-secondary)]";
  const cls = `flex items-center gap-2 px-3 py-2.5 text-sm hover:bg-zinc-900/60 active:bg-zinc-900/80 ${
    item.disabled ? "opacity-30 pointer-events-none" : ""
  } ${tone}`;
  if (item.to && !item.disabled) {
    return (
      <Link to={item.to} onClick={onSelect} className={cls}>
        {item.label}
      </Link>
    );
  }
  return (
    <button
      type="button"
      onClick={() => {
        if (item.disabled) return;
        onSelect();
        item.onClick?.();
      }}
      disabled={item.disabled}
      className={`w-full text-left ${cls}`}
    >
      {item.label}
    </button>
  );
}

function Chevron() {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      aria-hidden="true"
      className="shrink-0 text-[var(--text-tertiary)]"
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
