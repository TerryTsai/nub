import type { ReactNode } from "react";
import { Link } from "react-router-dom";

export type Crumb =
  | { kind: "link"; label: string; to?: string }
  | { kind: "menu"; node: ReactNode };

/** App shell — single-row top header with the nub mark and a breadcrumb
 * path. Breadcrumb segments can be plain links or dropdown menus
 * (workspace/section pickers, foundry-style). The body holds page content;
 * the row immediately below the header is reserved for a future filter bar.
 */
export function Page({
  crumbs,
  children,
  fab,
  fill,
}: {
  crumbs?: Crumb[];
  children: ReactNode;
  /** Optional floating action — rendered fixed bottom-right. */
  fab?: ReactNode;
  /** Fill the viewport. The body becomes a flex column that doesn't scroll;
   * children are responsible for managing their own scroll regions. Used for
   * pages like the exec terminal that own their full-screen surface. */
  fill?: boolean;
}) {
  if (fill) {
    return (
      <div className="h-full flex flex-col overflow-hidden">
        <AppHeader crumbs={crumbs} />
        <main className="flex-1 min-h-0 flex flex-col">{children}</main>
        {fab}
      </div>
    );
  }
  return (
    <div className="min-h-full">
      <AppHeader crumbs={crumbs} />
      <main className="max-w-2xl mx-auto px-5 pt-3 pb-24 flex flex-col gap-4">
        {children}
      </main>
      {fab}
    </div>
  );
}

function AppHeader({ crumbs }: { crumbs?: Crumb[] }) {
  const all: Crumb[] = [{ kind: "link", label: "nub", to: "/" }, ...(crumbs ?? [])];
  return (
    <header className="sticky top-0 z-30 bg-[var(--bg-base)] border-b border-[var(--border-subtle)]">
      <div className="flex items-center h-11 px-5 gap-2">
        <NubMark />
        <nav className="flex items-center gap-1 min-w-0 text-xs">
          {all.map((c, i) => (
            <Segment key={i} crumb={c} last={i === all.length - 1} showSep={i > 0} />
          ))}
        </nav>
      </div>
    </header>
  );
}

function Segment({ crumb, last, showSep }: { crumb: Crumb; last: boolean; showSep: boolean }) {
  const sep = showSep && (
    <span className="text-[var(--text-tertiary)] px-1 shrink-0">/</span>
  );
  if (crumb.kind === "menu") {
    return (
      <>
        {sep}
        {crumb.node}
      </>
    );
  }
  const cls = last
    ? "text-[var(--text-primary)] font-medium font-display text-sm truncate"
    : "text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] font-display text-sm truncate";
  return (
    <>
      {sep}
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
