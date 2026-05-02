import { Link } from "react-router-dom";

/** Floating action button — bottom-right pill with sky-soft fill. The
 * primary "create" affordance on list pages. */
export function FAB({ to, label }: { to: string; label: string }) {
  return (
    <Link
      to={to}
      className="fixed bottom-[calc(1rem+env(safe-area-inset-bottom))] right-4 z-20 flex items-center gap-1.5 px-4 py-2 rounded-full text-sm font-medium bg-[var(--accent-fab)] border border-[var(--accent-border)] text-[var(--accent)] shadow-lg active:opacity-75 active:scale-[0.97] transition-[opacity,transform] duration-150"
      style={{ transform: "translateZ(0)" }}
    >
      <span className="text-[var(--accent)]">+</span>
      <span>{label}</span>
    </Link>
  );
}
