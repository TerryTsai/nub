import type { ReactNode } from "react";
import { Link } from "react-router-dom";

export interface Crumb {
  label: string;
  to?: string;
}

/** App shell — fixed top header with the nub mark, a breadcrumb path, and
 * an optional section-nav row below it. The body holds page content; a
 * filter bar (when present) lives at the top of the body, NOT in the header.
 * No per-page title or back button: navigation is via breadcrumb, and detail
 * pages render their own heading inside the body. */
export function Page({
  crumbs,
  nav,
  children,
  fab,
}: {
  crumbs?: Crumb[];
  /** Section-nav (e.g. Containers / Images / Volumes / Networks tabs).
   * Renders attached to the bottom of the app header, sticky with it. */
  nav?: ReactNode;
  children: ReactNode;
  /** Optional floating action — rendered fixed bottom-right. */
  fab?: ReactNode;
}) {
  return (
    <div className="min-h-full">
      <AppHeader crumbs={crumbs} nav={nav} />
      <main className="max-w-2xl mx-auto px-5 pt-3 pb-24 flex flex-col gap-4">
        {children}
      </main>
      {fab}
    </div>
  );
}

function AppHeader({ crumbs, nav }: { crumbs?: Crumb[]; nav?: ReactNode }) {
  const all: Crumb[] = [{ label: "nub", to: "/" }, ...(crumbs ?? [])];
  return (
    <header className="sticky top-0 z-30 bg-[var(--bg-base)] border-b border-[var(--border-subtle)]">
      <div className="flex items-center h-11 px-5 gap-2">
        <NubMark />
        <nav className="flex items-center gap-1 min-w-0 text-xs">
          {all.map((c, i) => (
            <CrumbLink key={i} crumb={c} last={i === all.length - 1} showSep={i > 0} />
          ))}
        </nav>
      </div>
      {nav && <div className="px-5 pb-2">{nav}</div>}
    </header>
  );
}

function CrumbLink({ crumb, last, showSep }: { crumb: Crumb; last: boolean; showSep: boolean }) {
  const cls = last
    ? "text-[var(--text-primary)] font-medium font-display text-sm truncate"
    : "text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] font-display text-sm truncate";
  return (
    <>
      {showSep && <span className="text-[var(--text-tertiary)] px-1 shrink-0">/</span>}
      {crumb.to && !last ? (
        <Link to={crumb.to} className={cls}>
          {crumb.label}
        </Link>
      ) : (
        <span className={cls}>{crumb.label}</span>
      )}
    </>
  );
}

/** 16x16 diamond — visually echoes foundry's mark without copying it. */
function NubMark() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true" className="shrink-0">
      <rect
        x="3.05"
        y="3.05"
        width="9.9"
        height="9.9"
        transform="rotate(45 8 8)"
        fill="none"
        stroke="var(--id-color)"
        strokeWidth="1.5"
      />
    </svg>
  );
}
