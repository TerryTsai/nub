import type { ReactNode } from "react";
import { Link, useNavigate } from "react-router-dom";
import { usePageConfig } from "./Layout";

export type Crumb =
  | { kind: "link"; label: string; to?: string }
  | { kind: "menu"; node: ReactNode };

/** Page declares its layout config (crumbs, sub-nav, optional FAB, fill
 * mode) via context. The Layout shell — mounted once for the whole app —
 * picks those up and renders the AppHeader / FAB. The actual page body
 * is just `children`, mounted inside the Layout's <Outlet/>.
 *
 * No DOM is created for the chrome by this component. That's deliberate:
 * the AppHeader stays mounted across navigations so back-nav doesn't
 * flicker through a moment of bare body. */
export function Page({
  crumbs,
  subnav,
  children,
  fab,
  fill,
}: {
  crumbs?: Crumb[];
  subnav?: ReactNode;
  children: ReactNode;
  fab?: ReactNode;
  fill?: boolean;
}) {
  usePageConfig({ crumbs, subnav, fab, fill });
  return <>{children}</>;
}

/** Renders the breadcrumb header. Lives in Layout; exported for Layout
 * only — pages never instantiate it directly.
 *
 * Position is `fixed` (not sticky) so it stays glued to the viewport
 * regardless of how the document scrolls. Sticky on iOS Safari has
 * intermittent offset bugs during scroll restoration; fixed avoids them.
 *
 * `top: env(safe-area-inset-top)` keeps the header below the notch —
 * body already has matching safe-area padding so the notch region shows
 * the body bg. */
export function AppHeader({ crumbs, subnav }: { crumbs?: Crumb[]; subnav?: ReactNode }) {
  const all = crumbs ?? [];
  const navigate = useNavigate();
  return (
    <header className="fixed top-[env(safe-area-inset-top)] left-0 right-0 z-30 bg-[var(--bg-base)] border-b border-[var(--border-subtle)]">
      <div className="flex items-center h-11 px-5 gap-2">
        <Link to="/" aria-label="hosts" className="shrink-0 flex items-center">
          <NubMark />
        </Link>
        {all.length > 0 && (
          <button
            type="button"
            aria-label="back"
            onClick={() => navigate(-1)}
            className="shrink-0 p-1.5 -ml-1 text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] active:opacity-75 transition-opacity"
          >
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <path
                d="M6.5 1.5L2.5 5l4 3.5"
                stroke="currentColor"
                fill="none"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}
        <nav className="flex items-center gap-1 min-w-0 text-xs">
          {all.map((c, i) => (
            <Segment key={i} crumb={c} last={i === all.length - 1} showSep={i > 0} />
          ))}
        </nav>
      </div>
      {subnav && (
        <div className="px-5 h-10 flex items-center gap-2 border-t border-[var(--border-subtle)] overflow-x-auto no-scrollbar scroll-fade-r [&>*]:shrink-0">
          {subnav}
        </div>
      )}
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
  const baseCls = "font-display text-sm truncate";
  const lastCls = `text-[var(--text-primary)] font-medium ${baseCls}`;
  const linkCls = `text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] underline underline-offset-2 decoration-[var(--border-subtle)] ${baseCls}`;
  return (
    <>
      {sep}
      {crumb.to && !last ? (
        <Link to={crumb.to} className={linkCls}>
          {crumb.label}
        </Link>
      ) : (
        <span className={lastCls}>{crumb.label}</span>
      )}
    </>
  );
}

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
