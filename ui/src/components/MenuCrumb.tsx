import { useState } from "react";
import { Link } from "react-router-dom";

export interface MenuItem {
  label: string;
  to: string;
  current?: boolean;
  /** Optional small right-aligned annotation (e.g. "archived"). */
  meta?: string;
  /** Optional leading + glyph. Use for "add" affordances at the bottom. */
  add?: boolean;
}

/** Breadcrumb segment that opens a small popover menu on click.
 *
 * When open, a transparent fixed backdrop covers the rest of the
 * viewport. Tapping the backdrop closes the menu *and* swallows the
 * click — important on phones where a stray tap on a list row would
 * otherwise navigate while you were just trying to dismiss. The backdrop
 * sits at z-40 (above the z-30 header), the menu at z-50 (above the
 * backdrop). */
export function MenuCrumb({ label, items }: { label: string; items: MenuItem[] }) {
  const [open, setOpen] = useState(false);

  function dismiss(e: React.MouseEvent) {
    e.stopPropagation();
    e.preventDefault();
    setOpen(false);
  }

  return (
    <div className="relative inline-flex items-baseline">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="appearance-none bg-transparent border-0 p-0 m-0 cursor-pointer inline-flex items-baseline"
      >
        <span className="text-[var(--text-primary)] font-medium font-display text-sm underline underline-offset-2 decoration-[var(--border-subtle)]">
          {label}
        </span>
      </button>
      {open && (
        <>
          <div
            className="fixed inset-0 z-40"
            aria-hidden="true"
            onClick={dismiss}
            onPointerDown={dismiss}
          />
          <div className="absolute left-0 top-full mt-1 min-w-[180px] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] py-1 z-50 shadow-lg">
            {items.map((it, i) => (
              <Link
                key={i}
                to={it.to}
                onClick={() => setOpen(false)}
                className="flex items-center gap-2 px-3 py-1.5 text-xs hover:bg-zinc-900/60"
              >
                {it.add && <span className="text-[var(--accent)]">+</span>}
                {!it.add && <Check current={!!it.current} />}
                <span className={it.current ? "text-[var(--text-primary)]" : "text-[var(--text-secondary)]"}>
                  {it.label}
                </span>
                {it.meta && (
                  <span className="ml-auto text-[10px] text-[var(--text-tertiary)]">{it.meta}</span>
                )}
              </Link>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function Check({ current }: { current: boolean }) {
  return (
    <span className="w-3 inline-flex items-center justify-center">
      {current && (
        <svg width="10" height="10" viewBox="0 0 10 10" className="text-[var(--id-color)]" aria-hidden="true">
          <path d="M2 5.5l2 2 4-5" stroke="currentColor" fill="none" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      )}
    </span>
  );
}
