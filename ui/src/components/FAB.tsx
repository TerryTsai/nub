import { Link } from "react-router-dom";

/** Floating action button — bottom-right circular "+" with sky-soft fill.
 * The primary "create" affordance on list pages. Icon-only; the page
 * breadcrumbs already say what's being added, so a label would just
 * repeat context. `label` is the noun (e.g. "container") and is exposed
 * to screen readers via aria-label. */
export function FAB({ to, label }: { to: string; label: string }) {
  return (
    <Link
      to={to}
      aria-label={`add ${label}`}
      className="fixed bottom-[calc(1rem+env(safe-area-inset-bottom))] right-4 z-20 flex items-center justify-center w-12 h-12 rounded-full bg-[var(--accent-soft)] border border-[var(--accent-border)] text-[var(--accent)] shadow-lg active:opacity-75 active:scale-[0.97] transition-[opacity,transform] duration-150"
    >
      <svg width="20" height="20" viewBox="0 0 20 20" aria-hidden="true">
        <path
          d="M10 4v12M4 10h12"
          stroke="currentColor"
          strokeWidth="1.75"
          strokeLinecap="round"
        />
      </svg>
    </Link>
  );
}
