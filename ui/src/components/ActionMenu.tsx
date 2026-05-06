import { useEffect, useState, type ReactNode } from "react";
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
 * The trigger is sized like `.btn-sm`; items inside use the same density
 * as MenuCrumb's menu rows. */
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

  useEffect(() => {
    if (!open) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setOpen(false);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  function dismiss(e: React.MouseEvent) {
    e.stopPropagation();
    e.preventDefault();
    setOpen(false);
  }

  return (
    <div className="relative inline-flex">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium border border-[var(--border-subtle)] bg-transparent text-[var(--text-secondary)] active:opacity-80 transition-opacity"
      >
        <span>{trigger ?? label}</span>
        <Chevron />
      </button>
      {open && (
        <>
          <div
            className="fixed inset-0 z-40"
            aria-hidden="true"
            onClick={dismiss}
            onPointerDown={dismiss}
          />
          <div className="absolute right-0 top-full mt-1 min-w-[180px] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] py-1 z-50 shadow-lg">
            {items.map((it, i) => (
              <Item key={i} item={it} onSelect={() => setOpen(false)} />
            ))}
          </div>
        </>
      )}
    </div>
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
