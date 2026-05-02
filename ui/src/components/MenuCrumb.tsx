import { useEffect, useRef, useState } from "react";
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

/** Breadcrumb segment that opens a small popover menu on click. Mirrors
 * foundry's `Sandbox ^` pattern — looks like a normal crumb but with a
 * caret, opens a list of options on tap. The trigger uses `font: inherit`
 * so it renders identically to neighbouring link segments (no <button>
 * font-family fallback). */
export function MenuCrumb({ label, items }: { label: string; items: MenuItem[] }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDocClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [open]);

  return (
    <div ref={ref} className="relative inline-flex items-baseline">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="appearance-none bg-transparent border-0 p-0 m-0 cursor-pointer text-[var(--text-primary)] font-medium font-display text-sm leading-none"
        style={{ font: "inherit" }}
      >
        <span className="font-display font-medium">{label}</span>
        <span className="text-[var(--text-tertiary)] ml-0.5 text-[10px]">▾</span>
      </button>
      {open && (
        <div className="absolute left-0 top-full mt-1 min-w-[180px] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] py-1 z-40 shadow-lg">
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
